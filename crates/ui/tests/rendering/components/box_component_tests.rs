//! Tests for BoxConfig component
//!
//! Validates box rendering with borders, title, subtitle, and footer items.
//! Tests padding calculations, color handling, and layout correctness.

use nettoolskit_ui::BoxConfig;
use owo_colors::Rgb;

// Constructor and Builder Tests

#[test]
fn test_box_config_new() {
    // Arrange & Act
    let config = BoxConfig::new("Test Title");

    // Assert
    assert_eq!(config.title, "Test Title");
    assert_eq!(config.title_color, Rgb(255, 255, 255)); // Default white
    assert_eq!(config.subtitle, None);
    assert_eq!(config.title_prefix, None);
    assert!(config.footer_items.is_empty());
    assert_eq!(config.width, 89); // Default width
}

#[test]
fn test_box_config_with_subtitle() {
    // Arrange & Act
    let config = BoxConfig::new("Title")
        .with_subtitle("Subtitle text");

    // Assert
    assert_eq!(config.subtitle, Some("Subtitle text".to_string()));
}

#[test]
fn test_box_config_with_title_prefix() {
    // Arrange & Act
    let config = BoxConfig::new("Title")
        .with_title_prefix(">_");

    // Assert
    assert_eq!(config.title_prefix, Some(">_".to_string()));
}

#[test]
fn test_box_config_with_footer_item() {
    // Arrange & Act
    let config = BoxConfig::new("Title")
        .add_footer_item("directory", "/home/user", Rgb(0, 255, 0));

    // Assert
    assert_eq!(config.footer_items.len(), 1);
    assert_eq!(config.footer_items[0].0, "directory");
    assert_eq!(config.footer_items[0].1, "/home/user");
    assert_eq!(config.footer_items[0].2, Rgb(0, 255, 0));
}

#[test]
fn test_box_config_multiple_footer_items() {
    // Arrange & Act
    let config = BoxConfig::new("Title")
        .add_footer_item("label1", "value1", Rgb(255, 0, 0))
        .add_footer_item("label2", "value2", Rgb(0, 255, 0))
        .add_footer_item("label3", "value3", Rgb(0, 0, 255));

    // Assert
    assert_eq!(config.footer_items.len(), 3);
}

#[test]
fn test_box_config_with_width() {
    // Arrange & Act
    let config = BoxConfig::new("Title")
        .with_width(120);

    // Assert
    assert_eq!(config.width, 120);
}

#[test]
fn test_box_config_minimum_width() {
    // Arrange & Act
    let config = BoxConfig::new("Title")
        .with_width(5); // Below minimum

    // Assert
    assert_eq!(config.width, 10, "Width should be clamped to minimum of 10");
}

#[test]
fn test_box_config_with_border_color() {
    // Arrange & Act
    let border_color = Rgb(128, 128, 128);
    let config = BoxConfig::new("Title")
        .with_border_color(border_color);

    // Assert
    assert_eq!(config.border_color, border_color);
}

#[test]
fn test_box_config_with_spacing() {
    // Arrange & Act
    let config = BoxConfig::new("Title")
        .with_spacing(true);

    // Assert
    assert_eq!(config.add_spacing, true);
}

#[test]
fn test_box_config_builder_chain() {
    // Arrange & Act
    let config = BoxConfig::new("Main Title")
        .with_subtitle("Subtitle")
        .with_title_prefix(">_")
        .add_footer_item("key1", "value1", Rgb(0, 255, 0))
        .add_footer_item("key2", "value2", Rgb(0, 0, 255))
        .with_width(100)
        .with_border_color(Rgb(200, 200, 200))
        .with_spacing(true);

    // Assert
    assert_eq!(config.title, "Main Title");
    assert_eq!(config.subtitle, Some("Subtitle".to_string()));
    assert_eq!(config.title_prefix, Some(">_".to_string()));
    assert_eq!(config.footer_items.len(), 2);
    assert_eq!(config.width, 100);
    assert_eq!(config.border_color, Rgb(200, 200, 200));
    assert_eq!(config.add_spacing, true);
}

// Edge Cases

#[test]
fn test_box_config_empty_title() {
    // Arrange & Act
    let config = BoxConfig::new("");

    // Assert
    assert_eq!(config.title, "");
}

#[test]
fn test_box_config_long_title() {
    // Arrange
    let long_title = "A".repeat(200);

    // Act
    let config = BoxConfig::new(&long_title);

    // Assert
    assert_eq!(config.title.len(), 200);
}

#[test]
fn test_box_config_unicode_title() {
    // Arrange & Act
    let config = BoxConfig::new("🚀 Título com emoções 🎉");

    // Assert
    assert_eq!(config.title, "🚀 Título com emoções 🎉");
}

#[test]
fn test_box_config_empty_footer_value() {
    // Arrange & Act
    let config = BoxConfig::new("Title")
        .add_footer_item("label", "", Rgb(0, 255, 0));

    // Assert
    assert_eq!(config.footer_items[0].1, "");
}

#[test]
fn test_box_config_default_values() {
    // Arrange & Act
    let config = BoxConfig::new("Title");

    // Assert
    assert_eq!(config.border_color, Rgb(155, 114, 255)); // Default purple
    assert_eq!(config.add_spacing, true); // Default with spacing
}
