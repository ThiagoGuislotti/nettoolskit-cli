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
//!
//! # Module Organization
//!
//! - `core`: Foundational types (colors, style utilities)
//! - `rendering`: UI components and output writing
//! - `interaction`: User interaction (palette, prompt, terminal control)

pub mod core;
pub mod rendering;
pub mod interaction;

// Re-export commonly used items
pub use core::colors::*;
pub use core::style;
pub use rendering::components::{self, BoxConfig, MenuConfig, render_box, render_interactive_menu};
pub use rendering::writer::UiWriter;
pub use interaction::palette::CommandPalette;
pub use interaction::prompt::*;
pub use interaction::terminal::*;
