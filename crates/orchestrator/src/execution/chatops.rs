//! ChatOps integration foundation for remote task orchestration.
//!
//! This module provides platform-agnostic contracts for Telegram/Discord
//! command ingress, notification dispatch, local audit persistence, and
//! deterministic command execution through the existing `/task` pipeline.

use super::processor::{process_command, process_control_envelope};
use crate::models::ExitStatus;
use nettoolskit_core::{
    AppConfig, ApprovalState, ControlEnvelope, ControlPolicyContext, IngressTransport,
    OperatorContext, OperatorKind, RuntimeMode, SessionContext, SessionKind, TaskIntent,
    TaskIntentKind,
};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::fmt::{Display, Formatter};
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Error, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

/// Supported ChatOps platforms.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChatOpsPlatform {
    /// Telegram bot/channel integration.
    Telegram,
    /// Discord bot/channel integration.
    Discord,
}

impl ChatOpsPlatform {
    /// Canonical lowercase label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Telegram => "telegram",
            Self::Discord => "discord",
        }
    }
}

impl Display for ChatOpsPlatform {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Remote ChatOps command envelope received from a platform adapter.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatOpsCommandEnvelope {
    /// Source platform.
    pub platform: ChatOpsPlatform,
    /// Stable request identifier for one inbound chat message.
    #[serde(default)]
    pub request_id: String,
    /// Optional end-to-end correlation identifier.
    #[serde(default)]
    pub correlation_id: Option<String>,
    /// Remote channel identifier.
    pub channel_id: String,
    /// Remote user identifier.
    pub user_id: String,
    /// Transport used by the source adapter.
    #[serde(default)]
    pub transport: IngressTransport,
    /// Raw command text from chat message.
    pub message_text: String,
    /// UTC unix timestamp in milliseconds.
    pub received_at_unix_ms: u64,
}

impl ChatOpsCommandEnvelope {
    /// Build a command envelope.
    #[must_use]
    pub fn new(
        platform: ChatOpsPlatform,
        channel_id: impl Into<String>,
        user_id: impl Into<String>,
        message_text: impl Into<String>,
        received_at_unix_ms: u64,
    ) -> Self {
        Self {
            platform,
            request_id: next_chatops_request_id(platform),
            correlation_id: None,
            channel_id: channel_id.into(),
            user_id: user_id.into(),
            transport: default_chatops_transport(platform),
            message_text: message_text.into(),
            received_at_unix_ms,
        }
    }

    /// Override the ingress transport attached to the envelope.
    #[must_use]
    pub fn with_transport(mut self, transport: IngressTransport) -> Self {
        self.transport = transport;
        self
    }

    /// Override the stable request identifier attached to the envelope.
    #[must_use]
    pub fn with_request_id(mut self, request_id: impl Into<String>) -> Self {
        let request_id = request_id.into();
        let request_id = request_id.trim();
        if !request_id.is_empty() {
            self.request_id = request_id.to_string();
        }
        self
    }

    /// Attach an end-to-end correlation identifier.
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

    fn resolved_request_id(&self) -> String {
        let request_id = self.request_id.trim();
        if request_id.is_empty() {
            next_chatops_request_id(self.platform)
        } else {
            request_id.to_string()
        }
    }

    fn resolved_transport(&self) -> IngressTransport {
        if self.transport == IngressTransport::Cli {
            default_chatops_transport(self.platform)
        } else {
            self.transport
        }
    }

    fn derived_session_id(&self) -> String {
        format!(
            "chatops-{}-{}-{}",
            self.platform.as_str(),
            self.user_id.trim(),
            self.channel_id.trim()
        )
    }
}

/// Parsed ChatOps intent mapped to an internal command operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChatOpsIntent {
    /// Submit task intent with payload.
    TaskSubmit {
        /// Task intent kind (`ai-plan`, `ai-explain`, etc.).
        intent: String,
        /// Free-form payload forwarded to task submission.
        payload: String,
    },
    /// List known tasks.
    TaskList,
    /// Watch task status.
    TaskWatch {
        /// Stable task identifier.
        task_id: String,
    },
    /// Cancel task by id.
    TaskCancel {
        /// Stable task identifier.
        task_id: String,
    },
    /// Show task command help.
    Help,
}

impl ChatOpsIntent {
    /// Render canonical internal slash command for execution.
    #[must_use]
    pub fn to_internal_command(&self) -> Option<String> {
        match self {
            Self::TaskSubmit { intent, payload } => {
                Some(format!("/task submit {intent} {payload}"))
            }
            Self::TaskList => Some("/task list".to_string()),
            Self::TaskWatch { task_id } => Some(format!("/task watch {task_id}")),
            Self::TaskCancel { task_id } => Some(format!("/task cancel {task_id}")),
            Self::Help => None,
        }
    }

    /// Canonical authorization scopes used by command allowlist matching.
    #[must_use]
    pub fn authorization_scopes(&self) -> Vec<String> {
        match self {
            Self::Help => vec!["help".to_string()],
            Self::TaskList => vec!["list".to_string()],
            Self::TaskWatch { .. } => vec!["watch".to_string()],
            Self::TaskCancel { .. } => vec!["cancel".to_string()],
            Self::TaskSubmit { intent, .. } => {
                let normalized_intent = intent.trim().to_ascii_lowercase();
                vec!["submit".to_string(), format!("submit:{normalized_intent}")]
            }
        }
    }
}

