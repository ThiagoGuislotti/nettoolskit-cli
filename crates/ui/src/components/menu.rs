//! Interactive menu component with keyboard navigation
//!
//! Provides a configurable menu component using the inquire library
//! with consistent styling and behavior across the application.

use inquire::ui::{RenderConfig, Color, Styled};
use inquire::Select;
use owo_colors::{OwoColorize, Rgb};
use std::fmt::Display;

/// Configuration for rendering an interactive menu
#[derive(Debug, Clone)]
pub struct MenuConfig<T> {
    /// Prompt text displayed above the menu
    pub prompt: String,

    /// Menu items to display
    pub items: Vec<T>,

    /// Color for cursor and selected items
    pub cursor_color: Rgb,

    /// Help message displayed above the menu
    pub help_message: Option<String>,

    /// Number of items visible at once (page size)
    pub page_size: usize,

    /// Whether to show the help message from inquire (usually disabled)
    pub show_inquire_help: bool,
}

impl<T> MenuConfig<T> {
    /// Create a new MenuConfig with required fields
    pub fn new(prompt: impl Into<String>, items: Vec<T>) -> Self {
        Self {
            prompt: prompt.into(),
            items,
            cursor_color: Rgb(155, 114, 255), // PRIMARY_COLOR
            help_message: None,
            page_size: 6,
            show_inquire_help: false,
        }
    }

    /// Set the cursor color
    pub fn with_cursor_color(mut self, color: Rgb) -> Self {
        self.cursor_color = color;
        self
    }

    /// Set the help message
    pub fn with_help_message(mut self, message: impl Into<String>) -> Self {
        self.help_message = Some(message.into());
        self
    }

    /// Set the page size
    pub fn with_page_size(mut self, size: usize) -> Self {
        self.page_size = size.max(1);
        self
    }

    /// Set whether to show inquire's built-in help
    pub fn with_inquire_help(mut self, show: bool) -> Self {
        self.show_inquire_help = show;
        self
    }
}

/// Render an interactive menu and return the selected item
///
/// # Errors
/// Returns an error if the user cancels the menu (Ctrl+C or Esc)
pub fn render_interactive_menu<T>(config: MenuConfig<T>) -> Result<T, inquire::InquireError>
where
    T: Display + Clone,
{
    // Print custom help message if provided
    if let Some(help_msg) = &config.help_message {
        println!("{}", help_msg.yellow());
        println!();
    }

    // Configure render config with cursor color
    let mut render_config = RenderConfig::default();
    render_config.prompt_prefix = Styled::new("?").with_fg(Color::Rgb {
        r: config.cursor_color.0,
        g: config.cursor_color.1,
        b: config.cursor_color.2
    });
    render_config.highlighted_option_prefix = Styled::new("❯").with_fg(Color::Rgb {
        r: config.cursor_color.0,
        g: config.cursor_color.1,
        b: config.cursor_color.2
    });
    render_config.selected_option = Some(
        render_config.selected_option.unwrap_or_default().with_fg(Color::Rgb {
            r: config.cursor_color.0,
            g: config.cursor_color.1,
            b: config.cursor_color.2
        })
    );
    render_config.help_message = render_config.help_message.with_fg(Color::DarkYellow);

    // Build and execute the select prompt
    let mut select = Select::new(&config.prompt, config.items)
        .with_page_size(config.page_size)
        .with_render_config(render_config);

    if !config.show_inquire_help {
        select = select.without_help_message();
    }

    select.prompt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_menu_config_builder() {
        let items = vec!["Option 1", "Option 2", "Option 3"];
        let config = MenuConfig::new("Select an option:", items)
            .with_cursor_color(Rgb(255, 0, 0))
            .with_help_message("Use arrow keys")
            .with_page_size(5);

        assert_eq!(config.prompt, "Select an option:");
        assert_eq!(config.items.len(), 3);
        assert_eq!(config.page_size, 5);
        assert_eq!(config.cursor_color, Rgb(255, 0, 0));
    }
}
