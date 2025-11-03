//! Terminal UI components for NetToolsKit CLI
//!
//! Provides terminal interface components including:
//! - Command palette for interactive command discovery
//! - Terminal layout management and logging
//! - ASCII art and branding display

mod ui;

// Re-export all components
pub use ui::display;
pub use ui::display::*;
pub use ui::palette;
pub use ui::palette::*;
pub use ui::prompt;
pub use ui::prompt::*;
pub use ui::terminal;
pub use ui::terminal::*;
