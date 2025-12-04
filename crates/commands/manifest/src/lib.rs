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
pub mod tasks;
pub mod ui;

// Backward compatibility aliases
pub mod parser {
    pub use crate::parsing::*;
}

pub mod files {
    pub use crate::execution::files::*;
}

pub mod definitions {
    pub use crate::models::manifest_action::*;
}

// Re-export core types
pub use models::ManifestAction;
pub use core::{ManifestError, ManifestResult};

// Re-export domain models from core
pub use core::models::{
    ArtifactKind, ExecutionSummary, FileChange, FileChangeKind, ManifestAggregate,
    ManifestContext, ManifestConventions, ManifestDomainEvent, ManifestEntity, ManifestEnum,
    ManifestEnumValue, ManifestField, ManifestKind, ManifestPolicy, ManifestProjectKind,
    ManifestRepository, ManifestUseCase, ManifestValueObject, RenderTask, TemplateMapping,
};

// Re-export execution types
pub use execution::{
    build_project_payload, build_project_stub, build_solution_stub, ensure_directory,
    execute_plan, normalize_line_endings, ExecutionConfig, ManifestExecutor,
};

// Re-export parsing types
pub use parsing::ManifestParser;

// Re-export handlers
pub use handlers::{execute_apply, check_file, display_validation_result, ValidationError, ValidationResult};

// Re-export ui types
pub use ui::show_menu;

// Re-export TemplateResolver from templating (no duplication)
pub use nettoolskit_templating::TemplateResolver;
