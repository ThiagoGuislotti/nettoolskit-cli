//! Help command for NetToolsKit CLI
//!
//! This crate provides help and discovery functionality for the CLI,
//! including manifest file discovery and display.

pub mod handlers;
pub mod models;

// Re-export commonly used types
pub use handlers::{discover_manifests, display_manifests};
pub use models::ManifestInfo;
