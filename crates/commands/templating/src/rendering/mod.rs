//! Template rendering engine and utilities.
//!
//! This module handles template rendering:
//! - `engine`: Core template engine with Handlebars
//! - `batch`: Batch/parallel rendering
//! - `resolver`: Template path resolution

/// Batch / parallel template rendering.
pub mod batch;
/// Core Handlebars-backed template engine.
pub mod engine;
/// Template path resolution with caching.
pub mod resolver;

/// Batch rendering types.
pub use batch::{BatchRenderResult, BatchRenderer, RenderRequest};
/// The primary template engine.
pub use engine::TemplateEngine;
/// Template file resolver.
pub use resolver::TemplateResolver;
