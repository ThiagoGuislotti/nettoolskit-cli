//! Execution Module
//!
//! Command execution and processing logic.

/// Async command executor with progress tracking.
pub mod executor;
pub mod processor;

// Re-export commonly used types
pub use executor::{
    AsyncCommandExecutor, CommandHandle, CommandProgress, CommandResult, ProgressSender,
};
pub use processor::{process_command, process_text};
