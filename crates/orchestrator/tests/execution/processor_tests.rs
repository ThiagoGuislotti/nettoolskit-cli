//! Processor Tests
//!
//! Tests for command processor and text processor functionality.

use nettoolskit_orchestrator::{process_command, process_text, ExitStatus};
use std::fs;
use tempfile::tempdir;

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

#[tokio::test]
async fn test_process_translate_missing_arguments() {
    let result = process_command("/translate").await;
    assert_eq!(
        result,
        ExitStatus::Error,
        "Translate without required args should fail"
    );
}

#[tokio::test]
async fn test_process_translate_with_valid_args_executes_handler() {
    let dir = tempdir().expect("failed to create temporary directory");
    let input_path = dir.path().join("sample.cs.hbs");
    fs::write(&input_path, "public class {{class_name}} {}")
        .expect("failed to write input template");

    let command = format!(
        "/translate --from dotnet --to rust {}",
        input_path.display()
    );
    let result = process_command(&command).await;

    assert_eq!(
        result,
        ExitStatus::Success,
        "Translate with valid args should succeed"
    );

    let output_path = dir.path().join("sample.rs");
    assert!(
        output_path.exists(),
        "Expected translated output file at {}",
        output_path.display()
    );
}

// ─── Text Processing Tests ────────────────────────────────────────────────

#[tokio::test]
async fn test_process_text_empty_input() {
    let result = process_text("").await;
    assert_eq!(
        result,
        ExitStatus::Success,
        "Empty text should succeed silently"
    );
}

#[tokio::test]
async fn test_process_text_whitespace_only() {
    let result = process_text("   ").await;
    assert_eq!(
        result,
        ExitStatus::Success,
        "Whitespace-only text should succeed silently"
    );
}

#[tokio::test]
async fn test_process_text_tab_and_newline() {
    let result = process_text("\t\n  \r\n").await;
    assert_eq!(
        result,
        ExitStatus::Success,
        "Tab/newline whitespace should succeed silently"
    );
}

#[tokio::test]
async fn test_process_text_regular_input() {
    let result = process_text("hello world").await;
    assert_eq!(
        result,
        ExitStatus::Success,
        "Regular text should succeed with hint"
    );
}

#[tokio::test]
async fn test_process_text_routes_help_alias() {
    let result = process_text("ajuda").await;
    assert_eq!(
        result,
        ExitStatus::Success,
        "Help alias should route to /help command"
    );
}

#[tokio::test]
async fn test_process_text_routes_translate_without_slash() {
    let dir = tempdir().expect("failed to create temporary directory");
    let input_path = dir.path().join("text-mode.cs.hbs");
    fs::write(&input_path, "public class {{class_name}} {}")
        .expect("failed to write input template");

    let command = format!("translate --from dotnet --to rust {}", input_path.display());
    let result = process_text(&command).await;

    assert_eq!(
        result,
        ExitStatus::Success,
        "Free-text translate should route and succeed"
    );

    let output_path = dir.path().join("text-mode.rs");
    assert!(
        output_path.exists(),
        "Expected translated output file at {}",
        output_path.display()
    );
}

#[tokio::test]
async fn test_process_text_special_characters() {
    let result = process_text("café 日本語 @#$%").await;
    assert_eq!(
        result,
        ExitStatus::Success,
        "Special characters should be handled gracefully"
    );
}

#[tokio::test]
async fn test_process_text_long_input() {
    let long = "a".repeat(1000);
    let result = process_text(&long).await;
    assert_eq!(
        result,
        ExitStatus::Success,
        "Long input should be handled gracefully"
    );
}
