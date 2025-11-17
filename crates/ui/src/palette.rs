use crate::colors::{GRAY_COLOR, PRIMARY_COLOR};
use crate::style::set_fg;
use crossterm::style::{Attribute, Print, SetAttribute};
use crossterm::terminal::ClearType;
use crossterm::{cursor, queue, terminal};
use nettoolskit_core::MenuEntry;
use std::cmp::Ordering;
use std::io::{self, Write};/// Entry type used internally by CommandPalette
struct PaletteEntry {
    label: String,
    description: String,
}

impl MenuEntry for PaletteEntry {
    fn label(&self) -> &str {
        &self.label
    }

    fn description(&self) -> &str {
        &self.description
    }
}

/// Command palette state following Codex specification.
///
/// This structure manages the interactive command palette that allows users to search
/// and select from available slash commands. It maintains the visual state including
/// the current query, filtered matches, selection position, and terminal anchoring.
pub struct CommandPalette {
    /// The text typed after '/' for filtering commands
    query: String,
    /// All available entries
    all_entries: Vec<PaletteEntry>,
    /// Entries after filtering and ranking
    matches: Vec<usize>,
    /// Currently selected line in the visible window
    selected: usize,
    /// Starting position of the visible window (for scrolling)
    offset: usize,
    /// Input line position in the terminal (y coordinate)
    y_input: u16,
    /// Whether the palette is currently active and displayed
    active: bool,
}

impl CommandPalette {
    /// Maximum number of visible items in the palette (8 items as per specification)
    const MAX_VISIBLE_ITEMS: usize = 8;

    /// Creates a new command palette with the given menu entries.
    ///
    /// The palette is initialized with an empty query, all entries as potential matches,
    /// and default positioning values. The palette remains inactive until explicitly opened.
    ///
    /// # Arguments
    ///
    /// * `entries` - Vector of items implementing MenuEntry trait
    ///
    /// # Returns
    ///
    /// Returns a new `CommandPalette` instance ready for use.
    pub fn new<T: MenuEntry>(entries: Vec<T>) -> Self {
        let all_entries: Vec<PaletteEntry> = entries
            .into_iter()
            .map(|e| PaletteEntry {
                label: e.label().to_string(),
                description: e.description().to_string(),
            })
            .collect();

        let matches: Vec<usize> = (0..all_entries.len()).collect();

        Self {
            query: String::new(),
            all_entries,
            matches,
            selected: 0,
            offset: 0,
            y_input: 0,
            active: false,
        }
    }

    /// Opens the command palette with an initial query.
    ///
    /// This function activates the palette, sets the initial search query, anchors the palette
    /// to the current cursor position, ensures sufficient terminal space, updates the matches,
    /// and renders the initial view.
    ///
    /// # Arguments
    ///
    /// * `initial_query` - The initial search text to filter commands
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success or an `io::Error` if terminal operations fail.
    pub fn open(&mut self, initial_query: &str) -> io::Result<()> {
        self.active = true;
        self.query = initial_query.to_string();

        // Use cursor::position() to capture y_input and anchor the palette
        if let Ok((_, y)) = cursor::position() {
            self.y_input = y;
        }

        // Ensure sufficient terminal space for the palette
        self.ensure_terminal_space()?;

        self.update_matches();
        // Reset selection to 0 as per acceptance criteria
        self.selected = 0;
        self.offset = 0;

        self.render()
    }

    /// Closes the command palette and cleans up the terminal region.
    ///
    /// This function deactivates the palette, clears the terminal region used by the palette,
    /// and repositions the cursor back to the input line without adding new lines to the history.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success or an `io::Error` if terminal operations fail.
    pub fn close(&mut self) -> io::Result<()> {
        if !self.active {
            return Ok(());
        }

        self.active = false;

        // Clear the entire region used by the palette as per specification
        self.clear_region()?;

        // Reposition cursor to input line as per specification
        // Do not print additional lines to history
        if let Ok((x, _)) = cursor::position() {
            queue!(io::stdout(), cursor::MoveTo(x, self.y_input))?;
        }

        io::stdout().flush()
    }

