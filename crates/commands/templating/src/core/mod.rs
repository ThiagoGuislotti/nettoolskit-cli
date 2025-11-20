//! Core templating types and utilities.
//!
//! This module contains fundamental types:
//! - `TemplateError`: Error handling for template operations
//! - `common`: Common types and utilities
//! - `helpers`: Helper functions for templates

pub mod common;
pub mod error;
pub mod helpers;

pub use error::{TemplateError, TemplateResult};