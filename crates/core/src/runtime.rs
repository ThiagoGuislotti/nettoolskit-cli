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
    /// Repository workflow task (`clone -> branch -> execute -> commit/push/pr`).
    RepoWorkflow,
}

impl TaskIntentKind {
    /// Canonical lowercase label for display and metadata emission.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::CommandExecution => "command",
            Self::AiAsk => "ai-ask",
            Self::AiPlan => "ai-plan",
            Self::AiExplain => "ai-explain",
            Self::AiApplyDryRun => "ai-apply-dry-run",
            Self::RepoWorkflow => "repo-workflow",
        }
    }

    /// Parse the canonical label or a supported alias.
    #[must_use]
    pub fn from_alias(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "command" | "cmd" => Some(Self::CommandExecution),
            "ai-ask" | "ask" => Some(Self::AiAsk),
            "ai-plan" | "plan" => Some(Self::AiPlan),
            "ai-explain" | "explain" => Some(Self::AiExplain),
            "ai-apply-dry-run" | "apply-dry-run" | "apply" => Some(Self::AiApplyDryRun),
            "repo-workflow" | "repo" | "repository-workflow" => Some(Self::RepoWorkflow),
            _ => None,
        }
    }
}

impl fmt::Display for TaskIntentKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
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

/// Operator identity class admitted by the control plane.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum OperatorKind {
    /// Trusted local workstation operator.
    #[default]
    LocalHuman,
    /// Remote human operator using HTTP or ChatOps transport.
    RemoteHuman,
    /// Non-human automation operator.
    Automation,
    /// Adapter/platform identity acting on behalf of another operator.
    PlatformAdapter,
}

impl OperatorKind {
    /// Canonical label for display and serialization-like usage.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::LocalHuman => "local_human",
            Self::RemoteHuman => "remote_human",
            Self::Automation => "automation",
            Self::PlatformAdapter => "platform_adapter",
        }
    }
}

impl fmt::Display for OperatorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Ingress transport used to submit work to the control plane.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum IngressTransport {
    /// Local interactive CLI runtime.
    #[default]
    Cli,
    /// Service HTTP ingress.
    ServiceHttp,
    /// Telegram webhook ingress.
    TelegramWebhook,
    /// Telegram polling ingress.
    TelegramPolling,
    /// Discord interactions ingress.
    DiscordInteractions,
    /// Discord polling ingress.
    DiscordPolling,
}

impl IngressTransport {
    /// Canonical label for display and serialization-like usage.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Cli => "cli",
            Self::ServiceHttp => "service_http",
            Self::TelegramWebhook => "telegram_webhook",
            Self::TelegramPolling => "telegram_polling",
            Self::DiscordInteractions => "discord_interactions",
            Self::DiscordPolling => "discord_polling",
        }
    }
}

impl fmt::Display for IngressTransport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Session boundary kinds recognized by the control plane.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum SessionKind {
    /// Local interactive CLI session.
    #[default]
    CliInteractive,
    /// Local AI conversation session.
    AiConversation,
    /// Per-request service session.
    ServiceRequest,
    /// Remote ChatOps session.
    ChatOps,
    /// Repository automation workflow session.
    RepoWorkflow,
}

impl SessionKind {
    /// Canonical label for display and serialization-like usage.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::CliInteractive => "cli_interactive",
            Self::AiConversation => "ai_conversation",
            Self::ServiceRequest => "service_request",
            Self::ChatOps => "chatops",
            Self::RepoWorkflow => "repo_workflow",
        }
    }
}

impl fmt::Display for SessionKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Approval state carried by the control envelope.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalState {
    /// Approval is not required for the current request.
    #[default]
    NotRequired,
    /// Approval is required before mutation/execution.
    Required,
    /// Approval was explicitly granted.
    Approved,
    /// Approval was explicitly rejected.
    Rejected,
}

impl ApprovalState {
    /// Returns `true` when the request still requires approval.
    #[must_use]
    pub const fn is_pending(self) -> bool {
        matches!(self, Self::Required)
    }
}

/// Normalized operator identity admitted by the control plane.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OperatorContext {
    /// Operator class.
    pub kind: OperatorKind,
    /// Stable operator identifier.
    pub id: String,
    /// Optional remote channel/group identifier.
    pub channel_id: Option<String>,
    /// Transport used by the operator.
    pub transport: IngressTransport,
    /// Authentication evidence summary.
    pub authentication: Option<String>,
    /// Canonical scope list for authorization decisions.
    pub scopes: Vec<String>,
}

