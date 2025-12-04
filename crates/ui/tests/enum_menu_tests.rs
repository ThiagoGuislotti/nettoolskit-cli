//! Tests for enum-based menu functionality

use nettoolskit_ui::{EnumMenuConfig, Color};

// Test enum for demonstration
#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::Display)]
#[strum(serialize_all = "lowercase")]
enum TestCommand {
    #[strum(serialize = "start")]
    Start,
    #[strum(serialize = "stop")]
    Stop,
}

impl nettoolskit_core::MenuEntry for TestCommand {
    fn label(&self) -> &str {
        match self {
            TestCommand::Start => "start",
            TestCommand::Stop => "stop",
        }
    }

    fn description(&self) -> &str {
        match self {
            TestCommand::Start => "Start the service",
            TestCommand::Stop => "Stop the service",
        }
    }
}

impl nettoolskit_core::MenuProvider for TestCommand {
    fn menu_items() -> Vec<String> {
        vec![
            "start - Start the service".to_string(),
            "stop - Stop the service".to_string(),
        ]
    }

    fn all_variants() -> Vec<Self> {
        vec![TestCommand::Start, TestCommand::Stop]
    }
}

#[test]
fn test_enum_menu_config_creation() {
    // Act
    let config = EnumMenuConfig::new(
        "Test Menu",
        "Select an option",
        "/test/dir",
    );

    // Assert
    assert_eq!(config.title, "Test Menu");
    assert_eq!(config.subtitle, "Select an option");
    assert_eq!(config.current_dir, "/test/dir");
    assert_eq!(config.theme_color, Color::PURPLE);
    assert_eq!(config.width, 89);
}

#[test]
fn test_enum_menu_config_with_theme_color() {
    // Arrange
    let custom_color = Color::WHITE;

    // Act
    let config = EnumMenuConfig::new("Test", "Subtitle", "/dir")
        .with_theme_color(custom_color);

    // Assert
    assert_eq!(config.theme_color, custom_color);
}

#[test]
fn test_enum_menu_config_with_width() {
    // Act
    let config = EnumMenuConfig::new("Test", "Subtitle", "/dir")
        .with_width(120);

    // Assert
    assert_eq!(config.width, 120);
}

#[test]
fn test_enum_menu_config_with_footer_items() {
    // Act
    let config = EnumMenuConfig::new("Test", "Subtitle", "/dir")
        .add_footer_item("key1", "value1", Color::WHITE)
        .add_footer_item("key2", "value2", Color::GRAY);

    // Assert
    assert_eq!(config.footer_items.len(), 2);
    assert_eq!(config.footer_items[0].0, "key1");
    assert_eq!(config.footer_items[0].1, "value1");
    assert_eq!(config.footer_items[1].0, "key2");
    assert_eq!(config.footer_items[1].1, "value2");
}

#[test]
fn test_enum_menu_config_builder_pattern() {
    // Act
    let config = EnumMenuConfig::new("Menu", "Subtitle", "/path")
        .with_theme_color(Color::PURPLE_LIGHT)
        .with_width(100)
        .add_footer_item("project", "nettoolskit", Color::WHITE);

    // Assert
    assert_eq!(config.title, "Menu");
    assert_eq!(config.subtitle, "Subtitle");
    assert_eq!(config.current_dir, "/path");
    assert_eq!(config.theme_color, Color::PURPLE_LIGHT);
    assert_eq!(config.width, 100);
    assert_eq!(config.footer_items.len(), 1);
}

#[test]
fn test_test_command_menu_items() {
    use nettoolskit_core::MenuProvider;

    // Act
    let items = TestCommand::menu_items();

    // Assert
    assert_eq!(items.len(), 2);
    assert!(items.contains(&"start - Start the service".to_string()));
    assert!(items.contains(&"stop - Stop the service".to_string()));
}

#[test]
fn test_test_command_all_variants() {
    use nettoolskit_core::MenuProvider;

    // Act
    let variants = TestCommand::all_variants();

    // Assert
    assert_eq!(variants.len(), 2);
    assert!(variants.contains(&TestCommand::Start));
    assert!(variants.contains(&TestCommand::Stop));
}

#[test]
fn test_test_command_menu_entry_label() {
    use nettoolskit_core::MenuEntry;

    // Arrange
    let cmd = TestCommand::Start;

    // Act & Assert
    assert_eq!(cmd.label(), "start");
}

#[test]
fn test_test_command_menu_entry_description() {
    use nettoolskit_core::MenuEntry;

    // Arrange
    let cmd = TestCommand::Stop;

    // Act & Assert
    assert_eq!(cmd.description(), "Stop the service");
}