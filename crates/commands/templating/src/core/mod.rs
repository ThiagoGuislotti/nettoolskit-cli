//! Core templating types and utilities.
//!
//! This module contains fundamental types:
//! - `TemplateError`: Error handling for template operations
//! - `helpers`: Helper functions for templates

pub mod error;
pub mod helpers;

pub use error::{TemplateError, TemplateResult};