/// Parse failures for chat message intents.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChatOpsParseError {
    /// Command format was invalid.
    InvalidFormat(String),
    /// Command action is unsupported.
    UnsupportedCommand(String),
}

impl Display for ChatOpsParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidFormat(message) => write!(f, "invalid ChatOps command format: {message}"),
            Self::UnsupportedCommand(command) => {
                write!(f, "unsupported ChatOps command: {command}")
            }
        }
    }
}

impl std::error::Error for ChatOpsParseError {}

/// Parse chat message to a supported task-management intent.
///
/// Supported forms:
/// - `help`
/// - `list` or `task list`
/// - `watch <task-id>` or `task watch <task-id>`
/// - `cancel <task-id>` or `task cancel <task-id>`
/// - `submit <intent> <payload...>` or `task submit <intent> <payload...>`
/// - Optional prefixes `/` and `/ntk`.
pub fn parse_chatops_intent(message: &str) -> Result<ChatOpsIntent, ChatOpsParseError> {
    let mut normalized = message.trim();
    if normalized.is_empty() {
        return Err(ChatOpsParseError::InvalidFormat(
            "empty message".to_string(),
        ));
    }

    if let Some(without_prefix) = normalized.strip_prefix("/ntk") {
        normalized = without_prefix.trim_start();
    }
    normalized = normalized.trim_start_matches('/');

    let tokens: Vec<&str> = normalized.split_whitespace().collect();
    if tokens.is_empty() {
        return Err(ChatOpsParseError::InvalidFormat(
            "empty command".to_string(),
        ));
    }

    let first = tokens[0].to_ascii_lowercase();
    if first == "help" {
        return Ok(ChatOpsIntent::Help);
    }

    let (command, offset) = if first == "task" {
        if tokens.len() < 2 {
            return Err(ChatOpsParseError::InvalidFormat(
                "task command requires an action".to_string(),
            ));
        }
        (tokens[1].to_ascii_lowercase(), 2usize)
    } else {
        (first, 1usize)
    };

    match command.as_str() {
        "list" => Ok(ChatOpsIntent::TaskList),
        "watch" => {
            let task_id = tokens
                .get(offset)
                .ok_or_else(|| {
                    ChatOpsParseError::InvalidFormat("watch requires <task-id>".to_string())
                })?
                .to_string();
            Ok(ChatOpsIntent::TaskWatch { task_id })
        }
        "cancel" => {
            let task_id = tokens
                .get(offset)
                .ok_or_else(|| {
                    ChatOpsParseError::InvalidFormat("cancel requires <task-id>".to_string())
                })?
                .to_string();
            Ok(ChatOpsIntent::TaskCancel { task_id })
        }
        "submit" => {
            if tokens.len() < offset + 2 {
                return Err(ChatOpsParseError::InvalidFormat(
                    "submit requires <intent> <payload>".to_string(),
                ));
            }

            let intent = tokens[offset].to_string();
            let payload = tokens[offset + 1..].join(" ");
            Ok(ChatOpsIntent::TaskSubmit { intent, payload })
        }
        unsupported => Err(ChatOpsParseError::UnsupportedCommand(
            unsupported.to_string(),
        )),
    }
}

/// Authorization policy for remote ChatOps commands.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatOpsAuthorizationPolicy {
    /// Explicit allowlist of user ids.
    pub allowed_user_ids: Vec<String>,
    /// Explicit allowlist of channel ids.
    pub allowed_channel_ids: Vec<String>,
    /// Optional allowlist of command scopes.
    ///
    /// Examples:
    /// - `list`, `watch`, `cancel`, `help`
    /// - `submit` (all submit intents)
    /// - `submit:ai-plan` (specific submit intent)
    /// - `*` (all command scopes)
    pub allowed_command_scopes: Vec<String>,
}

impl ChatOpsAuthorizationPolicy {
    /// Default secure policy that blocks all requests until configured.
    #[must_use]
    pub fn deny_by_default() -> Self {
        Self {
            allowed_user_ids: Vec::new(),
            allowed_channel_ids: Vec::new(),
            allowed_command_scopes: Vec::new(),
        }
    }

    /// Build policy from explicit allowlists.
    #[must_use]
    pub fn new(allowed_user_ids: Vec<String>, allowed_channel_ids: Vec<String>) -> Self {
        Self::new_with_scopes(allowed_user_ids, allowed_channel_ids, Vec::new())
    }

    /// Build policy from explicit allowlists and command scopes.
    #[must_use]
    pub fn new_with_scopes(
        allowed_user_ids: Vec<String>,
        allowed_channel_ids: Vec<String>,
        allowed_command_scopes: Vec<String>,
    ) -> Self {
        Self {
            allowed_user_ids,
            allowed_channel_ids,
            allowed_command_scopes: allowed_command_scopes
                .into_iter()
                .map(|scope| scope.trim().to_ascii_lowercase())
                .filter(|scope| !scope.is_empty())
                .collect(),
        }
    }

