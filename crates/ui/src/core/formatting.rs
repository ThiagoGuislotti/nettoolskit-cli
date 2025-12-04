//! Text formatting utilities for menu items and display

/// Format a menu item with aligned description.
///
/// Calculates padding to align descriptions at a fixed column,
/// ensuring consistent visual alignment regardless of label length.
///
/// # Arguments
///
/// * `label` - The command label (e.g., "/ help")
/// * `description` - The description text
/// * `align_column` - Column position where description should start (default: 20)
///
/// # Returns
///
/// Formatted string with aligned description
///
/// # Examples
///
/// ```
/// use nettoolskit_ui::core::formatting::format_menu_item;
///
/// let formatted = format_menu_item("/ help", "Display help", 20);
/// // Result: "   / help           - Display help"
/// ```
pub fn format_menu_item(label: &str, description: &str, align_column: usize) -> String {
    if description.is_empty() {
        return format!("   {}", label);
    }

    // Calculate actual label width (including "   " prefix)
    let prefix = "   ";
    let label_width = prefix.len() + label.len();

    // Calculate spaces needed to reach align column
    let spaces_needed = if label_width < align_column {
        align_column - label_width
    } else {
        2 // Minimum spacing if label is too long
    };

    format!(
        "{}{}{}- \x1b[90m{}\x1b[0m",
        prefix,
        label,
        " ".repeat(spaces_needed),
        description
    )
}
