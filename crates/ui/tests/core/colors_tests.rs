//! Tests for color constants
//!
//! Validates the color palette constants used throughout the UI.

use nettoolskit_ui::{GRAY_COLOR, PRIMARY_COLOR, SECONDARY_COLOR, WHITE_COLOR};

#[test]
fn test_primary_color_rgb_values() {
    // Arrange
    let expected = (155, 114, 255);

    // Act
    let color = PRIMARY_COLOR;

    // Assert
    assert_eq!(color.0, expected.0);
    assert_eq!(color.1, expected.1);
    assert_eq!(color.2, expected.2);
}

#[test]
fn test_secondary_color_rgb_values() {
    // Arrange
    let expected = (204, 185, 254);

    // Act
    let color = SECONDARY_COLOR;

    // Assert
    assert_eq!(color.0, expected.0);
    assert_eq!(color.1, expected.1);
    assert_eq!(color.2, expected.2);
}

#[test]
fn test_white_color_rgb_values() {
    // Arrange
    let expected = (255, 255, 255);

    // Act
    let color = WHITE_COLOR;

    // Assert
    assert_eq!(color.0, expected.0);
    assert_eq!(color.1, expected.1);
    assert_eq!(color.2, expected.2);
}

#[test]
fn test_gray_color_rgb_values() {
    // Arrange
    let expected = (128, 128, 128);

    // Act
    let color = GRAY_COLOR;

    // Assert
    assert_eq!(color.0, expected.0);
    assert_eq!(color.1, expected.1);
    assert_eq!(color.2, expected.2);
}
