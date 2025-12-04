//! Command management models
//!
//! This module organizes all models used for command management

pub mod main_action;

// Re-export with both new and old names for compatibility
pub use main_action::{MainAction, get_main_action};

// Backward compatibility aliases
pub use main_action::MainAction as Command;
pub use main_action::get_main_action as get_command;
pub use nettoolskit_core::ExitStatus;
