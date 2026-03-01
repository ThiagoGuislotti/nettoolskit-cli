//! Generic enum-based menu renderer
//!
//! Provides a reusable, type-safe menu system for enums that implement MenuProvider.
//! This allows consistent menu rendering across all command types.

use crate::rendering::components::box_component::responsive_box_width_from_terminal;
use crate::{render_box, render_interactive_menu, BoxConfig, Color, MenuConfig};
use crossterm::terminal;
use nettoolskit_core::MenuProvider;
use std::fmt::Display;

const NARROW_TERMINAL_WIDTH: usize = 80;
const COMPACT_TERMINAL_WIDTH: usize = 60;
const DEFAULT_PAGE_SIZE: usize = 6;
const NARROW_PAGE_SIZE: usize = 5;
const COMPACT_PAGE_SIZE: usize = 4;

/// Configuration for rendering an enum-based menu
#[derive(Debug, Clone)]
pub struct EnumMenuConfig {
    /// Box title (e.g., "Main Menu")
    pub title: String,

    /// Box subtitle (e.g., "Select a command")
    pub subtitle: String,

    /// Current directory to display in footer
    pub current_dir: String,

    /// Color for borders and cursor
    pub theme_color: owo_colors::Rgb,

    /// Box width
    pub width: usize,

    /// Additional footer items (key, value, color)
    pub footer_items: Vec<(String, String, owo_colors::Rgb)>,
}

impl EnumMenuConfig {
    /// Create a new enum menu config with required fields
    pub fn new(
        title: impl Into<String>,
        subtitle: impl Into<String>,
        current_dir: impl Into<String>,
    ) -> Self {
        Self {
            title: title.into(),
            subtitle: subtitle.into(),
            current_dir: current_dir.into(),
            theme_color: Color::PURPLE,
            width: responsive_box_width_from_terminal(
                terminal::size().ok().map(|(width, _)| width as usize),
            ),
            footer_items: Vec::new(),
        }
    }

    /// Set the theme color
    pub fn with_theme_color(mut self, color: owo_colors::Rgb) -> Self {
        self.theme_color = color;
        self
    }

    /// Set the box width
    pub fn with_width(mut self, width: usize) -> Self {
        self.width = width;
        self
    }

    /// Add a footer item
    pub fn add_footer_item(
        mut self,
        key: impl Into<String>,
        value: impl Into<String>,
        color: owo_colors::Rgb,
    ) -> Self {
        self.footer_items.push((key.into(), value.into(), color));
        self
    }
}

/// Render an interactive menu for an enum that implements MenuProvider
///
/// This function provides a complete menu experience:
/// 1. Renders a styled box with title, subtitle, and footer
/// 2. Displays an interactive selection menu
/// 3. Returns the selected enum variant
///
/// # Type Parameters
/// - `T`: The enum type that implements `MenuProvider + Display`
///
/// # Errors
/// Returns an error if the user cancels the menu (Ctrl+C or Esc)
///
/// # Example
/// ```rust,ignore
/// let config = EnumMenuConfig::new("Main Menu", "Select a command", "/path/to/dir");
/// let selected = render_enum_menu::<Command>(config)?;
/// ```
pub fn render_enum_menu<T>(config: EnumMenuConfig) -> Result<T, inquire::InquireError>
where
    T: MenuProvider + Display,
{
    let compact_mode = is_narrow_terminal(config.width);

    // Build and render the box
    let mut box_config = BoxConfig::new(&config.title)
        .with_title_prefix(">_")
        .with_title_color(Color::WHITE)
        .with_border_color(config.theme_color)
        .with_width(config.width)
        .with_spacing(true);

    if !compact_mode {
        box_config = box_config.with_subtitle(&config.subtitle).add_footer_item(
            "directory",
            config.current_dir,
            Color::WHITE,
        );

        // Add any additional footer items
        for (key, value, color) in config.footer_items {
            box_config = box_config.add_footer_item(key, value, color);
        }
    }

    render_box(box_config);
    println!();

    // Build menu items from enum
    let menu_items = T::menu_items();

    // Create interactive menu
    let menu_config = MenuConfig::new(menu_prompt_for_width(config.width), menu_items)
        .with_cursor_color(config.theme_color)
        .with_page_size(menu_page_size_for_width(config.width));

    // Get user selection
    let selected_display = render_interactive_menu(menu_config)?;

    // Parse selection back to enum variant
    // Extract the label from "label - description" format
    let selected_label = selected_display.split(" - ").next().unwrap_or("");

    // Find matching variant
    T::all_variants()
        .into_iter()
        .find(|variant| variant.label() == selected_label)
        .ok_or_else(|| {
            inquire::InquireError::Custom(format!("Invalid selection: {}", selected_label).into())
        })
}

fn is_narrow_terminal(width: usize) -> bool {
    width < NARROW_TERMINAL_WIDTH
}

fn menu_prompt_for_width(width: usize) -> &'static str {
    if width < NARROW_TERMINAL_WIDTH {
        "Select:"
    } else {
        "Select an option:"
    }
}

fn menu_page_size_for_width(width: usize) -> usize {
    if width < COMPACT_TERMINAL_WIDTH {
        COMPACT_PAGE_SIZE
    } else if width < NARROW_TERMINAL_WIDTH {
        NARROW_PAGE_SIZE
    } else {
        DEFAULT_PAGE_SIZE
    }
}

#[cfg(test)]
mod tests {
    use super::{is_narrow_terminal, menu_page_size_for_width, menu_prompt_for_width};

    #[test]
    fn enum_menu_compact_detection() {
        assert!(is_narrow_terminal(70));
        assert!(!is_narrow_terminal(80));
    }

    #[test]
    fn enum_menu_prompt_changes_for_narrow_terminal() {
        assert_eq!(menu_prompt_for_width(70), "Select:");
        assert_eq!(menu_prompt_for_width(80), "Select an option:");
    }

    #[test]
    fn enum_menu_page_size_changes_by_width() {
        assert_eq!(menu_page_size_for_width(50), 4);
        assert_eq!(menu_page_size_for_width(70), 5);
        assert_eq!(menu_page_size_for_width(120), 6);
    }
}
