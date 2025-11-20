/// Batch rendering with parallelism for high-throughput template generation
use super::engine::TemplateEngine;
use crate::core::error::{TemplateError, TemplateResult};
use super::resolver::TemplateResolver;
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::task::JoinSet;

/// Request for rendering a single template
#[derive(Debug, Clone)]
pub struct RenderRequest<T> {
    /// Template path (resolved by TemplateResolver)
    pub template: String,
    /// Data to render with
    pub data: T,
    /// Output file path
    pub output: PathBuf,
}

/// Result of a batch render operation
#[derive(Debug)]
pub struct BatchRenderResult {
    /// Number of successfully rendered templates
    pub succeeded: usize,
    /// Number of failed renders
    pub failed: usize,
    /// Errors encountered (if any)
    pub errors: Vec<(String, TemplateError)>,
    /// Total time taken
    pub duration: std::time::Duration,
}

/// High-performance batch renderer with parallelism
///
/// # Architecture
/// - **Parallelism**: Uses tokio::spawn for concurrent rendering (N templates in parallel)
/// - **Bounded Concurrency**: JoinSet limits concurrent tasks to prevent resource exhaustion
/// - **Shared Resources**: Arc-wrapped Engine + Resolver for zero-copy sharing
///
/// # Performance
/// - Sequential: N templates × 100μs = N×100μs total
/// - Parallel (10 workers): N templates ÷ 10 × 100μs = N×10μs total (10x speedup)
/// - Scales linearly with CPU cores (up to I/O bottleneck)
///
/// # Example
/// ```no_run
/// use nettoolskit_templating::{BatchRenderer, RenderRequest};
/// use serde_json::json;
/// use std::path::PathBuf;
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let renderer = BatchRenderer::new("templates");
///
/// let requests = vec![
///     RenderRequest {
///         template: "dotnet/Domain/Entity.cs.hbs".to_string(),
///         data: json!({"name": "User"}),
///         output: PathBuf::from("output/User.cs"),
///     },
///     RenderRequest {
///         template: "dotnet/Domain/Entity.cs.hbs".to_string(),
///         data: json!({"name": "Product"}),
///         output: PathBuf::from("output/Product.cs"),
///     },
/// ];
///
/// // Renders in parallel (2x faster than sequential)
/// let result = renderer.render_batch(requests).await?;
/// println!("Rendered {} files in {:?}", result.succeeded, result.duration);
/// # Ok(())
/// # }
/// ```
pub struct BatchRenderer {
    engine: Arc<TemplateEngine>,
    resolver: Arc<TemplateResolver>,
    max_concurrency: usize,
}

impl BatchRenderer {
    /// Create a new batch renderer
    ///
    /// # Arguments
    /// * `templates_root` - Root directory for template files
    ///
    /// # Default Settings
    /// - Max concurrency: num_cpus::get() (scales with available cores)
    /// - Engine: Default TemplateEngine with caching
    /// - Resolver: Default TemplateResolver with caching
    pub fn new<P: AsRef<Path>>(templates_root: P) -> Self {
        Self {
            engine: Arc::new(TemplateEngine::new()),
            resolver: Arc::new(TemplateResolver::new(templates_root)),
            max_concurrency: num_cpus::get(),
        }
    }

    /// Set maximum concurrent rendering tasks
    ///
    /// # Performance Tuning
    /// - Lower value: Less memory, more sequential
    /// - Higher value: More parallelism, but diminishing returns beyond CPU cores
    /// - Default: num_cpus::get() (optimal for most workloads)
    pub fn with_max_concurrency(mut self, max: usize) -> Self {
        self.max_concurrency = max.max(1); // At least 1
        self
    }

    /// Set custom engine (useful for sharing cache across renderers)
    pub fn with_engine(mut self, engine: Arc<TemplateEngine>) -> Self {
        self.engine = engine;
        self
    }

