//! Runtime contracts shared by CLI and background-service execution modes.
//!
//! This module defines deterministic runtime mode resolution and common task
//! lifecycle contracts that can be reused by both local interactive execution
//! and background worker orchestration flows.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Environment variable used to override runtime mode selection.
pub const NTK_RUNTIME_MODE_ENV: &str = "NTK_RUNTIME_MODE";

/// Runtime operating mode for NetToolsKit execution.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum RuntimeMode {
    /// Interactive/local execution mode.
    #[default]
    Cli,
    /// Background service execution mode.
    Service,
}

impl RuntimeMode {
    /// Canonical lowercase label for display and serialization-like usage.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Cli => "cli",
            Self::Service => "service",
        }
    }
}

impl fmt::Display for RuntimeMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for RuntimeMode {
    type Err = &'static str;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().as_str() {
            "cli" => Ok(Self::Cli),
            "service" | "background" | "background-service" => Ok(Self::Service),
            _ => Err("runtime_mode must be one of: cli, service"),
        }
    }
}

/// Resolve runtime mode using deterministic precedence:
/// 1. environment override (if valid)
/// 2. file/default mode fallback
#[must_use]
pub fn resolve_runtime_mode(
    file_or_default: RuntimeMode,
    env_override: Option<&str>,
) -> RuntimeMode {
    env_override
        .and_then(|value| value.parse::<RuntimeMode>().ok())
        .unwrap_or(file_or_default)
}

/// Canonical task intent kinds shared between CLI and service runtimes.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum TaskIntentKind {
    /// Generic command execution task.
    #[default]
    CommandExecution,
    /// AI request task (`/ai ask` equivalent).
    AiAsk,
    /// AI planning task (`/ai plan` equivalent).
    AiPlan,
    /// AI explanation task (`/ai explain` equivalent).
    AiExplain,
    /// AI dry-run apply task (`/ai apply --dry-run` equivalent).
    AiApplyDryRun,
}

/// Shared task intent payload contract.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TaskIntent {
    /// Intent kind describing task semantics.
    pub kind: TaskIntentKind,
    /// Human-readable task title.
    pub title: String,
    /// Optional prompt/command payload.
    pub payload: String,
}

impl TaskIntent {
    /// Build a new task intent contract.
    #[must_use]
    pub fn new(kind: TaskIntentKind, title: impl Into<String>, payload: impl Into<String>) -> Self {
        Self {
            kind,
            title: title.into(),
            payload: payload.into(),
        }
    }
}

/// Shared execution lifecycle states for queued task processing.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum TaskExecutionStatus {
    /// Task accepted and queued, waiting for worker pickup.
    #[default]
    Queued,
    /// Task currently executing.
    Running,
    /// Task completed successfully.
    Succeeded,
    /// Task completed with failure.
    Failed,
    /// Task was cancelled before successful completion.
    Cancelled,
}

impl TaskExecutionStatus {
    /// Returns `true` when task state is terminal.
    #[must_use]
    pub const fn is_terminal(self) -> bool {
        matches!(self, Self::Succeeded | Self::Failed | Self::Cancelled)
    }

    /// Validate legal lifecycle transitions.
    #[must_use]
    pub fn can_transition_to(self, next: Self) -> bool {
        match (self, next) {
            (Self::Queued, Self::Running | Self::Cancelled) => true,
            (Self::Running, Self::Succeeded | Self::Failed | Self::Cancelled) => true,
            // Allow idempotent writes for status updates in persistence layers.
            (current, target) if current == target => true,
            _ => false,
        }
    }
}

/// Audit event contract for task lifecycle tracking.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TaskAuditEvent {
    /// Stable task identifier.
    pub task_id: String,
    /// Runtime mode where event was produced.
    pub runtime_mode: RuntimeMode,
    /// Status emitted by the event.
    pub status: TaskExecutionStatus,
    /// Human-readable event detail.
    pub message: String,
    /// UTC Unix timestamp in milliseconds.
    pub timestamp_unix_ms: u64,
}

impl TaskAuditEvent {
    /// Build a task audit event contract.
    #[must_use]
    pub fn new(
        task_id: impl Into<String>,
        runtime_mode: RuntimeMode,
        status: TaskExecutionStatus,
        message: impl Into<String>,
        timestamp_unix_ms: u64,
    ) -> Self {
        Self {
            task_id: task_id.into(),
            runtime_mode,
            status,
            message: message.into(),
            timestamp_unix_ms,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_mode_parsing_accepts_supported_values() {
        assert_eq!("cli".parse::<RuntimeMode>(), Ok(RuntimeMode::Cli));
        assert_eq!("service".parse::<RuntimeMode>(), Ok(RuntimeMode::Service));
        assert_eq!(
            "background-service".parse::<RuntimeMode>(),
            Ok(RuntimeMode::Service)
        );
    }

    #[test]
    fn runtime_mode_resolution_uses_env_override_when_valid() {
        let resolved = resolve_runtime_mode(RuntimeMode::Cli, Some("service"));
        assert_eq!(resolved, RuntimeMode::Service);
    }

    #[test]
    fn runtime_mode_resolution_falls_back_on_invalid_env() {
        let resolved = resolve_runtime_mode(RuntimeMode::Service, Some("invalid"));
        assert_eq!(resolved, RuntimeMode::Service);
    }

    #[test]
    fn task_execution_status_transition_rules_are_enforced() {
        assert!(TaskExecutionStatus::Queued.can_transition_to(TaskExecutionStatus::Running));
        assert!(TaskExecutionStatus::Running.can_transition_to(TaskExecutionStatus::Succeeded));
        assert!(TaskExecutionStatus::Running.can_transition_to(TaskExecutionStatus::Failed));
        assert!(TaskExecutionStatus::Running.can_transition_to(TaskExecutionStatus::Cancelled));
        assert!(!TaskExecutionStatus::Queued.can_transition_to(TaskExecutionStatus::Succeeded));
        assert!(!TaskExecutionStatus::Succeeded.can_transition_to(TaskExecutionStatus::Running));
    }

    #[test]
    fn task_audit_event_roundtrip_json_is_stable() {
        let event = TaskAuditEvent::new(
            "task-1",
            RuntimeMode::Service,
            TaskExecutionStatus::Running,
            "worker picked task",
            1_737_000_000_000,
        );
        let json = serde_json::to_string(&event).expect("task audit should serialize");
        let parsed: TaskAuditEvent =
            serde_json::from_str(&json).expect("task audit should deserialize");
        assert_eq!(parsed, event);
    }
}
