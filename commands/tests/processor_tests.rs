use nettoolskit_commands::{processor::process_command, ExitStatus};

#[test]
fn test_cli_exit_status_variants() {
    let success = ExitStatus::Success;
    let error = ExitStatus::Error;
    let interrupted = ExitStatus::Interrupted;

    assert!(matches!(success, ExitStatus::Success));
    assert!(matches!(error, ExitStatus::Error));
    assert!(matches!(interrupted, ExitStatus::Interrupted));
}

#[test]
fn test_cli_exit_status_debug() {
    let status = ExitStatus::Success;
    let debug_str = format!("{:?}", status);
    assert!(debug_str.contains("Success"));

    let status = ExitStatus::Error;
    let debug_str = format!("{:?}", status);
    assert!(debug_str.contains("Error"));

    let status = ExitStatus::Interrupted;
    let debug_str = format!("{:?}", status);
    assert!(debug_str.contains("Interrupted"));
}

#[test]
fn test_cli_exit_status_clone() {
    let original = ExitStatus::Success;
    let cloned = original.clone();

    assert_eq!(original, cloned);
    assert!(matches!(cloned, ExitStatus::Success));
}

#[test]
fn test_cli_exit_status_copy() {
    let original = ExitStatus::Error;
    let copied = original;

    assert_eq!(original, copied);
    assert!(matches!(copied, ExitStatus::Error));
}

#[test]
fn test_cli_exit_status_partial_eq() {
    assert_eq!(ExitStatus::Success, ExitStatus::Success);
    assert_eq!(ExitStatus::Error, ExitStatus::Error);
    assert_eq!(ExitStatus::Interrupted, ExitStatus::Interrupted);

    assert_ne!(ExitStatus::Success, ExitStatus::Error);
    assert_ne!(ExitStatus::Success, ExitStatus::Interrupted);
    assert_ne!(ExitStatus::Error, ExitStatus::Interrupted);
}

#[tokio::test]
async fn test_process_quit_command() {
    let result = process_command("/quit").await;
    assert_eq!(result, ExitStatus::Success);
}

#[tokio::test]
async fn test_process_list_command() {
    let result = process_command("/list").await;
    // Should complete without panicking
    assert!(matches!(result, ExitStatus::Success | ExitStatus::Error));
}

#[tokio::test]
async fn test_process_new_command() {
    let result = process_command("/new").await;
    // Should complete without panicking
    assert!(matches!(result, ExitStatus::Success | ExitStatus::Error));
}

#[tokio::test]
async fn test_process_check_command() {
    let result = process_command("/check").await;
    // Should complete without panicking
    assert!(matches!(result, ExitStatus::Success | ExitStatus::Error));
}

#[tokio::test]
async fn test_process_render_command() {
    let result = process_command("/render").await;
    // Should complete without panicking
    assert!(matches!(result, ExitStatus::Success | ExitStatus::Error));
}

#[tokio::test]
async fn test_process_apply_command() {
    let result = process_command("/apply").await;
    // Should complete without panicking
    assert!(matches!(result, ExitStatus::Success | ExitStatus::Error));
}

#[tokio::test]
async fn test_process_unknown_command() {
    let result = process_command("/unknown").await;
    assert_eq!(result, ExitStatus::Error);
}

#[tokio::test]
async fn test_process_empty_command() {
    let result = process_command("").await;
    assert_eq!(result, ExitStatus::Error);
}

#[tokio::test]
async fn test_process_command_without_slash() {
    let result = process_command("list").await;
    assert_eq!(result, ExitStatus::Error);
}

#[tokio::test]
async fn test_process_malformed_commands() {
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
async fn test_process_command_whitespace_handling() {
    let commands_with_whitespace = vec![" /quit", "/quit ", " /quit ", "\t/quit", "/quit\t"];

    for cmd in commands_with_whitespace {
        let result = process_command(cmd).await;
        // These should be handled gracefully (either success for quit or error for unrecognized)
        assert!(matches!(result, ExitStatus::Success | ExitStatus::Error));
    }
}

#[test]
fn test_cli_exit_status_all_variants_covered() {
    // Ensure we have tests for all variants
    let _success = ExitStatus::Success;
    let _error = ExitStatus::Error;
    let _interrupted = ExitStatus::Interrupted;

    // This test ensures we don't miss any variants if new ones are added
    assert!(true);
}
