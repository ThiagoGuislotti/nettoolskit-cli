//! Tests for prompt rendering and formatting
//!
//! Validates prompt display functions and symbol constants.

use nettoolskit_ui::{get_prompt_string, get_prompt_symbol};

#[test]
fn test_prompt_symbol_is_angle_bracket() {
    // Arrange
    let expected = "> ";

    // Act
    let symbol = get_prompt_symbol();

    // Assert
    assert_eq!(symbol, expected);
}

#[test]
fn test_prompt_symbol_has_trailing_space() {
    // Arrange
    // (no setup needed)

    // Act
    let symbol = get_prompt_symbol();

    // Assert
    assert!(symbol.ends_with(' '));
    assert_eq!(symbol.len(), 2);
}

#[test]
fn test_prompt_string_contains_symbol() {
    // Arrange
    let symbol = get_prompt_symbol();

    // Act
    let prompt = get_prompt_string();

    // Assert
    assert!(prompt.contains(symbol));
}

#[test]
fn test_prompt_string_not_empty() {
    // Arrange
    // (no setup needed)

    // Act
    let prompt = get_prompt_string();

    // Assert
    assert!(!prompt.is_empty());
}

#[test]
fn test_prompt_string_contains_angle_bracket() {
    // Arrange
    // (no setup needed)

    // Act
    let prompt = get_prompt_string();

    // Assert
    assert!(prompt.contains('>'));
}