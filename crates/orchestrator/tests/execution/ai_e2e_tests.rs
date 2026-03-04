//! AI end-to-end style integration tests.
//!
//! These tests exercise the public orchestrator entry points to validate
//! `/ai` command routing, safety guards, and default mock-provider behavior.

use nettoolskit_orchestrator::{process_command, process_text, ExitStatus};

#[tokio::test]
async fn e2e_ai_plan_command_succeeds_with_default_mock_provider() {
    let result = process_command("/ai plan implement cache invalidation strategy").await;
    assert_eq!(result, ExitStatus::Success);
}

#[tokio::test]
async fn e2e_ai_apply_dry_run_command_succeeds_without_mutation() {
    let result = process_command("/ai apply --dry-run update service layer").await;
    assert_eq!(result, ExitStatus::Success);
}

#[tokio::test]
async fn e2e_ai_apply_without_safety_flag_is_blocked() {
    let result = process_command("/ai apply update service layer").await;
    assert_eq!(
        result,
        ExitStatus::Error,
        "AI apply must be blocked when neither --dry-run nor explicit approval is provided"
    );
}

#[tokio::test]
async fn e2e_ai_text_routing_alias_reaches_ai_flow() {
    let result = process_text("assistant explain retry policy").await;
    assert_eq!(result, ExitStatus::Success);
}
