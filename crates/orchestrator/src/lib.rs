//! Command orchestration for NetToolsKit CLI
//!
//! This crate provides the orchestration layer between the CLI interface
//! and command implementations, including:
//! - Command models and menu system
//! - Async command execution with progress tracking
//! - Command processor for dispatch and routing

pub mod models;
pub mod execution;

// Re-export commonly used types
pub use models::{MainAction, Command, ExitStatus, get_main_action, get_command};
pub use execution::{
    executor::{AsyncCommandExecutor, CommandHandle, CommandProgress, CommandResult, ProgressSender},
    processor::{process_command, process_text},
};

// Backward compatibility modules
pub mod definitions {
    pub use crate::models::*;
}
