//! Manifest execution and rendering.
//!
//! This module handles the execution of manifest operations:
//! - `executor`: Main manifest execution logic
//! - `rendering`: Template rendering and file generation
//! - `files`: File system operations for manifests

/// Main manifest execution logic.
pub mod executor;
pub mod files;
/// Template rendering and file generation.
pub mod rendering;

pub use executor::{ExecutionConfig, ManifestExecutor};
pub use files::{ensure_directory, execute_plan};
pub use rendering::{build_project_stub, build_solution_stub, normalize_line_endings};