    /// Returns whether the command palette is currently active.
    ///
    /// # Returns
    ///
    /// Returns `true` if the palette is open and displayed, `false` otherwise.
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Updates the search query and recalculates matches with real-time filtering.
    ///
    /// This function updates the current search query, recalculates which commands match
    /// the new query, resets the selection to the first match, and re-renders the palette.
    ///
    /// # Arguments
    ///
    /// * `new_query` - The new search text to filter commands
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success or an `io::Error` if rendering fails.
    pub fn update_query(&mut self, new_query: &str) -> io::Result<()> {
        if !self.active {
            return Ok(());
        }

        self.query = new_query.to_string();
        self.update_matches();

        // Reset selection to 0 as per acceptance criteria
        self.selected = 0;
        self.offset = 0;

        // Update latency ≤ 1 terminal frame
        self.render()
    }

    /// Navigates up in the command list.
    ///
    /// Moves the selection to the previous command in the list. If already at the top,
    /// wraps around to the last command.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success or an `io::Error` if rendering fails.
    pub fn navigate_up(&mut self) -> io::Result<()> {
        if !self.active || self.matches.is_empty() {
            return Ok(());
        }

        if self.selected > 0 {
            self.selected -= 1;
            self.adjust_scroll_offset();
        } else {
            self.selected = self.matches.len() - 1;
            self.adjust_scroll_offset();
        }
        self.render_fast()
    }

    /// Navigates down in the command list.
    ///
    /// Moves the selection to the next command in the list. If already at the bottom,
    /// wraps around to the first command.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success or an `io::Error` if rendering fails.
    pub fn navigate_down(&mut self) -> io::Result<()> {
        if !self.active || self.matches.is_empty() {
            return Ok(());
        }

        if self.selected < self.matches.len() - 1 {
            self.selected += 1;
        } else {
            self.selected = 0; // Cycles to the first
        }
        self.adjust_scroll_offset();
        self.render_fast()
    }

    /// Navigates to the first command (Home key behavior).
    ///
    /// Moves the selection to the first command in the filtered list and adjusts
    /// the scroll offset as needed.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success or an `io::Error` if rendering fails.
    pub fn navigate_home(&mut self) -> io::Result<()> {
        if !self.active || self.matches.is_empty() {
            return Ok(());
        }

        self.selected = 0;
        self.adjust_scroll_offset();
        self.render_fast()
    }

    /// Navigates to the last command (End key behavior).
    ///
    /// Moves the selection to the last command in the filtered list and adjusts
    /// the scroll offset as needed.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success or an `io::Error` if rendering fails.
    pub fn navigate_end(&mut self) -> io::Result<()> {
        if !self.active || self.matches.is_empty() {
            return Ok(());
        }

        self.selected = self.matches.len().saturating_sub(1);
        self.adjust_scroll_offset();
        self.render_fast()
    }

    /// Returns the currently selected command.
    ///
    /// Gets the name of the command that is currently highlighted in the palette.
    ///
    /// # Returns
    ///
    /// Returns `Some(&str)` with the command name if there is a selection,
    /// or `None` if no commands match the current query.
    pub fn get_selected_command(&self) -> Option<&str> {
        if !self.active || self.matches.is_empty() {
            return None;
        }

        self.matches
            .get(self.selected)
            .and_then(|&idx| self.all_entries.get(idx))
            .map(|entry| entry.label())
    }

    /// Updates the list of matching commands based on the current query.
    ///
    /// This function filters and ranks all available commands based on how well they match
    /// the current search query. Commands are sorted by relevance, with exact matches and
    /// prefix matches ranking higher than substring matches.
    fn update_matches(&mut self) {
        let query = self.query.trim().to_lowercase();

        if query.is_empty() {
            self.matches = (0..self.all_entries.len()).collect();
            return;
        }

        let mut scored_matches: Vec<(usize, i32)> = Vec::new();

        for (idx, entry) in self.all_entries.iter().enumerate() {
            let label = entry.label().to_lowercase();
            let desc = entry.description().to_lowercase();

            let score = if label.starts_with(&query) {
                3 // Highest priority for starts_with in label
            } else if label.contains(&query) {
                2 // Second priority for contains in label
            } else if desc.contains(&query) {
                1 // Lower priority for contains in description
            } else {
                continue; // Don't include if no match
            };

            scored_matches.push((idx, score));
        }

        // Sort by score (desc) then by label (asc)
        scored_matches.sort_by(|a, b| match b.1.cmp(&a.1) {
            Ordering::Equal => {
                let label_a = &self.all_entries[a.0].label;
                let label_b = &self.all_entries[b.0].label;
                label_a.cmp(label_b)
            }
            other => other,
        });

        self.matches = scored_matches.into_iter().map(|(idx, _)| idx).collect();
    }

