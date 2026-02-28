//! Core templating types and utilities.
//!
//! This module contains fundamental types:
//! - `TemplateError`: Error handling for template operations
//! - `helpers`: Helper functions for templates

/// Error types for template operations.
pub mod error;
/// Handlebars helper functions for case conversion.
pub mod helpers;

/// Re-exported error and result types.
pub use error::{TemplateError, TemplateResult};
