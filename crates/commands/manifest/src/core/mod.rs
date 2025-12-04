//! Core manifest types and definitions.
//!
//! This module contains fundamental types used throughout the manifest crate:
//! - `ManifestError`: Error types for manifest operations
//! - `ManifestModels`: Data models for manifests, resources, and outputs

pub mod error;
pub mod models;

pub use error::{ManifestError, ManifestResult};
pub use models::*;