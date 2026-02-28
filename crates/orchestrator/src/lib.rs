//! Command orchestration for NetToolsKit CLI
//!
//! This crate provides the orchestration layer between the CLI interface
//! and command implementations, including:
//! - Command models and menu system
//! - Async command execution with progress tracking
//! - Command processor for dispatch and routing

pub mod execution;
pub mod models;

// Re-export commonly used types
pub use execution::{
    executor::{
        AsyncCommandExecutor, CommandHandle, CommandProgress, CommandResult, ProgressSender,
    },
    processor::{process_command, process_text},
};
pub use models::{get_command, get_main_action, Command, ExitStatus, MainAction};

// Backward compatibility modules
/// Backward-compatible re-export of model types.
pub mod definitions {
    pub use crate::models::*;
}
