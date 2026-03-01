use crate::core::colors::Color;
use crate::core::formatting::format_menu_item;
use crate::rendering::components::{
    render_box, render_command, render_interactive_menu, render_menu_instructions, BoxConfig,
    MenuConfig,
};
use crossterm::terminal;
use nettoolskit_core::MenuEntry;
use owo_colors::OwoColorize;

const NARROW_TERMINAL_WIDTH: usize = 80;
const COMPACT_TERMINAL_WIDTH: usize = 60;
const DEFAULT_ALIGN_COLUMN: usize = 20;
const NARROW_ALIGN_COLUMN: usize = 14;
const COMPACT_ALIGN_COLUMN: usize = 10;

/// Entry type used internally by CommandPalette
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

/// Command palette for interactive menu display.
///
/// Uses boxed layout (manifest style) with full-screen menu, border,
/// title, and subtitle for visual clarity and consistency.
pub struct CommandPalette {
    /// All available entries
    all_entries: Vec<PaletteEntry>,
    /// Title for the menu box (optional)
    title: Option<String>,
    /// Subtitle for the menu box
    subtitle: Option<String>,
    /// Directory context to display in footer
    directory: Option<String>,
    /// Prompt text for the menu selection (defaults to "Select →")
    prompt: Option<String>,
}

impl CommandPalette {
    /// Creates a new command palette with the given menu entries.
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

        Self {
            all_entries,
            title: None,
            subtitle: None,
            directory: None,
            prompt: None,
        }
    }

    /// Set the title for the menu.
    ///
    /// # Arguments
    ///
    /// * `title` - The title to display in the box
    ///
    /// # Returns
    ///
    /// Returns self for method chaining.
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set the subtitle for the menu.
    ///
    /// # Arguments
    ///
    /// * `subtitle` - The subtitle to display in the box
    ///
    /// # Returns
    ///
    /// Returns self for method chaining.
    pub fn with_subtitle(mut self, subtitle: impl Into<String>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }

    /// Set the directory context to display in footer.
    ///
    /// # Arguments
    ///
    /// * `directory` - The directory path to display
    ///
    /// # Returns
    ///
    /// Returns self for method chaining.
    pub fn with_directory(mut self, directory: impl Into<String>) -> Self {
        self.directory = Some(directory.into());
        self
    }

    /// Set the prompt text for the menu selection.
    ///
    /// # Arguments
    ///
    /// * `prompt` - The prompt text to display
    ///
    /// # Returns
    ///
    /// Returns self for method chaining.
    pub fn with_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.prompt = Some(prompt.into());
        self
    }

    /// Reloads the palette with new menu entries.
    ///
    /// # Arguments
    ///
    /// * `entries` - Vector of new items implementing MenuEntry trait
    pub fn reload_entries<T: MenuEntry>(&mut self, entries: Vec<T>) {
        self.all_entries = entries
            .into_iter()
            .map(|e| PaletteEntry {
                label: e.label().to_string(),
                description: e.description().to_string(),
            })
            .collect();
    }

    /// Shows the menu and returns the selected option.
    ///
    /// This function displays a full-screen boxed menu (manifest style) with the
    /// configured title, subtitle, and directory context. Uses render_box and
    /// render_interactive_menu components for visual consistency.
    ///
    /// # Returns
    ///
    /// Returns `Some(String)` with the selected command label, or `None` if cancelled.
    pub fn show(&self) -> Option<String> {
        // Don't clear screen - keep terminal layout and logs visible
        println!();
        let terminal_width = current_terminal_width();
        let compact_mode = is_narrow_terminal(terminal_width);

        // Render command header if title looks like a command
        if let Some(title) = &self.title {
            if title.starts_with('/') || title.contains("Commands") {
                let cmd = title.trim_start_matches('/').to_lowercase();
                if !cmd.is_empty() && !cmd.contains("commands") {
                    render_command(&cmd);
                }
            }
        }

        // Skip box rendering for main menu (already shown at startup)
        let is_main_menu = self
            .title
            .as_ref()
            .map(|t| t == "NetToolsKit Commands" || t == "Commands")
            .unwrap_or(true);

        if !is_main_menu {
            // Render box with title for submenus
            if let Some(title) = &self.title {
                let title_width = title.len() + 4;
                let detected_dir = std::env::current_dir()
                    .ok()
                    .and_then(|p| p.to_str().map(String::from));
                let current_dir = self.directory.clone().or(detected_dir);

                let mut box_config = BoxConfig::new(title)
                    .with_title_color(Color::WHITE)
                    .with_border_color(Color::PURPLE)
                    .with_width(title_width);

                if !compact_mode {
                    if let Some(subtitle) = &self.subtitle {
                        box_config = box_config.with_subtitle(subtitle);
                    }
                }

                if !compact_mode {
                    if let Some(directory) = current_dir {
                        box_config =
                            box_config.add_footer_item("directory", directory, Color::WHITE);
                    }
                }

                render_box(box_config);
            }
        }

        println!();
        render_menu_instructions();
        println!();

        // Build displayable items for inquire menu with aligned descriptions
        let display_items: Vec<String> = self
            .all_entries
            .iter()
            .map(|entry| {
                format_menu_item(
                    entry.label(),
                    entry.description(),
                    align_column_for_width(terminal_width),
                )
            })
            .collect();

        if display_items.is_empty() {
            println!("{}", "No menu options available".color(Color::RED));
            return None;
        }

        // Render interactive menu
        let prompt = self.prompt.as_deref().unwrap_or(default_prompt_for_width(
            terminal_width,
            crate::capabilities().unicode,
        ));
        let menu_config = MenuConfig::new(prompt, display_items).with_cursor_color(Color::PURPLE);

        match render_interactive_menu(menu_config) {
            Ok(selected) => {
                // Extract label from formatted string "   / help           - description"
                // The format_menu_item adds padding between label and description
                let label = selected.split(" - ").next().unwrap_or(&selected).trim();

                Some(label.to_string())
            }
            Err(_) => None, // User cancelled
        }
    }
}