    /// Validate a command envelope against configured allowlists.
    pub fn authorize(
        &self,
        envelope: &ChatOpsCommandEnvelope,
    ) -> Result<(), ChatOpsAuthorizationError> {
        if self.allowed_user_ids.is_empty() || self.allowed_channel_ids.is_empty() {
            return Err(ChatOpsAuthorizationError::PolicyNotConfigured);
        }

        if !self
            .allowed_user_ids
            .iter()
            .any(|id| id == envelope.user_id.trim())
        {
            return Err(ChatOpsAuthorizationError::UserNotAllowed(
                envelope.user_id.clone(),
            ));
        }

        if !self
            .allowed_channel_ids
            .iter()
            .any(|id| id == envelope.channel_id.trim())
        {
            return Err(ChatOpsAuthorizationError::ChannelNotAllowed(
                envelope.channel_id.clone(),
            ));
        }

        Ok(())
    }

    /// Validate a parsed intent against optional command-scope allowlist.
    pub fn authorize_intent(
        &self,
        intent: &ChatOpsIntent,
    ) -> Result<(), ChatOpsAuthorizationError> {
        if self.allowed_command_scopes.is_empty()
            || self
                .allowed_command_scopes
                .iter()
                .any(|scope| scope.as_str() == "*")
        {
            return Ok(());
        }

        let labels = intent.authorization_scopes();
        let allowed = labels.iter().any(|label| {
            self.allowed_command_scopes
                .iter()
                .any(|scope| scope == label)
        });
        if allowed {
            Ok(())
        } else {
            Err(ChatOpsAuthorizationError::IntentNotAllowed(
                labels.join(","),
            ))
        }
    }
}

impl Default for ChatOpsAuthorizationPolicy {
    fn default() -> Self {
        Self::deny_by_default()
    }
}

/// Authorization failures for ChatOps command execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChatOpsAuthorizationError {
    /// No allowlists were configured.
    PolicyNotConfigured,
    /// User id is not allowed.
    UserNotAllowed(String),
    /// Channel id is not allowed.
    ChannelNotAllowed(String),
    /// Command scope is not allowed.
    IntentNotAllowed(String),
}

impl Display for ChatOpsAuthorizationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PolicyNotConfigured => {
                write!(f, "ChatOps policy is not configured (allowlists are empty)")
            }
            Self::UserNotAllowed(user) => write!(f, "ChatOps user is not allowed: {user}"),
            Self::ChannelNotAllowed(channel) => {
                write!(f, "ChatOps channel is not allowed: {channel}")
            }
            Self::IntentNotAllowed(scope) => {
                write!(f, "ChatOps command scope is not allowed: {scope}")
            }
        }
    }
}

impl std::error::Error for ChatOpsAuthorizationError {}

/// Notification severity for ChatOps outbound messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChatOpsNotificationSeverity {
    /// Informational status.
    Info,
    /// Successful operation completion.
    Success,
    /// Warning requiring operator attention.
    Warning,
    /// Error/failure event.
    Error,
}

/// Outbound ChatOps notification payload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatOpsNotification {
    /// Target platform.
    pub platform: ChatOpsPlatform,
    /// Target channel.
    pub channel_id: String,
    /// User-facing notification text.
    pub message_text: String,
    /// Severity marker.
    pub severity: ChatOpsNotificationSeverity,
}

/// ChatOps ingress adapter contract.
pub trait ChatOpsIngress: Send + Sync {
    /// Pull pending envelopes for processing (bounded by `max_items`).
    fn pull_pending(&self, max_items: usize) -> Vec<ChatOpsCommandEnvelope>;
}

/// ChatOps notifier adapter contract.
pub trait ChatOpsNotifier: Send + Sync {
    /// Send a platform notification.
    ///
    /// # Errors
    ///
    /// Returns error if notification dispatch fails.
    fn send(&self, notification: &ChatOpsNotification) -> Result<(), ChatOpsAdapterError>;
}

/// Adapter error for ingress/notifier backends.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatOpsAdapterError {
    message: String,
}

impl ChatOpsAdapterError {
    /// Build adapter error from message.
    #[must_use]
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl Display for ChatOpsAdapterError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for ChatOpsAdapterError {}

/// Deterministic mock ingress storing queued envelopes in-memory.
#[derive(Clone, Default)]
pub struct MockChatOpsIngress {
    queue: Arc<Mutex<VecDeque<ChatOpsCommandEnvelope>>>,
}

impl MockChatOpsIngress {
    /// Build empty mock ingress.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Build ingress preloaded with envelopes.
    #[must_use]
    pub fn with_envelopes(envelopes: Vec<ChatOpsCommandEnvelope>) -> Self {
        Self {
            queue: Arc::new(Mutex::new(VecDeque::from(envelopes))),
        }
    }

    /// Append one envelope to ingress queue.
    pub fn push(&self, envelope: ChatOpsCommandEnvelope) {
        let mut queue = self
            .queue
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        queue.push_back(envelope);
    }
}

impl ChatOpsIngress for MockChatOpsIngress {
    fn pull_pending(&self, max_items: usize) -> Vec<ChatOpsCommandEnvelope> {
        if max_items == 0 {
            return Vec::new();
        }

        let mut queue = self
            .queue
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let mut items = Vec::new();
        for _ in 0..max_items {
            if let Some(next) = queue.pop_front() {
                items.push(next);
            } else {
                break;
            }
        }
        items
    }
}

/// Deterministic notifier that records notifications in-memory.
#[derive(Clone, Default)]
pub struct RecordingChatOpsNotifier {
    notifications: Arc<Mutex<Vec<ChatOpsNotification>>>,
}

impl RecordingChatOpsNotifier {
    /// Build empty recording notifier.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Retrieve recorded notifications snapshot.
    #[must_use]
    pub fn snapshot(&self) -> Vec<ChatOpsNotification> {
        self.notifications
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }
}

impl ChatOpsNotifier for RecordingChatOpsNotifier {
    fn send(&self, notification: &ChatOpsNotification) -> Result<(), ChatOpsAdapterError> {
        self.notifications
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .push(notification.clone());
        Ok(())
    }
}

/// Audit event category for ChatOps lifecycle tracking.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChatOpsAuditKind {
    /// Envelope received from ingress adapter.
    CommandReceived,
    /// Envelope rejected before execution.
    CommandRejected,
    /// Envelope executed through internal command router.
    CommandExecuted,
    /// Notification delivery completed.
    NotificationSent,
}

