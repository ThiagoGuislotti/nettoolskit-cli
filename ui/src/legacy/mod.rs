/// Legacy UI implementation (pre-TUI modernization)
///
/// This module contains the original printf-style UI implementation
/// that will be maintained for backward compatibility. It provides
/// basic terminal output without the advanced features of the modern TUI.

pub mod display;
pub mod palette;
pub mod terminal;

pub use display::*;
pub use palette::*;
pub use terminal::*;