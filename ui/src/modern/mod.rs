/// Modern TUI implementation using Ratatui
///
/// This module provides event-driven architecture and performance improvements
/// from Codex, while maintaining the exact same visual layout as legacy UI.
///
/// Key improvements:
/// - Phase 1.2: Event-driven input (16ms poll instead of 50ms)
/// - Phase 1.3: Event stream support (zero CPU when idle)
/// - Better event handling with Ratatui's event stream
/// - Maintains 100% visual compatibility with legacy UI
/// - No alternate screen - uses normal terminal flow

pub mod app;
pub mod events;
pub mod tui;
pub mod widgets;

pub use app::App;
pub use events::{handle_events, EventResult};
pub use tui::Tui;
pub use widgets::render_ui;

// Phase 1.3: Event stream support
#[cfg(feature = "modern-tui")]
pub use events::handle_events_stream;
#[cfg(feature = "modern-tui")]
pub use crossterm::event::EventStream;