impl OperatorContext {
    /// Build a normalized operator context.
    #[must_use]
    pub fn new(kind: OperatorKind, id: impl Into<String>, transport: IngressTransport) -> Self {
        Self {
            kind,
            id: id.into().trim().to_string(),
            channel_id: None,
            transport,
            authentication: None,
            scopes: Vec::new(),
        }
    }

    /// Attach a normalized channel identifier.
    #[must_use]
    pub fn with_channel_id(mut self, channel_id: impl Into<String>) -> Self {
        let channel_id = channel_id.into();
        let channel_id = channel_id.trim();
        self.channel_id = if channel_id.is_empty() {
            None
        } else {
            Some(channel_id.to_string())
        };
        self
    }

    /// Attach authentication evidence text.
    #[must_use]
    pub fn with_authentication(mut self, authentication: impl Into<String>) -> Self {
        let authentication = authentication.into();
        let authentication = authentication.trim();
        self.authentication = if authentication.is_empty() {
            None
        } else {
            Some(authentication.to_string())
        };
        self
    }

    /// Replace scopes with normalized lower-case values.
    #[must_use]
    pub fn with_scopes<I, S>(mut self, scopes: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.scopes = scopes
            .into_iter()
            .map(Into::into)
            .map(|scope| scope.trim().to_ascii_lowercase())
            .filter(|scope| !scope.is_empty())
            .collect();
        self
    }
}

/// Session context attached to one control-plane operation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionContext {
    /// Session class.
    pub kind: SessionKind,
    /// Stable session identifier.
    pub id: String,
    /// Whether the session is resumable.
    pub resumable: bool,
}

impl SessionContext {
    /// Build a normalized session context.
    #[must_use]
    pub fn new(kind: SessionKind, id: impl Into<String>, resumable: bool) -> Self {
        Self {
            kind,
            id: id.into().trim().to_string(),
            resumable,
        }
    }
}

/// Policy decision attached to one admitted control-plane request.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ControlPolicyContext {
    /// Approval state for the request.
    pub approval_state: ApprovalState,
    /// Whether mutating behavior is allowed for this request.
    pub mutable_actions_allowed: bool,
    /// Whether local audit persistence is enabled.
    pub persist_local_audit: bool,
    /// Optional local audit store path/label.
    pub audit_store: Option<String>,
}

impl ControlPolicyContext {
    /// Build a policy context with explicit approval and mutation state.
    #[must_use]
    pub fn new(approval_state: ApprovalState, mutable_actions_allowed: bool) -> Self {
        Self {
            approval_state,
            mutable_actions_allowed,
            persist_local_audit: false,
            audit_store: None,
        }
    }

    /// Mark local audit persistence details.
    #[must_use]
    pub fn with_local_audit(mut self, audit_store: impl Into<String>) -> Self {
        let audit_store = audit_store.into();
        let audit_store = audit_store.trim();
        self.persist_local_audit = true;
        self.audit_store = if audit_store.is_empty() {
            None
        } else {
            Some(audit_store.to_string())
        };
        self
    }
}

impl Default for ControlPolicyContext {
    fn default() -> Self {
        Self::new(ApprovalState::NotRequired, false)
    }
}

/// Transport-neutral control-plane envelope for admitted work.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ControlEnvelope {
    /// Stable request identifier.
    pub request_id: String,
    /// Optional end-to-end correlation identifier.
    pub correlation_id: Option<String>,
    /// Runtime mode where the envelope was created.
    pub runtime_mode: RuntimeMode,
    /// Operator responsible for the request.
    pub operator: OperatorContext,
    /// Session boundary attached to the request.
    pub session: SessionContext,
    /// Task intent carried by the request.
    pub task: TaskIntent,
    /// Policy decision for the request.
    pub policy: ControlPolicyContext,
}

impl ControlEnvelope {
    /// Build a control envelope with default policy.
    #[must_use]
    pub fn new(
        request_id: impl Into<String>,
        runtime_mode: RuntimeMode,
        operator: OperatorContext,
        session: SessionContext,
        task: TaskIntent,
    ) -> Self {
        Self {
            request_id: request_id.into().trim().to_string(),
            correlation_id: None,
            runtime_mode,
            operator,
            session,
            task,
            policy: ControlPolicyContext::default(),
        }
    }

    /// Attach a correlation identifier.
    #[must_use]
    pub fn with_correlation_id(mut self, correlation_id: impl Into<String>) -> Self {
        let correlation_id = correlation_id.into();
        let correlation_id = correlation_id.trim();
        self.correlation_id = if correlation_id.is_empty() {
            None
        } else {
            Some(correlation_id.to_string())
        };
        self
    }

    /// Attach an explicit policy context.
    #[must_use]
    pub fn with_policy(mut self, policy: ControlPolicyContext) -> Self {
        self.policy = policy;
        self
    }

