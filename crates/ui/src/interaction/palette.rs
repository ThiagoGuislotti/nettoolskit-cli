use crate::core::colors::{PRIMARY_COLOR, WHITE_COLOR};
use crate::rendering::components::{
    BoxConfig, MenuConfig, render_box, render_interactive_menu,
    render_command_header, render_menu_instructions,
};
use nettoolskit_core::MenuEntry;
use owo_colors::OwoColorize;

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
    /// Title for the menu box
    title: String,
    /// Subtitle for the menu box
    subtitle: Option<String>,
    /// Directory context to display in footer
    directory: Option<String>,
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
            title: String::from("Commands"),
            subtitle: None,
            directory: None,
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
        self.title = title.into();
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

        // Render command header if title looks like a command
        if self.title.starts_with('/') || self.title.contains("Commands") {
            let cmd = self.title.trim_start_matches('/').to_lowercase();
            if !cmd.is_empty() && !cmd.contains("commands") {
                render_command_header(&cmd);
            }
        }

        // Render box with title
        let mut box_config = BoxConfig::new(&self.title)
            .with_title_prefix(">_")
            .with_title_color(WHITE_COLOR)
            .with_border_color(PRIMARY_COLOR)
            .with_width(89)
            .with_spacing(false);

        if let Some(subtitle) = &self.subtitle {
            box_config = box_config.with_subtitle(subtitle);
        }

        if let Some(directory) = &self.directory {
            box_config = box_config.add_footer_item("directory", directory, WHITE_COLOR);
        }

        render_box(box_config);

        println!();
        render_menu_instructions();
        println!();

        // Build displayable items for inquire menu
        let display_items: Vec<String> = self
            .all_entries
            .iter()
            .map(|entry| {
                let desc = entry.description();
                if desc.is_empty() {
                    format!("   {}", entry.label())
                } else {
                    format!(
                        "   {} - \x1b[90m{}\x1b[0m",
                        entry.label(),
                        desc
                    )
                }
            })
            .collect();

        if display_items.is_empty() {
            println!("{}", "No menu options available".red());
            return None;
        }

        // Render interactive menu
        let menu_config = MenuConfig::new("Select option", display_items)
            .with_cursor_color(PRIMARY_COLOR)
            .with_page_size(10);

        match render_interactive_menu(menu_config) {
            Ok(selected) => {
                // Extract label from formatted string "   label - description"
                let label = selected
                    .trim()
                    .split(" - ")
                    .next()
                    .unwrap_or(&selected)
                    .trim();
                Some(label.to_string())
            }
            Err(_) => None, // User cancelled
        }
    }
}
