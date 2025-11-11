///! Manifest orchestration feature for NetToolsKit CLI
///!
///! This crate handles manifest parsing, validation, and execution for
///! code generation workflows. It orchestrates the templating engine to
///! apply manifests to existing projects.
///!
///! # Architecture
///!
///! ```text
///! manifest/
///! ├── models/       # Domain models (ManifestDocument, Project, etc.)
///! ├── parser/       # YAML parsing and validation
///! ├── executor/     # Manifest execution logic
///! ├── resolver/     # Path and template resolution
///! └── error/        # Error types
///! ```
///!
///! # Manifest Types
///!
///! - **Domain**: DDD artifacts (entities, value objects, aggregates)
///! - **Feature**: Feature-based  organization
///! - **Layer**: Layer-based architecture (controllers, services, repositories)
///! - **Artifact**: Individual code artifacts
///!
///! # Usage
///!
///! ```rust,no_run
///! use nettoolskit_manifest::{ManifestExecutor, ExecutionConfig};
///! use std::path::PathBuf;
///!
///! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
///! let executor = ManifestExecutor::new();
///! let config = ExecutionConfig {
///!     manifest_path: PathBuf::from("ntk-manifest.yml"),
///!     output_root: PathBuf::from("target/output"),
///!     dry_run: false,
///! };
///!
///! let summary = executor.execute(config).await?;
///! println!("Created: {} files", summary.created.len());
///! # Ok(())
///! # }
///! ```

mod error;
mod executor;
mod models;
mod parser;
mod rendering;

pub use error::{ManifestError, ManifestResult};
pub use models::{
    ManifestDocument, ManifestKind, ManifestMeta, ManifestConventions,
    ManifestSolution, ManifestProject, ManifestProjectKind, ManifestPolicy,
    ManifestGuards, ArtifactKind, ManifestContext, ExecutionSummary,
};
pub use parser::ManifestParser;
pub use executor::{ManifestExecutor, ExecutionConfig};

// Re-export TemplateResolver from templating (no duplication)
pub use nettoolskit_templating::TemplateResolver;