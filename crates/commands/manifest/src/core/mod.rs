//! Core manifest types and definitions.
//!
//! This module contains fundamental types used throughout the manifest crate:
//! - `ManifestAction`: Enum defining check/render/apply subcommands
//! - `ManifestError`: Error types for manifest operations
//! - `ManifestModels`: Data models for manifests, resources, and outputs

pub mod definitions;
pub mod error;
pub mod models;

pub use definitions::ManifestAction;
pub use error::{ManifestError, ManifestResult};
pub use models::*;