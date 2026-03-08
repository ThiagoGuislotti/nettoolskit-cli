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
fn test_main_action_ai_variant() {
    // Arrange
    let action = MainAction::Ai;

    // Act
    let is_ai = matches!(action, MainAction::Ai);

    // Assert
    assert!(is_ai, "MainAction::Ai should match Ai variant");
}

#[test]
fn test_main_action_task_variant() {
    // Arrange
    let action = MainAction::Task;

    // Act
    let is_task = matches!(action, MainAction::Task);

    // Assert
    assert!(is_task, "MainAction::Task should match Task variant");
}

#[test]
fn test_main_action_config_variant() {
    // Arrange
    let action = MainAction::Config;

    // Act
    let is_config = matches!(action, MainAction::Config);

    // Assert
    assert!(is_config, "MainAction::Config should match Config variant");
}

#[test]
fn test_main_action_clear_variant() {
    // Arrange
    let action = MainAction::Clear;

    // Act
    let is_clear = matches!(action, MainAction::Clear);

    // Assert
    assert!(is_clear, "MainAction::Clear should match Clear variant");
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
fn test_main_action_slash_static_ai() {
    // Arrange
    let action = MainAction::Ai;

    // Act
    let slash_cmd = action.slash_static();

    // Assert
    assert_eq!(slash_cmd, "/ai", "Ai should produce /ai command");
}

#[test]
fn test_main_action_slash_static_task() {
    // Arrange
    let action = MainAction::Task;

    // Act
    let slash_cmd = action.slash_static();

    // Assert
    assert_eq!(slash_cmd, "/task", "Task should produce /task command");
}

#[test]
fn test_main_action_slash_static_config() {
    // Arrange
    let action = MainAction::Config;

    // Act
    let slash_cmd = action.slash_static();

    // Assert
    assert_eq!(
        slash_cmd, "/config",
        "Config should produce /config command"
    );
}

#[test]
fn test_main_action_slash_static_clear() {
    // Arrange
    let action = MainAction::Clear;

    // Act
    let slash_cmd = action.slash_static();

    // Assert
    assert_eq!(slash_cmd, "/clear", "Clear should produce /clear command");
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
        MainAction::Ai,
        MainAction::Task,
        MainAction::Config,
        MainAction::Clear,
        MainAction::Quit,
    ];

    // Act & Assert
    for action in actions {
        let matched = match action {
            MainAction::Help => true,
            MainAction::Manifest => true,
            MainAction::Ai => true,
            MainAction::Task => true,
            MainAction::Config => true,
            MainAction::Clear => true,
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
