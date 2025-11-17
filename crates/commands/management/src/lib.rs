//! Command management and orchestration for NetToolsKit CLI
//!
//! This crate provides centralized command management including:
//! - Command definitions and enumeration (definitions)
//! - Command processor for execution dispatch (processor)
//! - Async executor for long-running operations (executor)
//! - Command handlers implementation (handlers)
//! - I/O utilities for output formatting (io)
//! - Interactive submenus for complex commands (submenu)

pub mod definitions;
pub mod executor;
pub mod handlers;
pub mod io;
pub mod processor;
pub mod submenu;

// Re-export commonly used types
pub use definitions::{menu_entries, palette_entries, Command, ExitStatus};
pub use executor::{
    AsyncCommandExecutor, CommandHandle, CommandProgress, CommandResult, ProgressSender,
};
pub use handlers::{discover_manifests, display_manifests, ManifestInfo};
pub use processor::{process_command, process_text};