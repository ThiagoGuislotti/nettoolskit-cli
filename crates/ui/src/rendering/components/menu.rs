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
        println!();
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
    // Always disable inquire's built-in help message
    Select::new(&config.prompt, config.items)
        .with_page_size(config.page_size)
        .with_render_config(render_config)
        .without_help_message()
        .prompt()
}
