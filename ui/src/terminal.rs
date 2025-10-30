use crossterm::terminal::{Clear, ClearType};
use crossterm::{cursor, execute};
use std::io::{self, Write};

/// Clear the terminal screen and move cursor to top-left position.
///
/// This function performs a complete terminal reset, useful for
/// starting with a clean display state.
///
/// # Returns
///
/// Returns `Ok(())` if the terminal is cleared successfully,
/// or an `io::Error` if terminal operations fail.
///
/// # Examples
///
/// ```
/// use nettoolskit_ui::clear_terminal;
/// clear_terminal().expect("Failed to clear terminal");
/// ```
pub fn clear_terminal() -> io::Result<()> {
    execute!(
        io::stdout(),
        Clear(ClearType::All),
        cursor::MoveTo(0, 0)
    )?;
    io::stdout().flush()
}