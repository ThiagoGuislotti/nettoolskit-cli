//! Tests for MenuConfig component
//!
//! Validates menu configuration, builder pattern, and interactive menu setup.
//! Tests cursor colors, page sizes, help messages, and prompt customization.

use nettoolskit_ui::components::MenuConfig;
use owo_colors::Rgb;

// Constructor and Builder Tests

#[test]
fn test_menu_config_new() {
    // Arrange
    let items = vec!["Option 1", "Option 2", "Option 3"];

    // Act
    let config = MenuConfig::new("Select an option:", items.clone());

    // Assert
    assert_eq!(config.prompt, "Select an option:");
    assert_eq!(config.items, items);
    assert_eq!(config.page_size, 6); // Default
    assert_eq!(config.cursor_color, Rgb(155, 114, 255)); // Default primary color (purple)
    assert_eq!(config.help_message, None);
}

#[test]
fn test_menu_config_with_cursor_color() {
    // Arrange
    let items = vec!["Option 1"];
    let red = Rgb(255, 0, 0);

    // Act
    let config = MenuConfig::new("Select:", items)
        .with_cursor_color(red);

    // Assert
    assert_eq!(config.cursor_color, red);
}

#[test]
fn test_menu_config_with_help_message() {
    // Arrange
    let items = vec!["Option 1"];
    let help = "Use arrow keys to navigate";

    // Act
    let config = MenuConfig::new("Select:", items)
        .with_help_message(help);

    // Assert
    assert_eq!(config.help_message, Some(help.to_string()));
}

#[test]
fn test_menu_config_with_page_size() {
    // Arrange
    let items = vec!["Option 1", "Option 2"];

    // Act
    let config = MenuConfig::new("Select:", items)
        .with_page_size(5);

    // Assert
    assert_eq!(config.page_size, 5);
}

#[test]
fn test_menu_config_page_size_minimum() {
    // Arrange
    let items = vec!["Option 1"];

    // Act
    let config = MenuConfig::new("Select:", items)
        .with_page_size(0); // Below minimum

    // Assert
    assert_eq!(config.page_size, 1, "Page size should be clamped to minimum of 1");
}

#[test]
fn test_menu_config_builder_chain() {
    // Arrange
    let items = vec!["A", "B", "C"];
    let color = Rgb(255, 100, 50);

    // Act
    let config = MenuConfig::new("Choose:", items.clone())
        .with_cursor_color(color)
        .with_help_message("Press Enter")
        .with_page_size(8);

    // Assert
    assert_eq!(config.prompt, "Choose:");
    assert_eq!(config.items, items);
    assert_eq!(config.cursor_color, color);
    assert_eq!(config.help_message, Some("Press Enter".to_string()));
    assert_eq!(config.page_size, 8);
}

// Edge Cases

#[test]
fn test_menu_config_empty_prompt() {
    // Arrange
    let items = vec!["Option"];

    // Act
    let config = MenuConfig::new("", items);

    // Assert
    assert_eq!(config.prompt, "");
}

#[test]
fn test_menu_config_single_item() {
    // Arrange
    let items = vec!["Only One"];

    // Act
    let config = MenuConfig::new("Select:", items.clone());

    // Assert
    assert_eq!(config.items.len(), 1);
    assert_eq!(config.items[0], "Only One");
}

#[test]
fn test_menu_config_many_items() {
    // Arrange
    let items: Vec<String> = (1..=100).map(|i| format!("Option {}", i)).collect();

    // Act
    let config = MenuConfig::new("Select:", items.clone());

    // Assert
    assert_eq!(config.items.len(), 100);
    assert_eq!(config.page_size, 6); // Should paginate
}

#[test]
fn test_menu_config_unicode_items() {
    // Arrange
    let items = vec!["🚀 Launch", "⚙️ Settings", "📊 Reports"];

    // Act
    let config = MenuConfig::new("Choose:", items.clone());

    // Assert
    assert_eq!(config.items, items);
}

#[test]
fn test_menu_config_long_prompt() {
    // Arrange
    let items = vec!["A", "B"];
    let long_prompt = "This is a very long prompt that might wrap across multiple lines in the terminal";

    // Act
    let config = MenuConfig::new(long_prompt, items);

    // Assert
    assert_eq!(config.prompt, long_prompt);
}

#[test]
fn test_menu_config_no_help_message() {
    // Arrange
    let items = vec!["Option"];

    // Act
    let config = MenuConfig::new("Select:", items);

    // Assert
    assert_eq!(config.help_message, None);
}

#[test]
fn test_menu_config_large_page_size() {
    // Arrange
    let items = vec!["A", "B", "C"];

    // Act
    let config = MenuConfig::new("Select:", items)
        .with_page_size(1000);

    // Assert
    assert_eq!(config.page_size, 1000);
}

// Type Safety Tests

#[test]
fn test_menu_config_with_custom_type() {
    // Arrange
    #[derive(Debug, Clone, PartialEq)]
    struct CustomOption {
        id: u32,
        name: String,
    }

    impl std::fmt::Display for CustomOption {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.name)
        }
    }

    let items = vec![
        CustomOption { id: 1, name: "First".to_string() },
        CustomOption { id: 2, name: "Second".to_string() },
    ];

    // Act
    let config = MenuConfig::new("Select:", items.clone());

    // Assert
    assert_eq!(config.items.len(), 2);
    assert_eq!(config.items[0].id, 1);
    assert_eq!(config.items[1].name, "Second");
}

#[test]
fn test_menu_config_with_string_type() {
    // Arrange
    let items = vec![
        String::from("Option 1"),
        String::from("Option 2"),
    ];

    // Act
    let config = MenuConfig::new("Select:", items.clone());

    // Assert
    assert_eq!(config.items, items);
}

#[test]
fn test_menu_config_with_str_slice() {
    // Arrange
    let items = vec!["Option 1", "Option 2"];

    // Act
    let config = MenuConfig::new("Select:", items.clone());

    // Assert
    assert_eq!(config.items, items);
}
