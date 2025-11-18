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
mod error;
mod executor;
mod menu;
mod rendering;

// Public modules for testing
pub mod definitions;
pub mod files;
pub mod models;
pub mod parser;
pub mod tasks;

pub use definitions::{ManifestAction, get_action, menu_entries, palette_entries};
pub use error::{ManifestError, ManifestResult};
pub use menu::show_menu;
pub use executor::{ExecutionConfig, ManifestExecutor};
pub use models::{
    ArtifactKind, ExecutionSummary, FileChange, ManifestContext, ManifestConventions,
    ManifestDocument, ManifestGuards, ManifestKind, ManifestMeta, ManifestPolicy, ManifestProject,
    ManifestProjectKind, ManifestSolution,
};
pub use parser::ManifestParser;

// Re-export TemplateResolver from templating (no duplication)
pub use nettoolskit_templating::TemplateResolver;
