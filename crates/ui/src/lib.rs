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
pub mod interaction;
pub mod rendering;

// Re-export commonly used items
pub use core::capabilities::{
    self, capabilities, maybe_gray, pick_char, pick_str, set_color_override, set_unicode_override,
    ColorLevel, TerminalCaps,
};
pub use core::colors::*;
pub use core::style;
// Re-export components
pub use interaction::clipboard::{copy_to_clipboard, paste_from_clipboard};
pub use interaction::file_picker::FilePicker;
pub use interaction::history_viewer::HistoryViewer;
pub use interaction::notification::emit_desktop_attention_notification;
pub use interaction::palette::CommandPalette;
pub use interaction::prompt::*;
pub use interaction::status_bar::{StatusBar, StatusBarMode, StatusNotificationLevel};
pub use interaction::terminal::*;
pub use rendering::components::{
    format_menu_item, render_box, render_command, render_enum_menu, render_interactive_menu,
    render_menu_instructions, render_section_title, BoxConfig, EnumMenuConfig, MenuConfig,
};
pub use rendering::markdown::render_markdown;
pub use rendering::writer::UiWriter;
