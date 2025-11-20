//! Tests for Command enum

use nettoolskit_management::Command;
use strum::IntoEnumIterator;

#[test]
fn test_command_count() {
    // Arrange
    let expected_count = 4;

    // Act
    let actual_count = Command::iter().count();

    // Assert
    assert_eq!(actual_count, expected_count);
}

#[test]
fn test_help_command() {
    // Arrange
    let command = Command::Help;

    // Act
    let slash_command = command.slash();

    // Assert
    assert_eq!(slash_command, "/help");
}

#[test]
fn test_manifest_command() {
    // Arrange
    let command = Command::Manifest;

    // Act
    let slash_command = command.slash();

    // Assert
    assert_eq!(slash_command, "/manifest");
}

#[test]
fn test_all_start_with_slash() {
    // Arrange
    let commands = Command::iter();

    // Act & Assert
    for cmd in commands {
        assert!(cmd.slash().starts_with('/'));
    }
}
