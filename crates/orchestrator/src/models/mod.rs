//! Command management models
//!
//! This module organizes all models used for command management

pub mod main_action;

// Re-export canonical names only
pub use main_action::{get_main_action, MainAction};
pub use nettoolskit_core::ExitStatus;
