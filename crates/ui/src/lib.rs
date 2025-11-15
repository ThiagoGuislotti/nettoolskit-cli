//! Terminal UI components for NetToolsKit CLI
//!
//! Provides terminal interface components including:
//! - Command palette for interactive command discovery
//! - Terminal layout management and logging
//! - ASCII art and branding display
//! - Color and style formatting utilities

pub mod display;
pub mod palette;
pub mod prompt;
pub mod style;
pub mod terminal;
pub mod writer;

// Re-export commonly used items
pub use display::*;
pub use palette::*;
pub use prompt::*;
pub use style::*;
pub use terminal::*;
pub use writer::UiWriter;
