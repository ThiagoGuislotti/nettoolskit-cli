//! Centralized prompt formatting and rendering
//!
//! This module provides a consistent interface for displaying the CLI prompt
//! across all input contexts, ensuring uniform styling and behavior.

use owo_colors::OwoColorize;
use std::io::{self, Write};

/// The prompt symbol used throughout the CLI
const PROMPT_SYMBOL: &str = "> ";

/// Render the prompt with consistent white styling
///
/// # Examples
///
/// ```no_run
/// use nettoolskit_ui::render_prompt;
/// render_prompt().expect("Failed to render prompt");
/// ```
pub fn render_prompt() -> io::Result<()> {
    // Reset terminal attributes first to prevent color bleeding from previous commands
    print!("\x1b[0m{}", PROMPT_SYMBOL.white());
    io::stdout().flush()
}

/// Render the prompt with a command after it (used for autocompletion)
///
/// This function clears the current line and renders the prompt followed by
/// the specified command text.
///
/// # Arguments
///
/// * `cmd` - The command text to display after the prompt
///
/// # Examples
///
/// ```no_run
/// use nettoolskit_ui::render_prompt_with_command;
/// render_prompt_with_command("/help").expect("Failed to render");
/// ```
pub fn render_prompt_with_command(cmd: &str) -> io::Result<()> {
    print!("\r\x1b[K{} {}", PROMPT_SYMBOL.white(), cmd);
    io::stdout().flush()
}

/// Get the formatted prompt string (for display purposes)
///
/// Returns the prompt symbol with white color formatting applied.
///
/// # Examples
///
/// ```no_run
/// use nettoolskit_ui::get_prompt_string;
/// let prompt = get_prompt_string();
/// println!("{}", prompt);
/// ```
pub fn get_prompt_string() -> String {
    format!("{}", PROMPT_SYMBOL.white())
}

/// Get the raw prompt symbol without formatting
///
/// # Examples
///
/// ```no_run
/// use nettoolskit_ui::get_prompt_symbol;
/// assert_eq!(get_prompt_symbol(), "> ");
/// ```
pub fn get_prompt_symbol() -> &'static str {
    PROMPT_SYMBOL
}
