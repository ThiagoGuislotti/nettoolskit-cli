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

/// Trait for enums that can provide interactive menu functionality
///
/// This trait combines MenuEntry with iteration capabilities to enable
/// generic menu rendering. Any enum that implements this trait can be
/// used with the generic menu system.
///
/// # Requirements
/// - Must implement `MenuEntry` for label/description
/// - Must be `Clone` for menu interaction
/// - Must be `Display` for rendering
pub trait MenuProvider: MenuEntry + Clone + std::fmt::Display {
    /// Generate formatted menu items for display
    ///
    /// Returns a vector of strings in the format "label - description"
    fn menu_items() -> Vec<String>
    where
        Self: Sized;

    /// Get all enum variants as a vector
    fn all_variants() -> Vec<Self>
    where
        Self: Sized;
}

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
// Re-export commonly used items
pub use features::Features;