fn current_terminal_width() -> Option<usize> {
    terminal::size().ok().map(|(width, _)| width as usize)
}

fn is_narrow_terminal(width: Option<usize>) -> bool {
    width.is_some_and(|value| value < NARROW_TERMINAL_WIDTH)
}

fn align_column_for_width(width: Option<usize>) -> usize {
    let terminal_width = width.unwrap_or(NARROW_TERMINAL_WIDTH);
    if terminal_width < COMPACT_TERMINAL_WIDTH {
        COMPACT_ALIGN_COLUMN
    } else if terminal_width < NARROW_TERMINAL_WIDTH {
        NARROW_ALIGN_COLUMN
    } else {
        DEFAULT_ALIGN_COLUMN
    }
}

fn default_prompt_for_width(width: Option<usize>, unicode: bool) -> &'static str {
    let terminal_width = width.unwrap_or(NARROW_TERMINAL_WIDTH);
    if terminal_width < COMPACT_TERMINAL_WIDTH {
        "> "
    } else if terminal_width < NARROW_TERMINAL_WIDTH || !unicode {
        "Select >"
    } else {
        "Select →"
    }
}

#[cfg(test)]
mod tests {
    use super::{align_column_for_width, default_prompt_for_width, is_narrow_terminal};

    #[test]
    fn narrow_terminal_detection_works() {
        assert!(is_narrow_terminal(Some(70)));
        assert!(!is_narrow_terminal(Some(80)));
        assert!(!is_narrow_terminal(None));
    }

    #[test]
    fn align_column_uses_compact_threshold() {
        assert_eq!(align_column_for_width(Some(50)), 10);
        assert_eq!(align_column_for_width(Some(70)), 14);
        assert_eq!(align_column_for_width(Some(120)), 20);
    }

    #[test]
    fn prompt_fallback_respects_width_and_unicode() {
        assert_eq!(default_prompt_for_width(Some(50), true), "> ");
        assert_eq!(default_prompt_for_width(Some(70), true), "Select >");
        assert_eq!(default_prompt_for_width(Some(120), false), "Select >");
        assert_eq!(default_prompt_for_width(Some(120), true), "Select →");
    }
}