/// Local audit entry persisted as JSONL.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatOpsAuditEntry {
    /// Event category.
    pub kind: ChatOpsAuditKind,
    /// Source platform.
    pub platform: ChatOpsPlatform,
    /// Source channel.
    pub channel_id: String,
    /// Source user.
    pub user_id: String,
    /// Raw message text.
    pub message_text: String,
    /// Normalized internal command, when available.
    pub internal_command: Option<String>,
    /// Stable request identifier attributed to the chat ingress event.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    /// Optional end-to-end correlation identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub correlation_id: Option<String>,
    /// Normalized operator identifier when a typed control envelope exists.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub operator_id: Option<String>,
    /// Normalized session identifier when a typed control envelope exists.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    /// Ingress transport used by the message.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transport: Option<IngressTransport>,
    /// Stable task identifier when task admission succeeded.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub task_id: Option<String>,
    /// Exit status label for executed commands.
    pub exit_status: Option<String>,
    /// Human-readable note.
    pub note: String,
    /// UTC unix timestamp in milliseconds.
    pub timestamp_unix_ms: u64,
}

struct ChatOpsAuditContext<'a> {
    internal_command: Option<&'a str>,
    control_envelope: Option<&'a ControlEnvelope>,
    task_id: Option<&'a str>,
    exit_status: Option<&'a str>,
    note: &'a str,
}

/// Local JSONL persistence for ChatOps audit events.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatOpsLocalAuditStore {
    path: PathBuf,
}

impl ChatOpsLocalAuditStore {
    /// Relative file path used under default data directory.
    pub const DEFAULT_RELATIVE_PATH: &'static str = "chatops/audit.jsonl";

    /// Build store from explicit path.
    #[must_use]
    pub fn from_path(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    /// Build store using default local data directory.
    #[must_use]
    pub fn from_default_data_dir() -> Option<Self> {
        AppConfig::default_data_dir().map(|base| Self {
            path: base.join(Self::DEFAULT_RELATIVE_PATH),
        })
    }

    /// Access configured store path.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Append audit entry to local JSONL file.
    ///
    /// # Errors
    ///
    /// Returns error when path creation or write fails.
    pub fn append(&self, entry: &ChatOpsAuditEntry) -> Result<(), Error> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;
        let line = serde_json::to_string(entry)
            .map_err(|err| Error::other(format!("serialize audit entry: {err}")))?;
        writeln!(file, "{line}")?;
        file.flush()
    }

    /// Load latest audit entries (best-effort parse, malformed lines are skipped).
    ///
    /// # Errors
    ///
    /// Returns error when file cannot be opened/read.
    pub fn load_latest(&self, limit: usize) -> Result<Vec<ChatOpsAuditEntry>, Error> {
        if limit == 0 || !self.path.exists() {
            return Ok(Vec::new());
        }

        let file = OpenOptions::new().read(true).open(&self.path)?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();
        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            if let Ok(parsed) = serde_json::from_str::<ChatOpsAuditEntry>(&line) {
                entries.push(parsed);
            }
        }

        if entries.len() > limit {
            let drain_until = entries.len() - limit;
            entries.drain(0..drain_until);
        }
        Ok(entries)
    }
}

/// Execution errors for ChatOps envelope processing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChatOpsExecutionError {
    /// Authorization failed.
    Unauthorized(String),
    /// Parsing failed.
    Parse(String),
    /// Notification dispatch failed.
    Notify(String),
}

impl Display for ChatOpsExecutionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unauthorized(message) => write!(f, "chatops authorization failed: {message}"),
            Self::Parse(message) => write!(f, "chatops parse failed: {message}"),
            Self::Notify(message) => write!(f, "chatops notification failed: {message}"),
        }
    }
}

impl std::error::Error for ChatOpsExecutionError {}

