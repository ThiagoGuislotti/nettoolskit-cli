//! Repository workflow integration tests.

use nettoolskit_orchestrator::{
    execute_repo_workflow, parse_repo_workflow_payload, validate_repo_workflow_request,
    RepoWorkflowPolicy,
};
use std::path::PathBuf;

fn test_policy() -> RepoWorkflowPolicy {
    RepoWorkflowPolicy {
        enabled: true,
        allowed_hosts: vec!["github.com".to_string(), "local".to_string()],
        allowed_command_prefixes: vec!["cargo test".to_string(), "git status".to_string()],
        allow_push: false,
        allow_pull_request: false,
        workspace_root: PathBuf::from(".temp/repo-workflow-tests"),
    }
}

#[test]
fn parse_repo_workflow_payload_json_roundtrip_succeeds() {
    let payload = r#"{"repo":"https://github.com/acme/demo.git","branch":"feature/integration","command":"cargo test","dry_run":true}"#;
    let parsed = parse_repo_workflow_payload(payload).expect("json payload should parse");
    assert_eq!(parsed.repo_url, "https://github.com/acme/demo.git");
    assert_eq!(parsed.branch_name, "feature/integration");
    assert_eq!(parsed.command, "cargo test");
    assert!(parsed.dry_run);
}

#[test]
fn validate_repo_workflow_request_rejects_unknown_command_prefix() {
    let request = parse_repo_workflow_payload(
        "repo=https://github.com/acme/demo.git;branch=feature/a;command=npm install;dry_run=true",
    )
    .expect("payload should parse");
    let result = validate_repo_workflow_request(&request, &test_policy());
    assert!(result.is_err(), "unexpected command prefix must be denied");
}

#[test]
fn execute_repo_workflow_returns_dry_run_plan() {
    let request = parse_repo_workflow_payload(
        "repo=https://github.com/acme/demo.git;branch=feature/a;command=cargo test;dry_run=true",
    )
    .expect("payload should parse");
    let result = execute_repo_workflow(&request, &test_policy()).expect("dry-run should pass");
    assert!(!result.executed);
    assert!(result.summary.contains("Dry-run"));
    assert!(result
        .plan
        .steps
        .iter()
        .any(|step| step.contains("git clone")));
}
