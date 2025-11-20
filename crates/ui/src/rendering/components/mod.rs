//! Reusable UI components for terminal interfaces
//!
//! This module provides building blocks for creating consistent terminal UIs:
//! - `box_component`: Configurable bordered boxes for displaying information
//! - `menu`: Interactive menus with keyboard navigation
//!
//! Components follow the Single Responsibility Principle and can be composed
//! to create complex interfaces while maintaining separation of concerns.

pub mod box_component;
pub mod menu;

pub use box_component::{BoxConfig, render_box};
pub use menu::{MenuConfig, render_interactive_menu};