    /// Set custom resolver (useful for sharing cache across renderers)
    pub fn with_resolver(mut self, resolver: Arc<TemplateResolver>) -> Self {
        self.resolver = resolver;
        self
    }

    /// Render multiple templates in parallel
    ///
    /// # Performance
    /// - Spawns up to `max_concurrency` tasks simultaneously
    /// - Uses JoinSet for structured concurrency (bounded parallelism)
    /// - Each task: resolve → render → write (fully independent)
    ///
    /// # Example Speedup
    /// - 100 templates, 10 cores, 100μs per template
    /// - Sequential: 100 × 100μs = 10,000μs = 10ms
    /// - Parallel: 100 ÷ 10 × 100μs = 1,000μs = 1ms (10x faster)
    pub async fn render_batch<T>(
        &self,
        requests: Vec<RenderRequest<T>>,
    ) -> TemplateResult<BatchRenderResult>
    where
        T: Serialize + Send + Sync + 'static,
    {
        let start = std::time::Instant::now();
        let total_requests = requests.len();

        let mut join_set = JoinSet::new();
        let mut succeeded = 0;
        let mut failed = 0;
        let mut errors = Vec::new();

        // Spawn tasks with bounded concurrency
        for request in requests {
            let engine = Arc::clone(&self.engine);
            let resolver = Arc::clone(&self.resolver);

            // Wait if we've reached max concurrency
            if join_set.len() >= self.max_concurrency {
                if let Some(result) = join_set.join_next().await {
                    match result {
                        Ok(Ok(_)) => succeeded += 1,
                        Ok(Err((template, err))) => {
                            failed += 1;
                            errors.push((template, err));
                        }
                        Err(join_err) => {
                            failed += 1;
                            errors.push((
                                "unknown".to_string(),
                                TemplateError::RenderError {
                                    template: "unknown".to_string(),
                                    message: format!("Task panic: {}", join_err),
                                },
                            ));
                        }
                    }
                }
            }

            // Spawn new task
            join_set.spawn(async move { Self::render_single(engine, resolver, request).await });
        }

        // Wait for remaining tasks
        while let Some(result) = join_set.join_next().await {
            match result {
                Ok(Ok(_)) => succeeded += 1,
                Ok(Err((template, err))) => {
                    failed += 1;
                    errors.push((template, err));
                }
                Err(join_err) => {
                    failed += 1;
                    errors.push((
                        "unknown".to_string(),
                        TemplateError::RenderError {
                            template: "unknown".to_string(),
                            message: format!("Task panic: {}", join_err),
                        },
                    ));
                }
            }
        }

        let duration = start.elapsed();

        // Sanity check
        assert_eq!(
            succeeded + failed,
            total_requests,
            "Rendered count mismatch"
        );

        Ok(BatchRenderResult {
            succeeded,
            failed,
            errors,
            duration,
        })
    }

    /// Render a single template (internal helper)
    async fn render_single<T>(
        engine: Arc<TemplateEngine>,
        resolver: Arc<TemplateResolver>,
        request: RenderRequest<T>,
    ) -> Result<(), (String, TemplateError)>
    where
        T: Serialize + Sync,
    {
        // Resolve template path
        let template_path = resolver
            .resolve(&request.template)
            .await
            .map_err(|err| (request.template.clone(), err))?;

        // Render template
        let rendered = engine
            .render_from_file(&template_path, &request.data)
            .await
            .map_err(|err| (request.template.clone(), err))?;

        // Create output directory if needed
        if let Some(parent) = request.output.parent() {
            tokio::fs::create_dir_all(parent).await.map_err(|err| {
                (
                    request.template.clone(),
                    TemplateError::ReadError {
                        path: parent.display().to_string(),
                        source: err,
                    },
                )
            })?;
        }

        // Write output file
        tokio::fs::write(&request.output, rendered)
            .await
            .map_err(|err| {
                (
                    request.template.clone(),
                    TemplateError::ReadError {
                        path: request.output.display().to_string(),
                        source: err,
                    },
                )
            })?;

        Ok(())
    }
}
