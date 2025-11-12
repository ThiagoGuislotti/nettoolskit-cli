//! Core types and utilities for `NetToolsKit` CLI
//!
//! This crate provides fundamental types, traits, and utilities
//! used across the `NetToolsKit` CLI application.

/// Core error types for the application
pub type Result<T> = anyhow::Result<T>;

/// Feature detection and configuration for opt-in TUI improvements
pub mod features;

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

/// Command definitions for the interactive palette
pub mod commands {
    /// Constants for the command palette
    pub const COMMANDS: &[(&str, &str)] = &[
        ("/list", "List available templates"),
        ("/check", "Validate a manifest or template"),
        ("/render", "Render a template preview"),
        ("/new", "Create a project from a template"),
        ("/apply", "Apply a manifest to an existing solution"),
        ("/quit", "Exit NetToolsKit CLI"),
    ];
}

// Re-export commonly used items
pub use features::Features;
