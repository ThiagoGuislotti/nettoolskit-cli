//! UI components module
//!
//! This module provides reusable UI components such as boxes and menus.

pub mod box_component;
pub mod menu;
pub mod helpers;

pub use box_component::{BoxConfig, render_box};
pub use menu::{MenuConfig, render_interactive_menu};
pub use helpers::{render_command_header, render_menu_instructions, render_section_title, format_menu_item};
