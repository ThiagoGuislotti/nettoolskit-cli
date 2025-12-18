use anyhow::Result;
use globset::{Glob, GlobSetBuilder};
use ignore::WalkBuilder;
use std::path::{Path, PathBuf};

/// File search configuration
#[derive(Debug, Clone)]
pub struct SearchConfig {
    /// Patterns to include
    pub include_patterns: Vec<String>,
    /// Patterns to exclude
    pub exclude_patterns: Vec<String>,
    /// Maximum depth to search
    pub max_depth: Option<usize>,
    /// Follow symbolic links
    pub follow_links: bool,
    /// Include hidden files
    pub include_hidden: bool,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            include_patterns: vec!["*".to_string()],
            exclude_patterns: Vec::new(),
            max_depth: None,
            follow_links: false,
            include_hidden: false,
        }
    }
}

/// Search for files matching the given configuration
///
/// # Errors
/// Returns an error if any glob pattern in `config` is invalid, if building the glob sets fails,
/// or if directory traversal fails.
pub fn search_files<P: AsRef<Path>>(root: P, config: &SearchConfig) -> Result<Vec<PathBuf>> {
    let root = root.as_ref();

    // Build include globset
    let mut include_builder = GlobSetBuilder::new();
    for pattern in &config.include_patterns {
        include_builder.add(Glob::new(pattern)?);
    }
    let include_set = include_builder.build()?;

    // Build exclude globset
    let mut exclude_builder = GlobSetBuilder::new();
    for pattern in &config.exclude_patterns {
        exclude_builder.add(Glob::new(pattern)?);
    }
    let exclude_set = exclude_builder.build()?;

    let mut walker = WalkBuilder::new(root);
    walker
        .follow_links(config.follow_links)
        .hidden(!config.include_hidden);

    if let Some(max_depth) = config.max_depth {
        walker.max_depth(Some(max_depth));
    }

    let mut results = Vec::new();

    for entry in walker.build() {
        let entry = entry?;
        let path = entry.path();

        if !entry.file_type().is_some_and(|ft| ft.is_file()) {
            continue;
        }

        // Check include patterns
        if !include_set.is_match(path) {
            continue;
        }

        // Check exclude patterns
        if exclude_set.is_match(path) {
            continue;
        }

        results.push(path.to_path_buf());
    }

    Ok(results)
}

/// Search for files asynchronously with parallel processing
///
/// # Errors
/// Returns an error if spawning the blocking task fails, or if the underlying
/// [`search_files`] call fails.
pub async fn search_files_async<P: AsRef<Path>>(
    root: P,
    config: &SearchConfig,
) -> Result<Vec<PathBuf>> {
    use tokio::task;

    let root = root.as_ref().to_path_buf();
    let config = config.clone();

    task::spawn_blocking(move || search_files(&root, &config)).await?
}

/// Search for files in multiple directories concurrently
///
/// # Errors
/// Returns the first error produced by any of the underlying searches.
pub async fn search_files_concurrent<P: AsRef<Path>>(
    roots: Vec<P>,
    config: &SearchConfig,
) -> Result<Vec<PathBuf>> {
    use futures::future::join_all;

    let futures: Vec<_> = roots
        .into_iter()
        .map(|root| search_files_async(root, config))
        .collect();

    let results = join_all(futures).await;
    let mut all_files = Vec::new();

    for result in results {
        match result {
            Ok(files) => all_files.extend(files),
            Err(e) => return Err(e),
        }
    }

    Ok(all_files)
}

/// Find template files in a directory
///
/// # Errors
/// Returns an error if glob compilation fails or if directory traversal fails.
pub fn find_templates<P: AsRef<Path>>(root: P) -> Result<Vec<PathBuf>> {
    let config = SearchConfig {
        include_patterns: vec!["*.hbs".to_string(), "*.template".to_string()],
        exclude_patterns: vec!["**/target/**".to_string(), "**/node_modules/**".to_string()],
        max_depth: Some(10),
        follow_links: false,
        include_hidden: false,
    };

    search_files(root, &config)
}

/// Find template files asynchronously
///
/// # Errors
/// Returns an error if spawning the blocking task fails, or if the underlying
/// search fails.
pub async fn find_templates_async<P: AsRef<Path>>(root: P) -> Result<Vec<PathBuf>> {
    let config = SearchConfig {
        include_patterns: vec!["*.hbs".to_string(), "*.template".to_string()],
        exclude_patterns: vec!["**/target/**".to_string(), "**/node_modules/**".to_string()],
        max_depth: Some(10),
        follow_links: false,
        include_hidden: false,
    };

    search_files_async(root, &config).await
}

/// Find manifest files in a directory
///
/// # Errors
/// Returns an error if glob compilation fails or if directory traversal fails.
pub fn find_manifests<P: AsRef<Path>>(root: P) -> Result<Vec<PathBuf>> {
    let config = SearchConfig {
        include_patterns: vec![
            "*.yml".to_string(),
            "*.yaml".to_string(),
            "ntk-*.yml".to_string(),
        ],
        exclude_patterns: vec!["**/target/**".to_string(), "**/node_modules/**".to_string()],
        max_depth: Some(5),
        follow_links: false,
        include_hidden: false,
    };

    search_files(root, &config)
}
