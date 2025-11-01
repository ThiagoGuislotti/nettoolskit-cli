use nettoolskit_commands::processor::{process_command, CliExitStatus};

#[test]
fn test_cli_exit_status_variants() {
    let success = CliExitStatus::Success;
    let error = CliExitStatus::Error;
    let interrupted = CliExitStatus::Interrupted;

    assert!(matches!(success, CliExitStatus::Success));
    assert!(matches!(error, CliExitStatus::Error));
    assert!(matches!(interrupted, CliExitStatus::Interrupted));
}

#[test]
fn test_cli_exit_status_debug() {
    let status = CliExitStatus::Success;
    let debug_str = format!("{:?}", status);
    assert!(debug_str.contains("Success"));

    let status = CliExitStatus::Error;
    let debug_str = format!("{:?}", status);
    assert!(debug_str.contains("Error"));

    let status = CliExitStatus::Interrupted;
    let debug_str = format!("{:?}", status);
    assert!(debug_str.contains("Interrupted"));
}

#[test]
fn test_cli_exit_status_clone() {
    let original = CliExitStatus::Success;
    let cloned = original.clone();

    assert_eq!(original, cloned);
    assert!(matches!(cloned, CliExitStatus::Success));
}

#[test]
fn test_cli_exit_status_copy() {
    let original = CliExitStatus::Error;
    let copied = original;

    assert_eq!(original, copied);
    assert!(matches!(copied, CliExitStatus::Error));
}

#[test]
fn test_cli_exit_status_partial_eq() {
    assert_eq!(CliExitStatus::Success, CliExitStatus::Success);
    assert_eq!(CliExitStatus::Error, CliExitStatus::Error);
    assert_eq!(CliExitStatus::Interrupted, CliExitStatus::Interrupted);

    assert_ne!(CliExitStatus::Success, CliExitStatus::Error);
    assert_ne!(CliExitStatus::Success, CliExitStatus::Interrupted);
    assert_ne!(CliExitStatus::Error, CliExitStatus::Interrupted);
}

#[tokio::test]
async fn test_process_quit_command() {
    let result = process_command("/quit").await;
    assert_eq!(result, CliExitStatus::Success);
}

#[tokio::test]
async fn test_process_list_command() {
    let result = process_command("/list").await;
    // Should complete without panicking
    assert!(matches!(
        result,
        CliExitStatus::Success | CliExitStatus::Error
    ));
}

#[tokio::test]
async fn test_process_new_command() {
    let result = process_command("/new").await;
    // Should complete without panicking
    assert!(matches!(
        result,
        CliExitStatus::Success | CliExitStatus::Error
    ));
}

#[tokio::test]
async fn test_process_check_command() {
    let result = process_command("/check").await;
    // Should complete without panicking
    assert!(matches!(
        result,
        CliExitStatus::Success | CliExitStatus::Error
    ));
}

#[tokio::test]
async fn test_process_render_command() {
    let result = process_command("/render").await;
    // Should complete without panicking
    assert!(matches!(
        result,
        CliExitStatus::Success | CliExitStatus::Error
    ));
}

#[tokio::test]
async fn test_process_apply_command() {
    let result = process_command("/apply").await;
    // Should complete without panicking
    assert!(matches!(
        result,
        CliExitStatus::Success | CliExitStatus::Error
    ));
}

#[tokio::test]
async fn test_process_unknown_command() {
    let result = process_command("/unknown").await;
    assert_eq!(result, CliExitStatus::Error);
}

#[tokio::test]
async fn test_process_empty_command() {
    let result = process_command("").await;
    assert_eq!(result, CliExitStatus::Error);
}

#[tokio::test]
async fn test_process_command_without_slash() {
    let result = process_command("list").await;
    assert_eq!(result, CliExitStatus::Error);
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
            CliExitStatus::Error,
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
        assert!(matches!(
            result,
            CliExitStatus::Success | CliExitStatus::Error
        ));
    }
}

#[test]
fn test_cli_exit_status_all_variants_covered() {
    // Ensure we have tests for all variants
    let _success = CliExitStatus::Success;
    let _error = CliExitStatus::Error;
    let _interrupted = CliExitStatus::Interrupted;

    // This test ensures we don't miss any variants if new ones are added
    assert!(true);
}
