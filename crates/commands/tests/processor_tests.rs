/// Integration tests for command processor
///
/// Tests the async command dispatcher, including command routing,
/// telemetry integration, and handler execution.

use nettoolskit_commands::{processor, ExitStatus};

#[tokio::test]
async fn test_process_quit_command() {
    let result = processor::process_command("/quit").await;
    assert_eq!(result, ExitStatus::Success);
}

#[tokio::test]
async fn test_process_list_command() {
    let result = processor::process_command("/list").await;
    // List command should return Success (even if not fully implemented)
    assert_eq!(result, ExitStatus::Success);
}

#[tokio::test]
async fn test_process_new_command() {
    let result = processor::process_command("/new").await;
    // New command should return Success (even if not fully implemented)
    assert_eq!(result, ExitStatus::Success);
}

#[tokio::test]
async fn test_process_check_command() {
    let result = processor::process_command("/check").await;
    // Check command will fail if no manifest exists, which is expected
    // We're testing that it executes without panicking
    assert!(matches!(result, ExitStatus::Success | ExitStatus::Error));
}

#[tokio::test]
async fn test_process_render_command() {
    let result = processor::process_command("/render").await;
    // Render command should return Success (even if not fully implemented)
    assert_eq!(result, ExitStatus::Success);
}

#[tokio::test]
async fn test_process_apply_command() {
    let result = processor::process_command("/apply").await;
    // Apply will fail if no manifest exists, which is expected in test env
    // We're testing that it executes without panicking
    assert!(matches!(result, ExitStatus::Success | ExitStatus::Error));
}

#[tokio::test]
async fn test_process_unknown_command() {
    let result = processor::process_command("/unknown").await;
    assert_eq!(result, ExitStatus::Error);
}

#[tokio::test]
async fn test_process_invalid_command() {
    let result = processor::process_command("invalid").await;
    assert_eq!(result, ExitStatus::Error);
}

#[tokio::test]
async fn test_process_empty_command() {
    let result = processor::process_command("").await;
    assert_eq!(result, ExitStatus::Error);
}

#[tokio::test]
async fn test_process_text_input() {
    let result = processor::process_text("some text").await;
    assert_eq!(result, ExitStatus::Success);
}

#[tokio::test]
async fn test_process_text_empty() {
    let result = processor::process_text("").await;
    assert_eq!(result, ExitStatus::Success);
}

#[tokio::test]
async fn test_multiple_commands_sequentially() {
    let commands = vec!["/list", "/new", "/render", "/quit"];

    for cmd in commands {
        let result = processor::process_command(cmd).await;
        // All should execute without panicking
        assert!(matches!(result, ExitStatus::Success | ExitStatus::Error));
    }
}

#[tokio::test]
async fn test_concurrent_command_processing() {
    let commands = vec!["/list", "/new", "/render"];

    let handles: Vec<_> = commands.into_iter()
        .map(|cmd| {
            tokio::spawn(async move {
                processor::process_command(cmd).await
            })
        })
        .collect();

    for handle in handles {
        let result = handle.await.unwrap();
        // All should complete successfully
        assert!(matches!(result, ExitStatus::Success | ExitStatus::Error));
    }
}

#[tokio::test]
async fn test_command_with_special_chars() {
    let result = processor::process_command("/list@#$%").await;
    assert_eq!(result, ExitStatus::Error);
}

#[tokio::test]
async fn test_command_with_spaces() {
    let result = processor::process_command("/list with spaces").await;
    assert_eq!(result, ExitStatus::Error);
}

#[tokio::test]
async fn test_process_command_idempotent() {
    // Execute same command multiple times
    for _ in 0..3 {
        let result = processor::process_command("/list").await;
        assert_eq!(result, ExitStatus::Success);
    }
}

#[tokio::test]
async fn test_processor_doesnt_panic_on_invalid_input() {
    let invalid_inputs = vec![
        "/",
        "//",
        "/123",
        "/cmd\0null",
        "/cmd\nline",
        "/cmd\ttab",
    ];

    for input in invalid_inputs {
        let result = processor::process_command(input).await;
        // Should not panic, should return Error
        assert_eq!(result, ExitStatus::Error);
    }
}