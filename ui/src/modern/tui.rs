/// TUI backend and rendering infrastructure
///
/// This module provides the terminal backend setup and rendering loop
/// for the modern Ratatui-based interface, while maintaining the visual
/// appearance of the legacy UI (no alternate screen, normal terminal flow).

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use std::io;

/// TUI backend wrapper with Ratatui terminal
pub struct Tui {
    /// Ratatui terminal instance
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
    /// Whether the TUI is currently active
    active: bool,
}

impl Tui {
    /// Create a new TUI instance with Ratatui backend
    pub fn new() -> io::Result<Self> {
        let backend = CrosstermBackend::new(io::stdout());
        let terminal = Terminal::new(backend)?;

        Ok(Self {
            terminal,
            active: false
        })
    }

    /// Enter the TUI (setup terminal)
    /// NOTE: Uses raw mode but NOT alternate screen to maintain legacy appearance
    pub fn enter(&mut self) -> io::Result<()> {
        if self.active {
            return Ok(());
        }

        enable_raw_mode()?;
        // NOTE: NO alternate screen! We want to stay in the normal terminal
        // like the legacy UI does. Only enable mouse capture for potential
        // future interactivity.
        execute!(io::stdout(), EnableMouseCapture)?;

        self.active = true;
        Ok(())
    }

    /// Exit the TUI (restore terminal)
    pub fn exit(&mut self) -> io::Result<()> {
        if !self.active {
            return Ok(());
        }

        disable_raw_mode()?;
        execute!(io::stdout(), DisableMouseCapture)?;
        // NOTE: No LeaveAlternateScreen since we never entered it

        self.active = false;
        Ok(())
    }

    /// Check if TUI is active
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Draw a frame with the given render function
    pub fn draw<F>(&mut self, render_fn: F) -> io::Result<()>
    where
        F: FnOnce(&mut ratatui::Frame),
    {
        self.terminal.draw(render_fn)?;
        Ok(())
    }

    /// Get terminal size
    pub fn size(&self) -> io::Result<(u16, u16)> {
        let size = self.terminal.size()?;
        Ok((size.width, size.height))
    }
}

impl Drop for Tui {
    fn drop(&mut self) {
        let _ = self.exit();
    }
}

impl Default for Tui {
    fn default() -> Self {
        Self::new().expect("Failed to create TUI")
    }
}