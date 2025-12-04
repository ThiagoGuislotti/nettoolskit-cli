//! Command implementations for NetToolsKit CLI
//!
//! This crate aggregates individual command implementations:
//! - **help**: Help and manifest discovery
//! - **manifest**: Manifest-driven code generation
//! - **translate**: Code translation between languages
//!
//! Command orchestration is provided by the `nettoolskit-orchestrator` crate.
//!
//! # Architecture
//!
//! Each command is in its own sub-crate with:
//! - handlers/: Command execution logic
//! - models/: Command-specific data structures
//! - lib.rs: Public API

// Re-export command implementations for convenient access
pub use nettoolskit_help;
pub use nettoolskit_manifest;
pub use nettoolskit_translate;
