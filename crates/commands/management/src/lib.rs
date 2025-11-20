//! Command management and orchestration for NetToolsKit CLI
//!
//! This crate provides centralized command management including:
//! - Command definitions and enumeration (core/definitions)
//! - Command processor for execution dispatch (execution/processor)
//! - Async executor for long-running operations (execution/executor)
//! - Command handlers implementation (handlers)
//! - I/O utilities for output formatting (io)

pub mod core;
pub mod execution;
pub mod handlers;
pub mod io;

// Re-export commonly used types
pub use core::definitions::{menu_entries, palette_entries, Command};
pub use core::error::{CommandError, Result};
pub use execution::executor::{
    AsyncCommandExecutor, CommandHandle, CommandProgress, CommandResult, ProgressSender,
};
pub use execution::processor::{process_command, process_text};
pub use handlers::{discover_manifests, display_manifests, ManifestInfo};

// Backward compatibility aliases
pub use core::definitions;
pub use execution::{executor, processor};