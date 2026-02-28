//! Manifest orchestration feature for NetToolsKit CLI
//!
//! This crate handles manifest parsing, validation, and execution for
//! code generation workflows. It orchestrates the templating engine to
//! apply manifests to existing projects.
//!
//! # Architecture
//!
//! ```text
//! manifest/
//! ├── models/       # Domain models (ManifestDocument, Project, etc.)
//! ├── parser/       # YAML parsing and validation
//! ├── executor/     # Manifest execution logic
//! ├── resolver/     # Path and template resolution
//! └── error/        # Error types
//! ```
//!
//! # Manifest Types
//!
//! - **Domain**: DDD artifacts (entities, value objects, aggregates)
//! - **Feature**: Feature-based  organization
//! - **Layer**: Layer-based architecture (controllers, services, repositories)
//! - **Artifact**: Individual code artifacts
//!
//! # Usage
//!
//! ```rust,no_run
//! use nettoolskit_manifest::{ManifestExecutor, ExecutionConfig};
//! use std::path::PathBuf;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let executor = ManifestExecutor::new();
//! let config = ExecutionConfig {
//!     manifest_path: PathBuf::from("ntk-manifest.yml"),
//!     output_root: PathBuf::from("target/output"),
//!     dry_run: false,
//! };
//!
//! let summary = executor.execute(config).await?;
//! println!("Created: {} files", summary.created.len());
//! # Ok(())
//! # }
//! ```
// Organized module structure
pub mod core;
pub mod execution;
pub mod handlers;
pub mod models;
pub mod parsing;
/// Task generation definitions.
pub mod tasks;
pub mod ui;

// Backward compatibility aliases
/// Backward-compatible re-export of the parsing module.
pub mod parser {
    pub use crate::parsing::*;
}

// Public API — externally consumed types
pub use core::{ManifestError, ManifestResult};
pub use execution::{ExecutionConfig, ManifestExecutor};
pub use handlers::execute_apply;
pub use models::ManifestAction;
pub use parsing::ManifestParser;
pub use ui::show_menu;
