//! UI helper functions for common rendering patterns
//!
//! Provides reusable helper functions for common UI patterns used across
//! the application, promoting consistency and reducing code duplication.

use crate::core::colors::Color;
use owo_colors::OwoColorize;

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
    println!(
        "{}",
        "   Use ↑↓ to navigate, Enter to select, /quit to exit".color(Color::GRAY)
    );
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
    println!();

    let display_text = if let Some(icon_str) = icon {
        format!("{} {}", icon_str, title)
    } else {
        title.to_string()
    };

    println!("{}", display_text.cyan().bold());

    // Calculate underline length based on visible characters (excluding ANSI codes)
    let underline_len = if icon.is_some() {
        title.len() + 2 // +2 for "icon + space"
    } else {
        title.len()
    };

    println!("{}", "─".repeat(underline_len).cyan());
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
/// // Returns: "check - \x1b[90mValidate manifest file\x1b[0m"
/// ```
pub fn format_menu_item(label: &str, description: Option<&str>) -> String {
    match description {
        Some(desc) if !desc.is_empty() => {
            format!("{} - \x1b[90m{}\x1b[0m", label, desc)
        }
        _ => label.to_string(),
    }
}