    /// Returns `true` when approval is still required.
    #[must_use]
    pub const fn requires_approval(&self) -> bool {
        self.policy.approval_state.is_pending()
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
    /// Optional control-plane envelope captured at admission time.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub control: Option<ControlEnvelope>,
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
            control: None,
            timestamp_unix_ms,
        }
    }

    /// Attach the admitted control-plane envelope for richer downstream audit.
    #[must_use]
    pub fn with_control_envelope(mut self, control: ControlEnvelope) -> Self {
        self.control = Some(control);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn operator_context_normalizes_channel_authentication_and_scopes() {
        let operator = OperatorContext::new(
            OperatorKind::RemoteHuman,
            " telegram:777 ",
            IngressTransport::TelegramWebhook,
        )
        .with_channel_id("  telegram:555 ")
        .with_authentication(" bearer+allowlist ")
        .with_scopes([" Submit:AI-Plan ", "watch", ""]);

        assert_eq!(operator.id, "telegram:777");
        assert_eq!(operator.channel_id.as_deref(), Some("telegram:555"));
        assert_eq!(operator.authentication.as_deref(), Some("bearer+allowlist"));
        assert_eq!(operator.scopes, vec!["submit:ai-plan", "watch"]);
    }

    #[test]
    fn control_envelope_with_policy_and_correlation_is_stable() {
        let envelope = ControlEnvelope::new(
            "req-1",
            RuntimeMode::Service,
            OperatorContext::new(
                OperatorKind::RemoteHuman,
                "discord:777",
                IngressTransport::DiscordInteractions,
            )
            .with_scopes(["submit:ai-plan"]),
            SessionContext::new(SessionKind::ChatOps, "chatops-discord-777-555", true),
            TaskIntent::new(
                TaskIntentKind::AiPlan,
                "Plan release gate",
                "tighten dual runtime",
            ),
        )
        .with_correlation_id("corr-1")
        .with_policy(
            ControlPolicyContext::new(ApprovalState::Required, false)
                .with_local_audit("chatops/audit.jsonl"),
        );

        assert!(envelope.requires_approval());
        assert_eq!(envelope.correlation_id.as_deref(), Some("corr-1"));
        assert_eq!(
            envelope.policy.audit_store.as_deref(),
            Some("chatops/audit.jsonl")
        );
        let json = serde_json::to_string(&envelope).expect("control envelope should serialize");
        let parsed: ControlEnvelope =
            serde_json::from_str(&json).expect("control envelope should deserialize");
        assert_eq!(parsed, envelope);
    }

    #[test]
    fn control_policy_context_default_is_safe() {
        let policy = ControlPolicyContext::default();
        assert_eq!(policy.approval_state, ApprovalState::NotRequired);
        assert!(!policy.mutable_actions_allowed);
        assert!(!policy.persist_local_audit);
        assert_eq!(policy.audit_store, None);
    }

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
    fn task_intent_kind_aliases_and_labels_are_stable() {
        assert_eq!(TaskIntentKind::AiPlan.as_str(), "ai-plan");
        assert_eq!(
            TaskIntentKind::from_alias("plan"),
            Some(TaskIntentKind::AiPlan)
        );
        assert_eq!(
            TaskIntentKind::from_alias("repository-workflow"),
            Some(TaskIntentKind::RepoWorkflow)
        );
        assert_eq!(TaskIntentKind::from_alias("unknown"), None);
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

    #[test]
    fn task_audit_event_with_control_envelope_roundtrips() {
        let control = ControlEnvelope::new(
            "req-1",
            RuntimeMode::Service,
            OperatorContext::new(
                OperatorKind::RemoteHuman,
                "service-http-operator",
                IngressTransport::ServiceHttp,
            ),
            SessionContext::new(SessionKind::ServiceRequest, "service-session-1", false),
            TaskIntent::new(TaskIntentKind::AiPlan, "ai-plan task", "review backlog"),
        )
        .with_correlation_id("corr-1")
        .with_policy(ControlPolicyContext::new(ApprovalState::NotRequired, true));
        let event = TaskAuditEvent::new(
            "task-1",
            RuntimeMode::Service,
            TaskExecutionStatus::Queued,
            "task submitted",
            1_737_000_000_000,
        )
        .with_control_envelope(control.clone());

        let json = serde_json::to_string(&event).expect("task audit should serialize");
        let parsed: TaskAuditEvent =
            serde_json::from_str(&json).expect("task audit should deserialize");

        assert_eq!(parsed.control, Some(control));
        assert_eq!(parsed, event);
    }
}
