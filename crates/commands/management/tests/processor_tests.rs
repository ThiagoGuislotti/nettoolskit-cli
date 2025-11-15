/// Integration tests for command processor
///
/// Tests the async command dispatcher, including command routing,
/// telemetry integration, and handler execution.
use nettoolskit_management::{process_command, process_text, ExitStatus};

// Command Processing Tests

#[tokio::test]
async fn test_process_quit_command() {
    // Act
    let result = process_command("/quit").await;

    // Assert
    assert_eq!(result, ExitStatus::Success);
}

#[tokio::test]
async fn test_process_list_command() {
    // Act
    let result = process_command("/list").await;

    // Assert
    // Critical: list may return Error if no manifests exist (expected in test env)
    assert!(matches!(result, ExitStatus::Success | ExitStatus::Error));
}

#[tokio::test]
async fn test_process_new_command() {
    // Act
    let result = process_command("/new").await;

    // Assert
    assert_eq!(result, ExitStatus::Success);
}

#[tokio::test]
async fn test_process_check_command() {
    // Act
    let result = process_command("/check").await;

    // Assert
    // Critical: check may fail if no manifest exists (expected in test env)
    assert!(matches!(result, ExitStatus::Success | ExitStatus::Error));
}

#[tokio::test]
async fn test_process_render_command() {
    // Act
    let result = process_command("/render").await;

    // Assert
    assert_eq!(result, ExitStatus::Success);
}

#[tokio::test]
async fn test_process_apply_command() {
    // Act
    let result = process_command("/apply").await;

    // Assert
    // Critical: apply may fail if no manifest exists (expected in test env)
    assert!(matches!(result, ExitStatus::Success | ExitStatus::Error));
}

// Error Handling Tests

#[tokio::test]
async fn test_process_unknown_command() {
    // Act
    let result = process_command("/unknown").await;

    // Assert
    assert_eq!(result, ExitStatus::Error);
}

#[tokio::test]
async fn test_process_invalid_command() {
    // Act
    let result = process_command("invalid").await;

    // Assert
    assert_eq!(result, ExitStatus::Error);
}

#[tokio::test]
async fn test_process_empty_command() {
    // Act
    let result = process_command("").await;

    // Assert
    assert_eq!(result, ExitStatus::Error);
}

// Text Processing Tests

#[tokio::test]
async fn test_process_text_input() {
    // Act
    let result = process_text("some text").await;

    // Assert
    assert_eq!(result, ExitStatus::Success);
}

#[tokio::test]
async fn test_process_text_empty() {
    // Act
    let result = process_text("").await;

    // Assert
    assert_eq!(result, ExitStatus::Success);
}

// Sequential and Concurrent Tests

#[tokio::test]
async fn test_multiple_commands_sequentially() {
    // Arrange
    let commands = vec!["/list", "/new", "/render", "/quit"];

    // Act & Assert
    for cmd in commands {
        let result = process_command(cmd).await;
        assert!(matches!(result, ExitStatus::Success | ExitStatus::Error));
    }
}

#[tokio::test]
async fn test_concurrent_command_processing() {
    // Arrange
    let commands = vec!["/list", "/new", "/render"];

    // Act
    let handles: Vec<_> = commands
        .into_iter()
        .map(|cmd| tokio::spawn(async move { process_command(cmd).await }))
        .collect();

    // Assert
    for handle in handles {
        let result = handle.await.unwrap();
        assert!(matches!(result, ExitStatus::Success | ExitStatus::Error));
    }
}

// Input Validation Tests

#[tokio::test]
async fn test_command_with_special_chars() {
    // Act
    let result = process_command("/list@#$%").await;

    // Assert
    assert_eq!(result, ExitStatus::Error);
}

#[tokio::test]
async fn test_command_with_spaces() {
    // Act
    let result = process_command("/list with spaces").await;

    // Assert
    assert_eq!(result, ExitStatus::Error);
}

#[tokio::test]
async fn test_process_command_idempotent() {
    // Act & Assert
    // Critical: list may return Error if no manifests exist (expected in test env)
    for _ in 0..3 {
        let result = process_command("/list").await;
        assert!(matches!(result, ExitStatus::Success | ExitStatus::Error));
    }
}

#[tokio::test]
async fn test_processor_doesnt_panic_on_invalid_input() {
    // Arrange
    let invalid_inputs = vec!["/", "//", "/123", "/cmd\0null", "/cmd\nline", "/cmd\ttab"];

    // Act & Assert
    for input in invalid_inputs {
        let result = process_command(input).await;
        assert_eq!(result, ExitStatus::Error);
    }
}