    /// Adjusts the scroll offset to keep the selected item visible within the 8-line window.
    ///
    /// This function ensures that the currently selected command is always visible in the
    /// palette by adjusting the scroll offset when the selection moves outside the visible area.
    fn adjust_scroll_offset(&mut self) {
        if self.matches.is_empty() {
            return;
        }

        let max_visible = Self::MAX_VISIBLE_ITEMS.min(self.matches.len());

        // Adjust offset to keep selected within [0, matches.len())
        if self.selected >= self.matches.len() {
            self.selected = self.matches.len().saturating_sub(1);
        }

        // Keep the selected item visible within the 8-line window
        if self.selected >= self.offset + max_visible {
            self.offset = self.selected - max_visible + 1;
        } else if self.selected < self.offset {
            self.offset = self.selected;
        }
    }

    /// Ensures there is sufficient space in the terminal to display the palette.
    ///
    /// This function checks if there is enough vertical space below the current cursor
    /// position to display the palette. If not, it scrolls the terminal or moves the
    /// cursor to create the necessary space.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success or an `io::Error` if terminal operations fail.
    fn ensure_terminal_space(&mut self) -> io::Result<()> {
        // Get terminal dimensions
        if let Ok((_, terminal_height)) = terminal::size() {
            let lines_needed = Self::MAX_VISIBLE_ITEMS as u16 + 2; // +2 for spacing
            let available_lines = terminal_height.saturating_sub(self.y_input + 1);

            if available_lines < lines_needed {
                // Not enough space, we need to scroll
                let lines_to_scroll = lines_needed - available_lines;

                // Scroll up (add blank lines at the end)
                for _ in 0..lines_to_scroll {
                    queue!(io::stdout(), Print("\n"))?;
                }

                // Update y_input position after scroll
                self.y_input = self.y_input.saturating_sub(lines_to_scroll);

                // Move cursor to new input position
                queue!(io::stdout(), cursor::MoveTo(0, self.y_input))?;
                io::stdout().flush()?;
            }
        }

        Ok(())
    }

    /// Clears the terminal region used by the palette.
    ///
    /// This function clears all lines that were used to display the palette,
    /// ensuring the terminal is cleaned up when the palette is closed.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success or an `io::Error` if terminal operations fail.
    fn clear_region(&self) -> io::Result<()> {
        // Clear from line y_input + 1 to the end of palette area
        queue!(
            io::stdout(),
            cursor::MoveTo(0, self.y_input + 1),
            terminal::Clear(ClearType::FromCursorDown)
        )?;

        // Clear only necessary lines to avoid flickering (including extra spacing line)
        let visible_items = Self::MAX_VISIBLE_ITEMS.min(self.matches.len());
        for i in 0..=visible_items + 1 {
            // +1 for extra spacing line
            queue!(
                io::stdout(),
                cursor::MoveTo(0, self.y_input + 1 + i as u16),
                terminal::Clear(terminal::ClearType::CurrentLine)
            )?;
        }

        io::stdout().flush()
    }

