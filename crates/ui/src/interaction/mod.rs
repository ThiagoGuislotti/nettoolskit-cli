//! User interaction components
//!
//! Provides interactive UI elements like command palettes, prompts, and terminal control.

/// Interactive file picker for path discovery and selection.
pub mod file_picker;
/// Interactive history viewer with filtering and pagination.
pub mod history_viewer;
/// Command palette for interactive selection.
pub mod palette;
pub mod prompt;
/// Runtime status bar for interactive session feedback.
pub mod status_bar;
/// Terminal control and lifecycle management.
pub mod terminal;
