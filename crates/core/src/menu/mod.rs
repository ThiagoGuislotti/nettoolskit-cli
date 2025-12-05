//! Menu system traits and utilities
//!
//! This module provides traits for building interactive menu systems
//! with consistent behavior across different command types.

mod command_entry;
mod menu_entry;
mod menu_provider;

pub use command_entry::CommandEntry;
pub use menu_entry::MenuEntry;
pub use menu_provider::MenuProvider;