    /// Renders the complete command palette following Codex specification.
    ///
    /// This function performs a full render of the palette, including the header,
    /// all visible command items, and proper highlighting of the selected item.
    /// It handles the visual layout and ensures proper positioning in the terminal.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success or an `io::Error` if terminal operations fail.
    fn render(&self) -> io::Result<()> {
        if !self.active {
            return Ok(());
        }

        // Save current cursor position
        let original_cursor_pos = cursor::position().unwrap_or((0, self.y_input));

        // 1) Clear from line y_input + 1 to the end of palette area
        self.clear_region()?;

        // 2) Draw min(8, matches.len()) lines starting from offset
        let visible_items = Self::MAX_VISIBLE_ITEMS.min(self.matches.len());
        let end_idx = (self.offset + visible_items).min(self.matches.len());

        for i in self.offset..end_idx {
            let line_idx = i - self.offset;
            let y_pos = self.y_input + 2 + line_idx as u16; // Palette with one spacing line below input

            if let Some(&entry_idx) = self.matches.get(i) {
                if let Some(entry) = self.all_entries.get(entry_idx) {
                    let is_selected = i == self.selected;

                    queue!(io::stdout(), cursor::MoveTo(0, y_pos))?;

                    if is_selected {
                        // Selected item - highlight with reverse as per specification
                        queue!(
                            io::stdout(),
                            SetAttribute(Attribute::Reverse),
                            Print(format!("› {}  {}", entry.label(), entry.description())), // Two spaces between command and description
                            SetAttribute(Attribute::Reset)
                        )?;
                    } else {
                        // Unselected item - NetToolsKit colors (purple/magenta)
                        queue!(
                            io::stdout(),
                            set_fg(GRAY_COLOR),
                            Print("  "),
                            set_fg(PRIMARY_COLOR),
                            Print(entry.label()),
                            set_fg(GRAY_COLOR),
                            Print(format!("  {}", entry.description())), // Two spaces between command and description
                            SetAttribute(Attribute::Reset)
                        )?;
                    }
                }
            }
        }

        // Restore original cursor position on input line
        queue!(
            io::stdout(),
            cursor::MoveTo(original_cursor_pos.0, original_cursor_pos.1)
        )?;
        // 3) Flush output as per specification
        io::stdout().flush()
    }

    /// Fast rendering for navigation updates (without full clear_region).
    ///
    /// This function provides optimized rendering for navigation operations,
    /// updating only the visible items without clearing the entire terminal region.
    /// This reduces flicker and improves responsiveness during navigation.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success or an `io::Error` if terminal operations fail.
    fn render_fast(&self) -> io::Result<()> {
        if !self.active {
            return Ok(());
        }

        // Save current cursor position
        let original_cursor_pos = cursor::position().unwrap_or((0, self.y_input));

        // Draw only visible lines without clearing the entire region
        let visible_items = Self::MAX_VISIBLE_ITEMS.min(self.matches.len());
        let end_idx = (self.offset + visible_items).min(self.matches.len());

        for i in self.offset..end_idx {
            let line_idx = i - self.offset;
            let y_pos = self.y_input + 2 + line_idx as u16;

            if let Some(&entry_idx) = self.matches.get(i) {
                if let Some(entry) = self.all_entries.get(entry_idx) {
                    let is_selected = i == self.selected;

                    // Clear only the current line before drawing
                    queue!(
                        io::stdout(),
                        cursor::MoveTo(0, y_pos),
                        terminal::Clear(terminal::ClearType::CurrentLine)
                    )?;

                    if is_selected {
                        // Selected item - highlight with reverse as per specification
                        queue!(
                            io::stdout(),
                            SetAttribute(Attribute::Reverse),
                            Print(format!("› {}  {}", entry.label(), entry.description())),
                            SetAttribute(Attribute::Reset)
                        )?;
                    } else {
                        // Unselected item - NetToolsKit colors
                        queue!(
                            io::stdout(),
                            set_fg(GRAY_COLOR),
                            Print("  "),
                            set_fg(PRIMARY_COLOR),
                            Print(entry.label()),
                            set_fg(GRAY_COLOR),
                            Print(format!("  {}", entry.description())),
                            SetAttribute(Attribute::Reset)
                        )?;
                    }
                }
            }
        }

        // Restore original cursor position on input line
        queue!(
            io::stdout(),
            cursor::MoveTo(original_cursor_pos.0, original_cursor_pos.1)
        )?;
        io::stdout().flush()
    }
}
