//! Approval gateway for AI-driven side-effect operations.
//!
//! This module enforces explicit approval for mutating actions and writes a
//! local audit trail for approved/denied decisions.

use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Side-effect action kinds that require approval control.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalActionKind {
    /// Execute a shell/CLI command.
    CommandExecution,
    /// Write/update file content.
    FileWrite,
}

impl ApprovalActionKind {
    /// Return stable action identifier string.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::CommandExecution => "command_execution",
            Self::FileWrite => "file_write",
        }
    }
}

/// Approval request payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApprovalRequest {
    /// Action classification.
    pub action: ApprovalActionKind,
    /// Human-readable target (path/command/workspace scope).
    pub target: String,
    /// Operation reason/context.
    pub reason: String,
    /// Whether operation is a dry-run (non-mutating).
    pub dry_run: bool,
    /// Explicit user-confirmation signal (e.g. `--approve-write`).
    pub explicit_approval: bool,
    /// Request source identifier.
    pub source: String,
}

impl ApprovalRequest {
    /// Build command execution approval request.
    #[must_use]
    pub fn command_execution(
        target: impl Into<String>,
        reason: impl Into<String>,
        dry_run: bool,
        explicit_approval: bool,
        source: impl Into<String>,
    ) -> Self {
        Self {
            action: ApprovalActionKind::CommandExecution,
            target: target.into(),
            reason: reason.into(),
            dry_run,
            explicit_approval,
            source: source.into(),
        }
    }

    /// Build file write approval request.
    #[must_use]
    pub fn file_write(
        target: impl Into<String>,
        reason: impl Into<String>,
        dry_run: bool,
        explicit_approval: bool,
        source: impl Into<String>,
    ) -> Self {
        Self {
            action: ApprovalActionKind::FileWrite,
            target: target.into(),
            reason: reason.into(),
            dry_run,
            explicit_approval,
            source: source.into(),
        }
    }
}

/// Approval decision outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApprovalDecision {
    /// Operation is allowed.
    Approved {
        /// Human-readable reason for approval.
        reason: String,
    },
    /// Operation is denied.
    Denied {
        /// Human-readable reason for denial.
        reason: String,
    },
}

impl ApprovalDecision {
    /// Returns whether decision is approved.
    #[must_use]
    pub fn is_approved(&self) -> bool {
        matches!(self, Self::Approved { .. })
    }

    /// Returns decision reason.
    #[must_use]
    pub fn reason(&self) -> &str {
        match self {
            Self::Approved { reason } | Self::Denied { reason } => reason,
        }
    }

    fn status(&self) -> &'static str {
        match self {
            Self::Approved { .. } => "approved",
            Self::Denied { .. } => "denied",
        }
    }
}

/// Local audit record serialized as JSONL.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ApprovalAuditRecord {
    timestamp_ms: u128,
    action: String,
    target: String,
    reason: String,
    dry_run: bool,
    explicit_approval: bool,
    decision: String,
    decision_reason: String,
    source: String,
}

/// Evaluate an approval request without side effects.
#[must_use]
pub fn evaluate_approval(request: &ApprovalRequest) -> ApprovalDecision {
    if request.dry_run {
        return ApprovalDecision::Approved {
            reason: "dry-run operation auto-approved".to_string(),
        };
    }

    if request.explicit_approval {
        return ApprovalDecision::Approved {
            reason: "explicit approval provided".to_string(),
        };
    }

    ApprovalDecision::Denied {
        reason: "missing explicit approval for mutating action".to_string(),
    }
}

/// Evaluate approval and append local audit entry.
#[must_use]
pub fn request_approval(request: ApprovalRequest) -> ApprovalDecision {
    let decision = evaluate_approval(&request);
    let _ = append_approval_audit(&request, &decision);
    decision
}

fn append_approval_audit(request: &ApprovalRequest, decision: &ApprovalDecision) -> io::Result<()> {
    let Some(path) = resolve_approval_audit_path() else {
        return Ok(());
    };

    append_approval_audit_to(&path, request, decision)
}

fn append_approval_audit_to(
    path: &Path,
    request: &ApprovalRequest,
    decision: &ApprovalDecision,
) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    let record = ApprovalAuditRecord {
        timestamp_ms: now_unix_millis(),
        action: request.action.as_str().to_string(),
        target: request.target.clone(),
        reason: request.reason.clone(),
        dry_run: request.dry_run,
        explicit_approval: request.explicit_approval,
        decision: decision.status().to_string(),
        decision_reason: decision.reason().to_string(),
        source: request.source.clone(),
    };

    let line = serde_json::to_string(&record).map_err(io::Error::other)?;
    writeln!(file, "{line}")?;
    Ok(())
}

fn resolve_approval_audit_path() -> Option<PathBuf> {
    if let Ok(path_override) = std::env::var("NTK_AI_APPROVAL_AUDIT_PATH") {
        let trimmed = path_override.trim();
        if !trimmed.is_empty() {
            return Some(PathBuf::from(trimmed));
        }
    }

    if let Ok(current_dir) = std::env::current_dir() {
        return Some(
            current_dir
                .join(".temp")
                .join("ai")
                .join("approval-audit.jsonl"),
        );
    }

    None
}

fn now_unix_millis() -> u128 {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => duration.as_millis(),
        Err(_) => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evaluate_approval_dry_run_is_auto_approved() {
        let request =
            ApprovalRequest::file_write("workspace", "preview patch", true, false, "test-suite");
        let decision = evaluate_approval(&request);
        assert!(decision.is_approved());
        assert!(decision.reason().contains("dry-run"));
    }

    #[test]
    fn evaluate_approval_mutating_without_explicit_confirmation_is_denied() {
        let request = ApprovalRequest::command_execution(
            "cargo build",
            "execute build",
            false,
            false,
            "test-suite",
        );
        let decision = evaluate_approval(&request);
        assert!(!decision.is_approved());
        assert!(decision.reason().contains("missing explicit approval"));
    }

    #[test]
    fn evaluate_approval_mutating_with_explicit_confirmation_is_approved() {
        let request = ApprovalRequest::file_write(
            "src/main.rs",
            "apply generated patch",
            false,
            true,
            "test-suite",
        );
        let decision = evaluate_approval(&request);
        assert!(decision.is_approved());
        assert!(decision.reason().contains("explicit approval"));
    }

    #[test]
    fn append_approval_audit_to_writes_jsonl_entry() {
        let temp = tempfile::tempdir().expect("tempdir should be created");
        let file_path = temp.path().join("approval-audit.jsonl");

        let request =
            ApprovalRequest::command_execution("echo hi", "command run", false, true, "tests");
        let decision = ApprovalDecision::Approved {
            reason: "explicit approval provided".to_string(),
        };

        append_approval_audit_to(&file_path, &request, &decision)
            .expect("audit entry should be written");

        let content = fs::read_to_string(file_path).expect("audit file should be readable");
        assert!(content.contains("\"action\":\"command_execution\""));
        assert!(content.contains("\"decision\":\"approved\""));
        assert!(content.contains("\"target\":\"echo hi\""));
    }
}
