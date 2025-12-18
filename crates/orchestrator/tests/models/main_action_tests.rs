//! MainAction Model Tests
//!
//! Tests for MainAction enum and its slash command functionality.

use nettoolskit_core::CommandEntry;
use nettoolskit_orchestrator::MainAction;

// MainAction Variant Tests

#[test]
fn test_main_action_help_variant() {
    // Arrange
    let action = MainAction::Help;

    // Act
    let is_help = matches!(action, MainAction::Help);

    // Assert
    assert!(is_help, "MainAction::Help should match Help variant");
}

#[test]
fn test_main_action_manifest_variant() {
    // Arrange
    let action = MainAction::Manifest;

    // Act
    let is_manifest = matches!(action, MainAction::Manifest);

    // Assert
    assert!(
        is_manifest,
        "MainAction::Manifest should match Manifest variant"
    );
}

#[test]
fn test_main_action_translate_variant() {
    // Arrange
    let action = MainAction::Translate;

    // Act
    let is_translate = matches!(action, MainAction::Translate);

    // Assert
    assert!(
        is_translate,
        "MainAction::Translate should match Translate variant"
    );
}

#[test]
fn test_main_action_quit_variant() {
    // Arrange
    let action = MainAction::Quit;

    // Act
    let is_quit = matches!(action, MainAction::Quit);

    // Assert
    assert!(is_quit, "MainAction::Quit should match Quit variant");
}

// Slash Command Tests

#[test]
fn test_main_action_slash_static_help() {
    // Arrange
    let action = MainAction::Help;

    // Act
    let slash_cmd = action.slash_static();

    // Assert
    assert_eq!(slash_cmd, "/help", "Help should produce /help command");
}

#[test]
fn test_main_action_slash_static_manifest() {
    // Arrange
    let action = MainAction::Manifest;

    // Act
    let slash_cmd = action.slash_static();

    // Assert
    assert_eq!(
        slash_cmd, "/manifest",
        "Manifest should produce /manifest command"
    );
}

#[test]
fn test_main_action_slash_static_translate() {
    // Arrange
    let action = MainAction::Translate;

    // Act
    let slash_cmd = action.slash_static();

    // Assert
    assert_eq!(
        slash_cmd, "/translate",
        "Translate should produce /translate command"
    );
}

#[test]
fn test_main_action_slash_static_quit() {
    // Arrange
    let action = MainAction::Quit;

    // Act
    let slash_cmd = action.slash_static();

    // Assert
    assert_eq!(slash_cmd, "/quit", "Quit should produce /quit command");
}

// Pattern Matching Tests

#[test]
fn test_main_action_pattern_matching_exhaustive() {
    // Arrange
    let actions = vec![
        MainAction::Help,
        MainAction::Manifest,
        MainAction::Translate,
        MainAction::Quit,
    ];

    // Act & Assert
    for action in actions {
        let matched = match action {
            MainAction::Help => true,
            MainAction::Manifest => true,
            MainAction::Translate => true,
            MainAction::Quit => true,
        };
        assert!(matched, "All MainAction variants should be handled");
    }
}

// Trait Tests

#[test]
fn test_main_action_debug() {
    // Arrange
    let action = MainAction::Help;

    // Act
    let debug_str = format!("{:?}", action);

    // Assert
    assert!(
        debug_str.contains("Help"),
        "Debug output should contain 'Help'"
    );
}

#[test]
fn test_main_action_clone() {
    // Arrange
    let original = MainAction::Manifest;

    // Act
    let cloned = original;

    // Assert
    assert!(
        matches!(cloned, MainAction::Manifest),
        "Cloned action should match original"
    );
}

#[test]
fn test_main_action_equality() {
    // Arrange
    let help1 = MainAction::Help;
    let help2 = MainAction::Help;
    let quit = MainAction::Quit;

    // Act
    let same_variants_equal = help1 == help2;
    let different_variants_not_equal = help1 != quit;

    // Assert
    assert!(same_variants_equal, "Same variants should be equal");
    assert!(
        different_variants_not_equal,
        "Different variants should not be equal"
    );
}
