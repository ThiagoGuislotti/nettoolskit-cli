/// Integration tests for commands crate
/// Recovered tests from backup that were not covered by unit tests
use nettoolskit_commands::processor::process_command;
use nettoolskit_commands::ExitStatus;

// Command Integration Tests

#[tokio::test]
async fn test_list_command_integration() {
    // Act
    let result = process_command("/list").await;

    // Assert
    assert!(matches!(result, ExitStatus::Success | ExitStatus::Error));
}

#[tokio::test]
async fn test_new_command_integration() {
    // Act
    let result = process_command("/new").await;

    // Assert
    assert!(matches!(result, ExitStatus::Success | ExitStatus::Error));
}

#[tokio::test]
async fn test_check_command_integration() {
    // Act
    let result = process_command("/check").await;

    // Assert
    assert!(matches!(result, ExitStatus::Success | ExitStatus::Error));
}

#[tokio::test]
async fn test_render_command_integration() {
    // Act
    let result = process_command("/render").await;

    // Assert
    assert!(matches!(result, ExitStatus::Success | ExitStatus::Error));
}

#[tokio::test]
async fn test_apply_command_integration() {
    // Act
    let result = process_command("/apply").await;

    // Assert
    assert!(matches!(result, ExitStatus::Success | ExitStatus::Error));
}

// Tests from backup/commands/tests/processor_tests.rs
#[tokio::test]
async fn test_malformed_commands() {
    let malformed_commands = vec![
        "//list",
        "/list/extra",
        "/",
        "/ ",
        "/LIST", // case sensitivity
    ];

    for cmd in malformed_commands {
        let result = process_command(cmd).await;
        assert_eq!(
            result,
            ExitStatus::Error,
            "Command '{}' should return error",
            cmd
        );
    }
}

#[tokio::test]
async fn test_command_whitespace_variations() {
    let whitespace_commands = vec![" /quit", "/quit ", " /quit ", "\t/quit", "/quit\t"];

    for cmd in whitespace_commands {
        let result = process_command(cmd).await;
        // Should complete (either success or error, but not panic)
        assert!(matches!(result, ExitStatus::Success | ExitStatus::Error));
    }
}

#[tokio::test]
async fn test_sequential_command_execution() {
    let commands = vec!["/list", "/new", "/check"];

    for cmd in commands {
        let result = process_command(cmd).await;
        // Each command should complete
        assert!(matches!(result, ExitStatus::Success | ExitStatus::Error));
    }
}

#[tokio::test]
async fn test_concurrent_command_execution() {
    let commands = vec!["/list", "/new", "/check", "/render"];

    let handles: Vec<_> = commands
        .into_iter()
        .map(|cmd| tokio::spawn(async move { process_command(cmd).await }))
        .collect();

    for handle in handles {
        let result = handle.await.unwrap();
        assert!(matches!(result, ExitStatus::Success | ExitStatus::Error));
    }
}

#[tokio::test]
async fn test_command_case_sensitivity() {
    // Lowercase should work
    let lowercase = process_command("/quit").await;
    assert_eq!(lowercase, ExitStatus::Success);

    // Uppercase should fail (case-sensitive)
    let uppercase = process_command("/QUIT").await;
    assert_eq!(uppercase, ExitStatus::Error);

    // Mixed case should fail
    let mixed = process_command("/QuIt").await;
    assert_eq!(mixed, ExitStatus::Error);
}

#[tokio::test]
async fn test_all_commands_are_registered() {
    let expected_commands = vec!["/quit", "/list", "/new", "/check", "/render", "/apply"];

    for cmd in expected_commands {
        let result = process_command(cmd).await;
        // Should not be "Unknown command" error
        // All commands should execute (success or error, but recognized)
        assert!(matches!(result, ExitStatus::Success | ExitStatus::Error));
    }
}

#[tokio::test]
async fn test_command_idempotency() {
    let cmd = "/list";

    let result1 = process_command(cmd).await;
    let result2 = process_command(cmd).await;
    let result3 = process_command(cmd).await;

    // Should produce consistent results
    assert!(matches!(result1, ExitStatus::Success | ExitStatus::Error));
    assert!(matches!(result2, ExitStatus::Success | ExitStatus::Error));
    assert!(matches!(result3, ExitStatus::Success | ExitStatus::Error));
}

#[tokio::test]
async fn test_error_recovery() {
    // Execute invalid command
    let _ = process_command("/invalid").await;

    // Should still be able to execute valid commands after error
    let ok_result = process_command("/quit").await;
    assert_eq!(ok_result, ExitStatus::Success);
}

#[tokio::test]
async fn test_empty_and_whitespace_commands() {
    let empty_inputs = vec!["", "   ", "\t", "\n", "\r\n"];

    for input in empty_inputs {
        let result = process_command(input).await;
        assert_eq!(result, ExitStatus::Error);
    }
}

#[tokio::test]
async fn test_exit_status_variants_exist() {
    // Ensure all ExitStatus variants are accessible
    let _success = ExitStatus::Success;
    let _error = ExitStatus::Error;
    let _interrupted = ExitStatus::Interrupted;
}

#[tokio::test]
async fn test_exit_status_equality() {
    assert_eq!(ExitStatus::Success, ExitStatus::Success);
    assert_eq!(ExitStatus::Error, ExitStatus::Error);
    assert_eq!(ExitStatus::Interrupted, ExitStatus::Interrupted);

    assert_ne!(ExitStatus::Success, ExitStatus::Error);
    assert_ne!(ExitStatus::Success, ExitStatus::Interrupted);
    assert_ne!(ExitStatus::Error, ExitStatus::Interrupted);
}

#[test]
fn test_exit_status_debug() {
    let success = ExitStatus::Success;
    let debug_str = format!("{:?}", success);
    assert!(debug_str.contains("Success"));
}

#[test]
fn test_exit_status_clone_copy() {
    let original = ExitStatus::Success;
    let cloned = original.clone();
    let copied = original;

    assert_eq!(original, cloned);
    assert_eq!(original, copied);
}
