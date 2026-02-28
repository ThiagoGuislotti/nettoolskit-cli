//! UI components module
//!
//! This module provides reusable UI components such as boxes and menus.

pub mod box_component;
pub mod enum_menu;
pub mod helpers;
pub mod menu;

// Re-export commonly used items
pub use box_component::{render_box, BoxConfig};
pub use enum_menu::{render_enum_menu, EnumMenuConfig};
pub use helpers::{
    format_menu_item, render_command, render_menu_instructions, render_section_title,
};
pub use menu::{render_interactive_menu, MenuConfig};
