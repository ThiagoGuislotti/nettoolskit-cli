//! Template rendering engine and utilities.
//!
//! This module handles template rendering:
//! - `engine`: Core template engine with Handlebars
//! - `batch`: Batch/parallel rendering
//! - `resolver`: Template path resolution

pub mod batch;
pub mod engine;
pub mod resolver;

pub use batch::{BatchRenderResult, BatchRenderer, RenderRequest};
pub use engine::TemplateEngine;
pub use resolver::TemplateResolver;