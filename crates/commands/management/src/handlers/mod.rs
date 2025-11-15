//! Command handlers
//!
//! This module contains implementations for each CLI command.
//! Each handler is responsible for the business logic of a specific command.

pub mod list;

// Re-export commonly used handler functions
pub use list::{discover_manifests, display_manifests, ManifestInfo};