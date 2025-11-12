mod batch;
/// High-performance async template rendering for NetToolsKit CLI
///
/// This crate provides a robust, async-first templating engine with:
/// - **Strategy Pattern**: Language-specific path resolution
/// - **Factory Pattern**: On-demand strategy instantiation
/// - **Async/Await**: Non-blocking I/O with tokio
/// - **Smart Caching**: DashMap for O(1) lookups (100-10,000x faster)
/// - **Parallelism**: Batch rendering with bounded concurrency
///
/// # Architecture
///
/// - Pure infrastructure layer (no business logic)
/// - Zero coupling to domain concerns
/// - Thread-safe: Arc + DashMap for concurrent access
/// - Extensible: Easy to add new languages via LanguageStrategy
///
/// # Performance
///
/// - **Cache hit**: ~10-50ns (DashMap lookup)
/// - **Cache miss**: ~100μs-1ms (I/O + compile + render)
/// - **Parallel rendering**: Linear speedup with CPU cores
///
/// # Example: Simple Rendering
///
/// ```no_run
/// use nettoolskit_templating::{TemplateEngine, TemplateResolver};
/// use serde_json::json;
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let resolver = TemplateResolver::new("templates");
/// let template_path = resolver.resolve("dotnet/Domain/Entity.cs.hbs").await?;
///
/// let engine = TemplateEngine::new();
/// let data = json!({"name": "User"});
/// let rendered = engine.render_from_file(&template_path, &data).await?;
/// # Ok(())
/// # }
/// ```
///
/// # Example: Batch Rendering (Parallel)
///
/// ```no_run
/// use nettoolskit_templating::{BatchRenderer, RenderRequest};
/// use serde_json::json;
/// use std::path::PathBuf;
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let renderer = BatchRenderer::new("templates");
/// let requests = vec![
///     RenderRequest {
///         template: "dotnet/Domain/Entity.cs.hbs".to_string(),
///         data: json!({"name": "User"}),
///         output: PathBuf::from("output/User.cs"),
///     },
/// ];
/// let result = renderer.render_batch(requests).await?;
/// # Ok(())
/// # }
/// ```
mod engine;
mod error;
mod factory;
mod helpers;
mod resolver;
mod strategy;

pub use batch::{BatchRenderResult, BatchRenderer, RenderRequest};
pub use engine::TemplateEngine;
pub use error::{TemplateError, TemplateResult};
pub use factory::{Language, LanguageStrategyFactory};
pub use resolver::TemplateResolver;
pub use strategy::{
    ClojureStrategy, DotNetStrategy, GoStrategy, JavaStrategy, LanguageConventions,
    LanguageStrategy, PythonStrategy, RustStrategy,
};
