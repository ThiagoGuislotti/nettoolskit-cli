//! Command handlers
//!
//! This module contains implementations for each CLI command.
//! Each handler is responsible for the business logic of a specific command.

pub mod check;
pub mod help;

// Re-export commonly used handler functions
pub use check::{check_file, display_validation_result, ValidationResult};
pub use help::{discover_manifests, display_manifests, ManifestInfo};