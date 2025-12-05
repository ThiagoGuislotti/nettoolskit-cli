//! Core types and utilities for `NetToolsKit` CLI
//!
//! This crate provides fundamental types, traits, and utilities
//! used across the `NetToolsKit` CLI application.

/// Core error types for the application
pub type Result<T> = anyhow::Result<T>;

/// Menu system traits and utilities
pub mod menu;

/// Exit status codes for command execution
pub mod exit_status;

/// Feature detection and configuration for opt-in TUI improvements
pub mod features;

/// String manipulation utilities
#[path = "string-utils/lib.rs"]
pub mod string_utils;

/// Async utilities (timeout, cancellation)
#[path = "async-utils/lib.rs"]
pub mod async_utils;

/// File search and filtering utilities
#[path = "file-search/lib.rs"]
pub mod file_search;

/// Path and directory utilities
#[path = "path-utils/lib.rs"]
pub mod path_utils;

// Re-export commonly used types
pub use exit_status::ExitStatus;
pub use features::Features;
pub use menu::{CommandEntry, MenuEntry, MenuProvider};