/// Execute one chat envelope using the existing task command pipeline.
///
/// # Errors
///
/// Returns error when authorization, parsing, or notification fails.
pub async fn execute_chatops_envelope(
    envelope: &ChatOpsCommandEnvelope,
    policy: &ChatOpsAuthorizationPolicy,
    notifier: &dyn ChatOpsNotifier,
    audit_store: Option<&ChatOpsLocalAuditStore>,
) -> Result<ExitStatus, ChatOpsExecutionError> {
    append_audit(
        audit_store,
        envelope,
        ChatOpsAuditKind::CommandReceived,
        ChatOpsAuditContext {
            internal_command: None,
            control_envelope: None,
            task_id: None,
            exit_status: None,
            note: "envelope received",
        },
    );

    if let Err(error) = policy.authorize(envelope) {
        append_audit(
            audit_store,
            envelope,
            ChatOpsAuditKind::CommandRejected,
            ChatOpsAuditContext {
                internal_command: None,
                control_envelope: None,
                task_id: None,
                exit_status: Some("error"),
                note: &error.to_string(),
            },
        );

        let notification = ChatOpsNotification {
            platform: envelope.platform,
            channel_id: envelope.channel_id.clone(),
            message_text: format!("Access denied: {error}"),
            severity: ChatOpsNotificationSeverity::Error,
        };
        notifier
            .send(&notification)
            .map_err(|err| ChatOpsExecutionError::Notify(err.to_string()))?;
        append_audit(
            audit_store,
            envelope,
            ChatOpsAuditKind::NotificationSent,
            ChatOpsAuditContext {
                internal_command: None,
                control_envelope: None,
                task_id: None,
                exit_status: Some("error"),
                note: "denial notification sent",
            },
        );
        return Err(ChatOpsExecutionError::Unauthorized(error.to_string()));
    }

    let intent = parse_chatops_intent(&envelope.message_text)
        .map_err(|error| ChatOpsExecutionError::Parse(error.to_string()))?;
    if let Err(error) = policy.authorize_intent(&intent) {
        append_audit(
            audit_store,
            envelope,
            ChatOpsAuditKind::CommandRejected,
            ChatOpsAuditContext {
                internal_command: None,
                control_envelope: None,
                task_id: None,
                exit_status: Some("error"),
                note: &error.to_string(),
            },
        );

        let notification = ChatOpsNotification {
            platform: envelope.platform,
            channel_id: envelope.channel_id.clone(),
            message_text: format!("Access denied: {error}"),
            severity: ChatOpsNotificationSeverity::Error,
        };
        notifier
            .send(&notification)
            .map_err(|err| ChatOpsExecutionError::Notify(err.to_string()))?;
        append_audit(
            audit_store,
            envelope,
            ChatOpsAuditKind::NotificationSent,
            ChatOpsAuditContext {
                internal_command: None,
                control_envelope: None,
                task_id: None,
                exit_status: Some("error"),
                note: "intent denial notification sent",
            },
        );
        return Err(ChatOpsExecutionError::Unauthorized(error.to_string()));
    }
    if matches!(intent, ChatOpsIntent::Help) {
        let help_message = "ChatOps commands: help | list | watch <task-id> | cancel <task-id> | submit <intent> <payload>";
        let notification = ChatOpsNotification {
            platform: envelope.platform,
            channel_id: envelope.channel_id.clone(),
            message_text: help_message.to_string(),
            severity: ChatOpsNotificationSeverity::Info,
        };
        notifier
            .send(&notification)
            .map_err(|err| ChatOpsExecutionError::Notify(err.to_string()))?;
        append_audit(
            audit_store,
            envelope,
            ChatOpsAuditKind::NotificationSent,
            ChatOpsAuditContext {
                internal_command: None,
                control_envelope: None,
                task_id: None,
                exit_status: Some("success"),
                note: "help notification sent",
            },
        );
        return Ok(ExitStatus::Success);
    }

    let internal_command = intent
        .to_internal_command()
        .expect("non-help intent should always map to command");
    let (status, control_envelope, task_id, execution_note) =
        match build_chatops_control_envelope(envelope, &intent, audit_store)? {
            Some(control_envelope) => {
                let submission = process_control_envelope(control_envelope.clone()).await;
                let note = if submission.task_id.is_some() {
                    "command executed through typed ChatOps control plane"
                } else {
                    "typed ChatOps control plane rejected task admission"
                };
                (
                    submission.exit_status,
                    Some(control_envelope),
                    submission.task_id,
                    note,
                )
            }
            None => (
                process_command(&internal_command).await,
                None,
                None,
                "command executed through task pipeline",
            ),
        };

    append_audit(
        audit_store,
        envelope,
        ChatOpsAuditKind::CommandExecuted,
        ChatOpsAuditContext {
            internal_command: Some(&internal_command),
            control_envelope: control_envelope.as_ref(),
            task_id: task_id.as_deref(),
            exit_status: Some(status.to_string().as_str()),
            note: execution_note,
        },
    );

    let severity = match status {
        ExitStatus::Success => ChatOpsNotificationSeverity::Success,
        ExitStatus::Interrupted => ChatOpsNotificationSeverity::Warning,
        ExitStatus::Error => ChatOpsNotificationSeverity::Error,
    };
    let notification = ChatOpsNotification {
        platform: envelope.platform,
        channel_id: envelope.channel_id.clone(),
        message_text: build_chatops_status_message(&internal_command, status, task_id.as_deref()),
        severity,
    };
    notifier
        .send(&notification)
        .map_err(|err| ChatOpsExecutionError::Notify(err.to_string()))?;

    append_audit(
        audit_store,
        envelope,
        ChatOpsAuditKind::NotificationSent,
        ChatOpsAuditContext {
            internal_command: Some(&internal_command),
            control_envelope: control_envelope.as_ref(),
            task_id: task_id.as_deref(),
            exit_status: Some(status.to_string().as_str()),
            note: "result notification sent",
        },
    );

    Ok(status)
}

/// Process pending envelopes from ingress adapter.
pub async fn process_chatops_inbox(
    ingress: &dyn ChatOpsIngress,
    policy: &ChatOpsAuthorizationPolicy,
    notifier: &dyn ChatOpsNotifier,
    audit_store: Option<&ChatOpsLocalAuditStore>,
    max_items: usize,
) -> Vec<Result<ExitStatus, ChatOpsExecutionError>> {
    let mut results = Vec::new();
    for envelope in ingress.pull_pending(max_items) {
        let result = execute_chatops_envelope(&envelope, policy, notifier, audit_store).await;
        results.push(result);
    }
    results
}

