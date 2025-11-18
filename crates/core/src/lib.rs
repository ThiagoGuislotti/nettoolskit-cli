//! Core types and utilities for `NetToolsKit` CLI
//!
//! This crate provides fundamental types, traits, and utilities
//! used across the `NetToolsKit` CLI application.

/// Core error types for the application
pub type Result<T> = anyhow::Result<T>;

/// Trait for items that can be displayed in UI menus
///
/// This trait defines the interface for items that can be shown in menus,
/// allowing the UI layer to remain decoupled from specific domain types.
pub trait MenuEntry {
    /// Get the label/identifier for this menu entry (e.g., "/list")
    fn label(&self) -> &str;

    /// Get the description for this menu entry
    fn description(&self) -> &str;
}

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

/// Configuration types and utilities
pub mod config {
    use serde::{Deserialize, Serialize};

    /// Application configuration structure
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Config {
        /// Application name
        pub name: String,
        /// Application version
        pub version: String,
    }

    impl Default for Config {
        fn default() -> Self {
            Self {
                name: "NetToolsKit CLI".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            }
        }
    }
}

// Re-export commonly used items
pub use features::Features;
