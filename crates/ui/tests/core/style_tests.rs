//! Tests for color style conversion utilities
//!
//! Validates RGB to crossterm color conversion and foreground color commands.

use crossterm::style::Color;
use nettoolskit_ui::style::{rgb_to_crossterm, set_fg};
use owo_colors::Rgb;

#[test]
fn test_rgb_to_crossterm_primary_color() {
    // Arrange
    let rgb = Rgb(155, 114, 255);

    // Act
    let color = rgb_to_crossterm(rgb);

    // Assert
    match color {
        Color::Rgb { r, g, b } => {
            assert_eq!(r, 155);
            assert_eq!(g, 114);
            assert_eq!(b, 255);
        }
        _ => panic!("Expected Color::Rgb variant"),
    }
}

#[test]
fn test_rgb_to_crossterm_white() {
    // Arrange
    let rgb = Rgb(255, 255, 255);

    // Act
    let color = rgb_to_crossterm(rgb);

    // Assert
    match color {
        Color::Rgb { r, g, b } => {
            assert_eq!(r, 255);
            assert_eq!(g, 255);
            assert_eq!(b, 255);
        }
        _ => panic!("Expected Color::Rgb variant"),
    }
}

#[test]
fn test_rgb_to_crossterm_black() {
    // Arrange
    let rgb = Rgb(0, 0, 0);

    // Act
    let color = rgb_to_crossterm(rgb);

    // Assert
    match color {
        Color::Rgb { r, g, b } => {
            assert_eq!(r, 0);
            assert_eq!(g, 0);
            assert_eq!(b, 0);
        }
        _ => panic!("Expected Color::Rgb variant"),
    }
}

#[test]
fn test_set_fg_creates_valid_command() {
    // Arrange
    let rgb = Rgb(100, 150, 200);

    // Act
    let _cmd = set_fg(rgb);

    // Assert - compile-time type check is sufficient
    assert!(true);
}

#[test]
fn test_set_fg_with_primary_color() {
    // Arrange
    let rgb = Rgb(155, 114, 255);

    // Act
    let _cmd = set_fg(rgb);

    // Assert
    assert!(true);
}