//! Tests for Command enum

use nettoolskit_management::Command;
use strum::IntoEnumIterator;

#[test]
fn test_command_count() {
    // Arrange
    let expected_count = 6;

    // Act
    let actual_count = Command::iter().count();

    // Assert
    assert_eq!(actual_count, expected_count);
}

#[test]
fn test_list_command() {
    // Arrange
    let command = Command::List;

    // Act
    let slash_command = command.slash();

    // Assert
    assert_eq!(slash_command, "/list");
}

#[test]
fn test_check_command() {
    // Arrange
    let command = Command::Check;

    // Act
    let slash_command = command.slash();

    // Assert
    assert_eq!(slash_command, "/check");
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
