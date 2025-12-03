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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_menu_item_short_label() {
        let result = format_menu_item("/ help", "Display help", 20);
        assert!(result.contains("/ help"));
        assert!(result.contains("Display help"));
    }

    #[test]
    fn test_format_menu_item_long_label() {
        let result = format_menu_item("/ verylongcommand", "Description", 20);
        assert!(result.contains("/ verylongcommand"));
        assert!(result.contains("Description"));
    }

    #[test]
    fn test_format_menu_item_empty_description() {
        let result = format_menu_item("/ test", "", 20);
        assert_eq!(result, "   / test");
    }

    #[test]
    fn test_alignment_consistency() {
        let items = vec![
            format_menu_item("/ help", "Help text", 20),
            format_menu_item("/ manifest", "Manifest text", 20),
            format_menu_item("/ translate", "Translate text", 20),
        ];

        // Find position of first dash in each item
        let dash_positions: Vec<usize> = items
            .iter()
            .filter_map(|s| s.find(" - "))
            .collect();

        // All dashes should be at the same position (or close for long labels)
        if dash_positions.len() > 1 {
            let first = dash_positions[0];
            for pos in &dash_positions[1..] {
                assert!((first as i32 - *pos as i32).abs() <= 2);
            }
        }
    }
}
