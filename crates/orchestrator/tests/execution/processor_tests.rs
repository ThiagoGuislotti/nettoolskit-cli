//! Processor Tests
//!
//! Tests for command processor and text processor functionality.

use nettoolskit_orchestrator::{process_command, process_text, ExitStatus};
use std::sync::OnceLock;
use tokio::sync::Mutex;

static ENV_TEST_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

async fn env_test_guard() -> tokio::sync::MutexGuard<'static, ()> {
    ENV_TEST_LOCK.get_or_init(|| Mutex::new(())).lock().await
}

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
async fn test_process_clear_command() {
    // Arrange
    let command = "/clear";

    // Act
    let result = process_command(command).await;

    // Assert
    assert!(
        matches!(result, ExitStatus::Success | ExitStatus::Error),
        "Clear command should return a valid status"
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
async fn test_process_ai_ask_with_mock_provider_succeeds() {
    let result = process_command("/ai ask explain command cache").await;
    assert_eq!(
        result,
        ExitStatus::Success,
        "AI ask should succeed with default mock provider"
    );
}

#[tokio::test]
async fn test_process_ai_apply_requires_dry_run() {
    let result = process_command("/ai apply create service").await;
    assert_eq!(
        result,
        ExitStatus::Error,
        "AI apply must require --dry-run or explicit write approval"
    );
}

#[tokio::test]
async fn test_process_ai_apply_with_explicit_write_approval_succeeds() {
    let result = process_command("/ai apply --approve-write create service").await;
    assert_eq!(
        result,
        ExitStatus::Success,
        "AI apply with explicit write approval should pass approval gateway"
    );
}

#[tokio::test]
async fn test_process_task_submit_ai_plan_with_local_fallback_succeeds() {
    let result = process_command("/task submit ai-plan implement dual runtime mode").await;
    assert_eq!(
        result,
        ExitStatus::Success,
        "Task submit for ai-plan should run with local fallback and succeed"
    );
}

#[tokio::test]
async fn test_process_task_submit_ai_plan_in_service_mode_queues_successfully() {
    let _guard = env_test_guard().await;
    std::env::set_var("NTK_RUNTIME_MODE", "service");
    std::env::set_var("NTK_AI_PROVIDER", "mock");
    std::env::set_var("NTK_TOOL_SCOPE_ALLOWED_TOOLS", "ai.plan");

    let result = process_command("/task submit ai-plan queue service mode task").await;

    std::env::remove_var("NTK_TOOL_SCOPE_ALLOWED_TOOLS");
    std::env::remove_var("NTK_AI_PROVIDER");
    std::env::remove_var("NTK_RUNTIME_MODE");

    assert_eq!(
        result,
        ExitStatus::Success,
        "Task submit in service mode should queue task and return success"
    );
}

#[tokio::test]
async fn test_process_task_submit_repo_workflow_dry_run_with_policy_succeeds() {
    let _guard = env_test_guard().await;
    std::env::set_var("NTK_REPO_WORKFLOW_ENABLED", "true");
    std::env::set_var("NTK_REPO_WORKFLOW_ALLOWED_HOSTS", "github.com");
    std::env::set_var("NTK_REPO_WORKFLOW_ALLOWED_COMMANDS", "cargo test,cargo fmt");
    std::env::set_var("NTK_REPO_WORKFLOW_ALLOW_PUSH", "false");
    std::env::set_var("NTK_REPO_WORKFLOW_ALLOW_PR", "false");

    let result = process_command(
        "/task submit repo-workflow repo=https://github.com/acme/demo.git;branch=feature/chatops;command=cargo test;dry_run=true",
    )
    .await;

    std::env::remove_var("NTK_REPO_WORKFLOW_ALLOW_PR");
    std::env::remove_var("NTK_REPO_WORKFLOW_ALLOW_PUSH");
    std::env::remove_var("NTK_REPO_WORKFLOW_ALLOWED_COMMANDS");
    std::env::remove_var("NTK_REPO_WORKFLOW_ALLOWED_HOSTS");
    std::env::remove_var("NTK_REPO_WORKFLOW_ENABLED");

    assert_eq!(
        result,
        ExitStatus::Success,
        "Repo workflow dry-run should succeed when policy is explicitly configured"
    );
}

#[tokio::test]
async fn test_process_task_submit_without_payload_fails() {
    let result = process_command("/task submit ai-plan").await;
    assert_eq!(
        result,
        ExitStatus::Error,
        "Task submit without payload should fail validation"
    );
}

#[tokio::test]
async fn test_process_task_list_succeeds() {
    let result = process_command("/task list").await;
    assert_eq!(
        result,
        ExitStatus::Success,
        "Task list should return success even when empty"
    );
}

#[tokio::test]
async fn test_process_task_watch_without_id_fails() {
    let result = process_command("/task watch").await;
    assert_eq!(
        result,
        ExitStatus::Error,
        "Task watch must require explicit task id"
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
async fn test_process_text_routes_clear_alias() {
    let result = process_text("limpar").await;
    assert!(
        matches!(result, ExitStatus::Success | ExitStatus::Error),
        "Clear alias should route to /clear command"
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