fn append_audit(
    store: Option<&ChatOpsLocalAuditStore>,
    envelope: &ChatOpsCommandEnvelope,
    kind: ChatOpsAuditKind,
    context: ChatOpsAuditContext<'_>,
) {
    let Some(store) = store else {
        return;
    };
    let entry = ChatOpsAuditEntry {
        kind,
        platform: envelope.platform,
        channel_id: envelope.channel_id.clone(),
        user_id: envelope.user_id.clone(),
        message_text: envelope.message_text.clone(),
        internal_command: context.internal_command.map(ToOwned::to_owned),
        request_id: Some(envelope.resolved_request_id()),
        correlation_id: context
            .control_envelope
            .and_then(|control| control.correlation_id.clone())
            .or_else(|| envelope.correlation_id.clone()),
        operator_id: context
            .control_envelope
            .map(|control| control.operator.id.clone()),
        session_id: context
            .control_envelope
            .map(|control| control.session.id.clone()),
        transport: Some(
            context
                .control_envelope
                .map(|control| control.operator.transport)
                .unwrap_or_else(|| envelope.resolved_transport()),
        ),
        task_id: context.task_id.map(ToOwned::to_owned),
        exit_status: context.exit_status.map(ToOwned::to_owned),
        note: context.note.to_string(),
        timestamp_unix_ms: envelope.received_at_unix_ms,
    };
    let _ = store.append(&entry);
}

fn build_chatops_status_message(
    internal_command: &str,
    status: ExitStatus,
    task_id: Option<&str>,
) -> String {
    match task_id {
        Some(task_id) => format!(
            "Command `{internal_command}` completed with status `{status}` as task `{task_id}`."
        ),
        None => format!("Command `{internal_command}` completed with status `{status}`."),
    }
}

fn build_chatops_control_envelope(
    envelope: &ChatOpsCommandEnvelope,
    intent: &ChatOpsIntent,
    audit_store: Option<&ChatOpsLocalAuditStore>,
) -> Result<Option<ControlEnvelope>, ChatOpsExecutionError> {
    let ChatOpsIntent::TaskSubmit {
        intent: task_intent,
        payload,
    } = intent
    else {
        return Ok(None);
    };

    let Some(task_kind) = TaskIntentKind::from_alias(task_intent) else {
        return Err(ChatOpsExecutionError::Parse(format!(
            "unsupported ChatOps task intent: {task_intent}"
        )));
    };

    let mut scopes = vec!["task.submit".to_string()];
    scopes.extend(intent.authorization_scopes());
    scopes.push(format!("platform:{}", envelope.platform.as_str()));

    let operator = OperatorContext::new(
        OperatorKind::RemoteHuman,
        format!("{}:{}", envelope.platform.as_str(), envelope.user_id.trim()),
        envelope.resolved_transport(),
    )
    .with_channel_id(envelope.channel_id.clone())
    .with_authentication(format!("chatops_{}", envelope.platform.as_str()))
    .with_scopes(scopes);
    let session = SessionContext::new(SessionKind::ChatOps, envelope.derived_session_id(), true);
    let task = TaskIntent::new(
        task_kind,
        format!("{} task", task_kind.as_str()),
        payload.trim(),
    );
    let mut control = ControlEnvelope::new(
        envelope.resolved_request_id(),
        RuntimeMode::Service,
        operator,
        session,
        task,
    )
    .with_policy(build_chatops_control_policy(audit_store));
    if let Some(correlation_id) = envelope.correlation_id.clone() {
        control = control.with_correlation_id(correlation_id);
    }

    Ok(Some(control))
}

fn build_chatops_control_policy(
    audit_store: Option<&ChatOpsLocalAuditStore>,
) -> ControlPolicyContext {
    let mut policy = ControlPolicyContext::new(ApprovalState::NotRequired, true);
    if let Some(audit_store) = audit_store {
        policy = policy.with_local_audit(audit_store.path().display().to_string());
    }
    policy
}

fn default_chatops_transport(platform: ChatOpsPlatform) -> IngressTransport {
    match platform {
        ChatOpsPlatform::Telegram => IngressTransport::TelegramPolling,
        ChatOpsPlatform::Discord => IngressTransport::DiscordPolling,
    }
}

fn next_chatops_request_id(platform: ChatOpsPlatform) -> String {
    static CHATOPS_REQUEST_SEQUENCE: AtomicU64 = AtomicU64::new(1);
    let sequence = CHATOPS_REQUEST_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    format!(
        "chatops-{}-{}-{sequence:08x}",
        platform.as_str(),
        current_unix_timestamp_ms()
    )
}

