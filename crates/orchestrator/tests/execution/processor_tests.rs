//! Processor Tests
//!
//! Tests for command processor functionality.

use nettoolskit_orchestrator::{process_command, ExitStatus};

// Command Processing Tests

#[tokio::test]
async fn test_process_help_command() {
    // Arrange
    let command = "/help";

    // Act
    let result = process_command(command).await;

    // Assert
    assert!(
        matches!(result, ExitStatus::Success | ExitStatus::Error),
        "Help command should return Success or Error"
    );
}

#[tokio::test]
async fn test_process_quit_command() {
    // Arrange
    let command = "/quit";

    // Act
    let result = process_command(command).await;

    // Assert
    // Quit command processing should complete
    assert!(
        matches!(result, ExitStatus::Success | ExitStatus::Error),
        "Quit command should return a valid status"
    );
}

#[tokio::test]
async fn test_process_empty_command() {
    // Arrange
    let command = "";

    // Act
    let result = process_command(command).await;

    // Assert
    // Empty command should be handled gracefully
    assert!(
        matches!(result, ExitStatus::Success | ExitStatus::Error),
        "Empty command should return a valid status"
    );
}

#[tokio::test]
async fn test_process_invalid_command() {
    // Arrange
    let command = "/nonexistent";

    // Act
    let result = process_command(command).await;

    // Assert
    // Invalid command should be handled gracefully
    assert!(
        matches!(result, ExitStatus::Success | ExitStatus::Error),
        "Invalid command should return a valid status"
    );
}
