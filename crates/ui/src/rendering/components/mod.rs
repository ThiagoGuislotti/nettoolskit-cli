//! UI components module
//!
//! This module provides reusable UI components such as boxes and menus.

pub mod box_component;
pub mod enum_menu;
pub mod helpers;
pub mod menu;

// Re-export commonly used items
pub use box_component::{BoxConfig, render_box};
pub use enum_menu::{EnumMenuConfig, render_enum_menu};
pub use helpers::{render_command, render_menu_instructions, render_section_title, format_menu_item};
pub use menu::{MenuConfig, render_interactive_menu};