fn current_unix_timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    struct EnvVarGuard {
        saved: Vec<(String, Option<String>)>,
    }

    impl EnvVarGuard {
        fn set(vars: &[(&str, Option<&str>)]) -> Self {
            let mut saved = Vec::with_capacity(vars.len());
            for (key, value) in vars {
                saved.push(((*key).to_string(), std::env::var(key).ok()));
                match value {
                    Some(value) => std::env::set_var(key, value),
                    None => std::env::remove_var(key),
                }
            }
            Self { saved }
        }
    }

    impl Drop for EnvVarGuard {
        fn drop(&mut self) {
            for (key, value) in self.saved.drain(..) {
                match value {
                    Some(value) => std::env::set_var(&key, value),
                    None => std::env::remove_var(&key),
                }
            }
        }
    }

    #[test]
    fn parse_chatops_intent_supports_submit_and_list_aliases() {
        assert_eq!(
            parse_chatops_intent("submit ai-plan improve docs"),
            Ok(ChatOpsIntent::TaskSubmit {
                intent: "ai-plan".to_string(),
                payload: "improve docs".to_string()
            })
        );
        assert_eq!(
            parse_chatops_intent("/task list"),
            Ok(ChatOpsIntent::TaskList)
        );
    }

    #[test]
    fn parse_chatops_intent_rejects_invalid_submit_format() {
        let parsed = parse_chatops_intent("submit ai-plan");
        assert!(matches!(parsed, Err(ChatOpsParseError::InvalidFormat(_))));
    }

    #[test]
    fn authorization_policy_denies_by_default() {
        let policy = ChatOpsAuthorizationPolicy::default();
        let envelope =
            ChatOpsCommandEnvelope::new(ChatOpsPlatform::Telegram, "c1", "u1", "list", 1);
        assert!(matches!(
            policy.authorize(&envelope),
            Err(ChatOpsAuthorizationError::PolicyNotConfigured)
        ));
    }

    #[test]
    fn authorization_policy_rejects_disallowed_submit_scope() {
        let policy = ChatOpsAuthorizationPolicy::new_with_scopes(
            vec!["u-1".to_string()],
            vec!["c-1".to_string()],
            vec!["submit:ai-plan".to_string()],
        );
        let disallowed = ChatOpsIntent::TaskSubmit {
            intent: "ai-explain".to_string(),
            payload: "why".to_string(),
        };
        let allowed = ChatOpsIntent::TaskSubmit {
            intent: "ai-plan".to_string(),
            payload: "plan".to_string(),
        };

        assert!(matches!(
            policy.authorize_intent(&disallowed),
            Err(ChatOpsAuthorizationError::IntentNotAllowed(_))
        ));
        assert!(policy.authorize_intent(&allowed).is_ok());
    }

    #[test]
    fn local_audit_store_roundtrip_keeps_latest_entries() {
        let dir = tempfile::tempdir().expect("temp dir");
        let store = ChatOpsLocalAuditStore::from_path(dir.path().join("audit.jsonl"));

        for index in 0..3 {
            let entry = ChatOpsAuditEntry {
                kind: ChatOpsAuditKind::CommandReceived,
                platform: ChatOpsPlatform::Discord,
                channel_id: "ops".to_string(),
                user_id: "user".to_string(),
                message_text: format!("list-{index}"),
                internal_command: None,
                request_id: None,
                correlation_id: None,
                operator_id: None,
                session_id: None,
                transport: None,
                task_id: None,
                exit_status: None,
                note: "received".to_string(),
                timestamp_unix_ms: index,
            };
            store.append(&entry).expect("append should succeed");
        }

        let latest = store.load_latest(2).expect("load should succeed");
        assert_eq!(latest.len(), 2);
        assert_eq!(latest[0].message_text, "list-1");
        assert_eq!(latest[1].message_text, "list-2");
    }

    #[tokio::test]
    async fn execute_chatops_envelope_runs_task_command_when_authorized() {
        let policy = ChatOpsAuthorizationPolicy::new(
            vec!["u-allowed".to_string()],
            vec!["channel-1".to_string()],
        );
        let notifier = RecordingChatOpsNotifier::new();
        let envelope = ChatOpsCommandEnvelope::new(
            ChatOpsPlatform::Telegram,
            "channel-1",
            "u-allowed",
            "list",
            42,
        );

        let status = execute_chatops_envelope(&envelope, &policy, &notifier, None)
            .await
            .expect("authorized command should execute");
        assert_eq!(status, ExitStatus::Success);

        let notifications = notifier.snapshot();
        assert_eq!(notifications.len(), 1);
        assert_eq!(
            notifications[0].severity,
            ChatOpsNotificationSeverity::Success
        );
    }

    #[tokio::test]
    async fn execute_chatops_envelope_denied_request_sends_error_notification() {
        let policy = ChatOpsAuthorizationPolicy::default();
        let notifier = RecordingChatOpsNotifier::new();
        let envelope = ChatOpsCommandEnvelope::new(
            ChatOpsPlatform::Discord,
            "channel-x",
            "user-x",
            "list",
            88,
        );

        let result = execute_chatops_envelope(&envelope, &policy, &notifier, None).await;
        assert!(matches!(
            result,
            Err(ChatOpsExecutionError::Unauthorized(_))
        ));

        let notifications = notifier.snapshot();
        assert_eq!(notifications.len(), 1);
        assert_eq!(
            notifications[0].severity,
            ChatOpsNotificationSeverity::Error
        );
    }

    #[tokio::test]
    async fn execute_chatops_envelope_denies_submit_intent_not_in_scope() {
        let policy = ChatOpsAuthorizationPolicy::new_with_scopes(
            vec!["user-1".to_string()],
            vec!["channel-1".to_string()],
            vec!["submit:ai-plan".to_string()],
        );
        let notifier = RecordingChatOpsNotifier::new();
        let envelope = ChatOpsCommandEnvelope::new(
            ChatOpsPlatform::Telegram,
            "channel-1",
            "user-1",
            "submit ai-explain why it failed",
            91,
        );

        let result = execute_chatops_envelope(&envelope, &policy, &notifier, None).await;
        assert!(matches!(
            result,
            Err(ChatOpsExecutionError::Unauthorized(_))
        ));

        let notifications = notifier.snapshot();
        assert_eq!(notifications.len(), 1);
        assert_eq!(
            notifications[0].severity,
            ChatOpsNotificationSeverity::Error
        );
    }

    #[test]
    fn build_chatops_control_envelope_normalizes_remote_operator_and_session() {
        let envelope = ChatOpsCommandEnvelope::new(
            ChatOpsPlatform::Discord,
            "channel-42",
            "user-99",
            "submit ai-plan review rollout",
            91,
        )
        .with_transport(IngressTransport::DiscordInteractions)
        .with_request_id("discord-req-001")
        .with_correlation_id("corr-42");
        let intent = ChatOpsIntent::TaskSubmit {
            intent: "ai-plan".to_string(),
            payload: "review rollout".to_string(),
        };

        let control = build_chatops_control_envelope(&envelope, &intent, None)
            .expect("control envelope should build")
            .expect("submit intent should produce typed envelope");

        assert_eq!(control.request_id, "discord-req-001");
        assert_eq!(control.correlation_id.as_deref(), Some("corr-42"));
        assert_eq!(control.operator.id, "discord:user-99");
        assert_eq!(control.operator.channel_id.as_deref(), Some("channel-42"));
        assert_eq!(
            control.operator.transport,
            IngressTransport::DiscordInteractions
        );
        assert_eq!(control.session.id, "chatops-discord-user-99-channel-42");
        assert_eq!(control.task.kind, TaskIntentKind::AiPlan);
        assert_eq!(control.task.title, "ai-plan task");
        assert!(control
            .operator
            .scopes
            .iter()
            .any(|scope| scope == "task.submit"));
        assert!(control
            .operator
            .scopes
            .iter()
            .any(|scope| scope == "submit:ai-plan"));
    }

    #[tokio::test]
    #[serial]
    async fn execute_chatops_envelope_submit_records_control_plane_metadata() {
        let _env_guard = EnvVarGuard::set(&[
            ("NTK_TOOL_SCOPE_ALLOWED_TOOLS", Some("ai.plan")),
            ("NTK_TOOL_SCOPE_INTENT_AI_PLAN_TOOLS", Some("ai.plan")),
        ]);
        let dir = tempfile::tempdir().expect("temp dir");
        let audit_store = ChatOpsLocalAuditStore::from_path(dir.path().join("chatops-audit.jsonl"));
        let policy = ChatOpsAuthorizationPolicy::new_with_scopes(
            vec!["user-1".to_string()],
            vec!["channel-1".to_string()],
            vec!["submit:ai-plan".to_string()],
        );
        let notifier = RecordingChatOpsNotifier::new();
        let envelope = ChatOpsCommandEnvelope::new(
            ChatOpsPlatform::Telegram,
            "channel-1",
            "user-1",
            "submit ai-plan harden chatops ingress",
            123,
        )
        .with_request_id("telegram-req-001")
        .with_correlation_id("corr-chatops-001");

        let status = execute_chatops_envelope(&envelope, &policy, &notifier, Some(&audit_store))
            .await
            .expect("authorized submit should execute");
        assert_eq!(status, ExitStatus::Success);

        let entries = audit_store
            .load_latest(16)
            .expect("audit entries should be readable");
        let executed = entries
            .iter()
            .find(|entry| entry.kind == ChatOpsAuditKind::CommandExecuted)
            .expect("executed entry should exist");
        assert_eq!(executed.request_id.as_deref(), Some("telegram-req-001"));
        assert_eq!(executed.correlation_id.as_deref(), Some("corr-chatops-001"));
        assert_eq!(executed.operator_id.as_deref(), Some("telegram:user-1"));
        assert_eq!(
            executed.session_id.as_deref(),
            Some("chatops-telegram-user-1-channel-1")
        );
        assert_eq!(executed.transport, Some(IngressTransport::TelegramPolling));
        assert!(executed.task_id.as_deref().is_some());
        assert!(executed.note.contains("typed ChatOps control plane"));

        let notifications = notifier.snapshot();
        assert_eq!(notifications.len(), 1);
        assert!(notifications[0].message_text.contains("task `task-"));
    }

    #[tokio::test]
    async fn process_chatops_inbox_processes_bounded_pending_messages() {
        let ingress = MockChatOpsIngress::with_envelopes(vec![
            ChatOpsCommandEnvelope::new(ChatOpsPlatform::Telegram, "channel-1", "u-1", "list", 1),
            ChatOpsCommandEnvelope::new(ChatOpsPlatform::Telegram, "channel-1", "u-1", "help", 2),
            ChatOpsCommandEnvelope::new(ChatOpsPlatform::Telegram, "channel-1", "u-1", "list", 3),
        ]);
        let policy =
            ChatOpsAuthorizationPolicy::new(vec!["u-1".to_string()], vec!["channel-1".to_string()]);
        let notifier = RecordingChatOpsNotifier::new();

        let results = process_chatops_inbox(&ingress, &policy, &notifier, None, 2).await;
        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|item| item.is_ok()));

        let remaining = ingress.pull_pending(10);
        assert_eq!(remaining.len(), 1);
    }
}
