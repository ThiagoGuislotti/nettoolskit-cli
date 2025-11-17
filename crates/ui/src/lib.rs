//! Terminal UI components for NetToolsKit CLI
//!
//! Provides reusable terminal interface components including:
//! - Command palette for interactive command discovery
//! - Terminal layout management and logging
//! - Color and style formatting utilities
//! - Reusable UI components (boxes, menus)
//!
//! # Architecture
//!
//! This crate provides generic, reusable UI components.
//! Application-specific display logic (logos, branding) should live in the CLI crate.

pub mod colors;
pub mod components;
pub mod palette;
pub mod prompt;
pub mod style;
pub mod terminal;
pub mod writer;

// Re-export commonly used items
pub use colors::*;
pub use components::*;
pub use palette::*;
pub use prompt::*;
pub use style::*;
pub use terminal::*;
pub use writer::UiWriter;
