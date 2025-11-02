// Legacy UI implementation (backward compatible)
pub mod legacy;

// Modern TUI implementation (opt-in via feature flags)
#[cfg(feature = "modern-tui")]
pub mod modern;

// Re-export legacy UI as default for backward compatibility
pub use legacy::{display, palette, terminal};
pub use legacy::display::*;
pub use legacy::palette::*;
pub use legacy::terminal::*;

// Re-export modern UI when feature is enabled
#[cfg(feature = "modern-tui")]
pub use modern::{App, Tui};
