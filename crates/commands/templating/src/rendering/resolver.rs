/// Async template file resolution with caching and strategy pattern
use crate::core::error::{TemplateError, TemplateResult};
use crate::strategies::LanguageStrategyFactory;
use dashmap::DashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;
use walkdir::WalkDir;

/// Template file resolver with async operations and caching
///
/// # Architecture
/// - **Strategy Pattern**: Uses LanguageStrategy for language-specific path normalization
/// - **Factory Pattern**: LanguageStrategyFactory instantiates strategies on-demand
/// - **Cache**: DashMap for thread-safe O(1) path lookups (reduces filesystem I/O)
/// - **Async**: Non-blocking I/O with tokio::fs
///
/// # Performance
/// - Cache hit: O(1) lookup, zero filesystem I/O
/// - Cache miss: 3 fallback strategies (direct, normalized, recursive)
/// - Thread-safe: DashMap allows concurrent reads/writes
pub struct TemplateResolver {
    templates_root: PathBuf,
    factory: Arc<LanguageStrategyFactory>,
    path_cache: Arc<DashMap<String, PathBuf>>,
}

impl TemplateResolver {
    /// Create a new resolver with the given templates root directory
    ///
    /// # Performance
    /// - Initializes factory (O(1) - pre-registered strategies)
    /// - Creates DashMap cache (lock-free concurrent HashMap)
    pub fn new<P: AsRef<Path>>(templates_root: P) -> Self {
        Self {
            templates_root: templates_root.as_ref().to_path_buf(),
            factory: Arc::new(LanguageStrategyFactory::new()),
            path_cache: Arc::new(DashMap::new()),
        }
    }

    /// Clear the path cache (useful for testing or after template changes)
    pub fn clear_cache(&self) {
        self.path_cache.clear();
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> (usize, usize) {
        let len = self.path_cache.len();
        let capacity = self.path_cache.capacity();
        (len, capacity)
    }

    /// Resolve a template path to an absolute file path (async)
    ///
    /// # Resolution Strategies (in order)
    /// 1. **Cache lookup**: O(1) - Check DashMap cache first
    /// 2. **Direct path**: Check if path exists relative to templates_root
    /// 3. **Strategy normalization**: Use LanguageStrategy to normalize path (e.g., insert "src/")
    /// 4. **Recursive search**: Fallback to filename search (slowest, cached for future)
    ///
    /// # Performance
    /// - Cache hit: ~10-50ns (DashMap lookup)
    /// - Cache miss + direct: ~1-10μs (filesystem metadata check)
    /// - Cache miss + normalized: ~2-20μs (2x filesystem checks)
    /// - Cache miss + recursive: ~100μs-10ms (depends on tree size, worst case)
    ///
    /// # Example
    ///
    /// ```no_run
    /// use nettoolskit_templating::TemplateResolver;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let resolver = TemplateResolver::new("templates");
    ///
    /// // First call: cache miss, ~10μs
    /// let path = resolver.resolve("dotnet/Domain/Entity.cs.hbs").await?;
    ///
    /// // Second call: cache hit, ~50ns (200x faster!)
    /// let path = resolver.resolve("dotnet/Domain/Entity.cs.hbs").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn resolve(&self, template: &str) -> TemplateResult<PathBuf> {
        // Strategy 0: Cache lookup (fastest - O(1) with no I/O)
        if let Some(cached) = self.path_cache.get(template) {
            return Ok(cached.clone());
        }

        // Strategy 1: Direct path
        let direct = self.templates_root.join(template);
        if fs::metadata(&direct).await.is_ok() {
            self.path_cache.insert(template.to_string(), direct.clone());
            return Ok(direct);
        }

        // Strategy 2: Language-specific path normalization using Factory
        if let Some(strategy) = self.factory.detect_from_path(template) {
            let parts: Vec<&str> = template.split('/').collect();
            if let Some(normalized) = strategy.normalize_path(&parts) {
                let alt = self.templates_root.join(&normalized);
                if fs::metadata(&alt).await.is_ok() {
                    self.path_cache.insert(template.to_string(), alt.clone());
                    return Ok(alt);
                }
            }
        }

        // Strategy 3: Recursive search by filename (slowest, but cached)
        if let Some(found) = self.search_by_filename(template).await {
            self.path_cache.insert(template.to_string(), found.clone());
            return Ok(found);
        }

        Err(TemplateError::NotFound {
            template: template.to_string(),
        })
    }

    /// Search for a template file by filename recursively (async)
    ///
    /// # Performance
    /// - Worst case: O(n) where n = number of files in tree
    /// - Uses WalkDir (synchronous) but runs in tokio::task::spawn_blocking
    /// - Result is cached in DashMap for future O(1) lookups
    async fn search_by_filename(&self, template: &str) -> Option<PathBuf> {
        let file_name = Path::new(template).file_name()?.to_owned();
        let templates_root = self.templates_root.clone();

        // Run blocking WalkDir in separate thread pool to avoid blocking tokio runtime
        tokio::task::spawn_blocking(move || {
            for entry in WalkDir::new(&templates_root)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                if entry.file_type().is_file() && entry.file_name() == file_name.as_os_str() {
                    return Some(entry.path().to_path_buf());
                }
            }
            None
        })
        .await
        .ok()?
    }
}
