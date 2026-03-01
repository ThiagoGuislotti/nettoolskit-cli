//! UI helper functions for common rendering patterns
//!
//! Provides reusable helper functions for common UI patterns used across
//! the application, promoting consistency and reducing code duplication.

use crate::core::colors::Color;
use crossterm::terminal;
use owo_colors::OwoColorize;

const NARROW_TERMINAL_WIDTH: usize = 80;
const COMPACT_TERMINAL_WIDTH: usize = 60;

/// Render a command header with consistent formatting
///
/// Displays the command being executed with proper spacing and color.
///
/// # Arguments
///
/// * `command` - The command name to display (without the "/" prefix)
///
/// # Example
///
/// ```no_run
/// use nettoolskit_ui::render_command;
/// render_command("manifest");
/// // Output:
/// //
/// // > manifest
/// //
/// ```
pub fn render_command(command: &str) {
    println!("{}", format!("> {}", command).color(Color::PURPLE));
}

/// Render menu navigation instructions
///
/// Displays standard instructions for navigating interactive menus.
///
/// # Example
///
/// ```no_run
/// use nettoolskit_ui::render_menu_instructions;
/// render_menu_instructions();
/// // Output: "   Use ↑↓ to navigate, Enter to select, /quit to exit"
/// ```
pub fn render_menu_instructions() {
    let message = menu_instructions_for_layout(
        terminal::size().ok().map(|(width, _)| width as usize),
        crate::capabilities().unicode,
    );
    println!("{}", message.color(Color::GRAY));
}

/// Render a section title with optional icon and underline
///
/// Displays a formatted section title with consistent styling, optional icon,
/// and an underline separator.
///
/// # Arguments
///
/// * `title` - The title text to display
/// * `icon` - Optional icon/emoji to prefix the title
///
/// # Example
///
/// ```no_run
/// use nettoolskit_ui::render_section_title;
/// render_section_title("Validating Manifest...", Some("✅"));
/// // Output:
/// //
/// // ✅ Validating Manifest...
/// // ───────────────────────────
/// //
/// ```
pub fn render_section_title(title: &str, icon: Option<&str>) {
    use crate::Color;
    println!();

    let display_text = if let Some(icon_str) = icon {
        format!("{} {}", icon_str, title)
    } else {
        title.to_string()
    };

    println!("{}", display_text.color(Color::CYAN).bold());

    // Calculate underline length based on visible characters (excluding ANSI codes)
    let underline_len = if icon.is_some() {
        title.len() + 2 // +2 for "icon + space"
    } else {
        title.len()
    };

    println!(
        "{}",
        crate::pick_str("─", "-")
            .repeat(underline_len)
            .color(Color::CYAN)
    );
    println!();
}

/// Format a menu item with label and description
///
/// Returns a formatted string with label and gray description,
/// following the pattern: "label - description"
///
/// # Arguments
///
/// * `label` - The main label text
/// * `description` - Optional description text (will be grayed out)
///
/// # Example
///
/// ```no_run
/// use nettoolskit_ui::format_menu_item;
/// let item = format_menu_item("check", Some("Validate manifest file"));
/// ```
pub fn format_menu_item(label: &str, description: Option<&str>) -> String {
    match description {
        Some(desc) if !desc.is_empty() => {
            format!("{} - {}", label, crate::maybe_gray(desc))
        }
        _ => label.to_string(),
    }
}

fn menu_instructions_for_layout(width: Option<usize>, unicode: bool) -> &'static str {
    let terminal_width = width.unwrap_or(NARROW_TERMINAL_WIDTH);

    if terminal_width < COMPACT_TERMINAL_WIDTH {
        "   Enter to select, /quit to exit"
    } else if terminal_width < NARROW_TERMINAL_WIDTH {
        if unicode {
            "   ↑↓ + Enter, /quit"
        } else {
            "   Up/Down + Enter, /quit"
        }
    } else if unicode {
        "   Use ↑↓ to navigate, Enter to select, /quit to exit"
    } else {
        "   Use Up/Down to navigate, Enter to select, /quit to exit"
    }
}

#[cfg(test)]
mod tests {
    use super::menu_instructions_for_layout;

    #[test]
    fn menu_instructions_wide_unicode() {
        assert_eq!(
            menu_instructions_for_layout(Some(120), true),
            "   Use ↑↓ to navigate, Enter to select, /quit to exit"
        );
    }

    #[test]
    fn menu_instructions_wide_ascii() {
        assert_eq!(
            menu_instructions_for_layout(Some(120), false),
            "   Use Up/Down to navigate, Enter to select, /quit to exit"
        );
    }

    #[test]
    fn menu_instructions_narrow_unicode() {
        assert_eq!(
            menu_instructions_for_layout(Some(70), true),
            "   ↑↓ + Enter, /quit"
        );
    }

    #[test]
    fn menu_instructions_narrow_ascii() {
        assert_eq!(
            menu_instructions_for_layout(Some(70), false),
            "   Up/Down + Enter, /quit"
        );
    }

    #[test]
    fn menu_instructions_compact() {
        assert_eq!(
            menu_instructions_for_layout(Some(50), true),
            "   Enter to select, /quit to exit"
        );
    }
}
