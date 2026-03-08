//! Local AI session persistence and resume utilities.
//!
//! This module stores AI conversation history in local application data only.
//! It also exposes a process-wide "active session id" used by interactive
//! flows to resume context across multiple `/ai` commands.

use crate::execution::ai::{AiMessage, AiRole};
use nettoolskit_core::AppConfig;
use nettoolskit_otel::next_correlation_id;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

/// Directory name under NTK data root used for local AI session snapshots.
pub const LOCAL_AI_SESSIONS_DIR_NAME: &str = "ai-sessions";
const LOCAL_AI_SESSION_FILE_EXTENSION: &str = "json";
const MAX_EXCHANGES_PER_SESSION: usize = 200;
const DEFAULT_AI_SESSION_COMPRESSION_MAX_CHARS: usize = 1200;
const DEFAULT_AI_SESSION_DELTA_MIN_SHARED_PREFIX_CHARS: usize = 80;
const SUMMARY_COMPACTION_MARKER_FALLBACK: &str = "... [summary]";
/// Compression mode env for local AI session assistant responses (`off`, `delta`, `summary`).
pub const NTK_AI_SESSION_COMPRESSION_MODE_ENV: &str = "NTK_AI_SESSION_COMPRESSION_MODE";
/// Max chars for `summary` compression mode.
pub const NTK_AI_SESSION_COMPRESSION_MAX_CHARS_ENV: &str = "NTK_AI_SESSION_COMPRESSION_MAX_CHARS";
/// Min shared prefix chars required before `delta` mode stores only response tail.
pub const NTK_AI_SESSION_DELTA_MIN_SHARED_PREFIX_CHARS_ENV: &str =
    "NTK_AI_SESSION_DELTA_MIN_SHARED_PREFIX_CHARS";
static ACTIVE_AI_SESSION_ID: OnceLock<Mutex<Option<String>>> = OnceLock::new();

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "kebab-case")]
/// Compression mode for persisted assistant responses in local AI session snapshots.
pub enum AiSessionCompressionMode {
    /// Persist full assistant responses without compaction.
    #[default]
    Off,
    /// Persist only the tail delta when consecutive assistant responses share large prefixes.
    Delta,
    /// Persist bounded summaries (head/tail with omission marker) for large responses.
    Summary,
}

impl AiSessionCompressionMode {
    fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "off" | "none" | "disabled" => Some(Self::Off),
            "delta" => Some(Self::Delta),
            "summary" | "compact" => Some(Self::Summary),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct AiSessionCompressionPolicy {
    mode: AiSessionCompressionMode,
    summary_max_chars: usize,
    delta_min_shared_prefix_chars: usize,
}

impl Default for AiSessionCompressionPolicy {
    fn default() -> Self {
        Self {
            mode: AiSessionCompressionMode::Off,
            summary_max_chars: DEFAULT_AI_SESSION_COMPRESSION_MAX_CHARS,
            delta_min_shared_prefix_chars: DEFAULT_AI_SESSION_DELTA_MIN_SHARED_PREFIX_CHARS,
        }
    }
}

#[derive(Debug, Clone)]
struct CompressedAiSessionResponse {
    content: String,
    mode: AiSessionCompressionMode,
    original_chars: usize,
}

/// Single persisted AI exchange (user prompt + assistant response).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AiSessionExchange {
    /// AI intent label (`ask`, `plan`, `explain`, `apply`).
    pub intent: String,
    /// Provider identifier used for this exchange.
    pub provider: String,
    /// User prompt submitted to provider.
    pub user_prompt: String,
    /// Assistant response returned by provider.
    pub assistant_response: String,
    /// Assistant response storage mode used for persisted representation.
    #[serde(default)]
    pub response_storage_mode: AiSessionCompressionMode,
    /// Char count of the original assistant response before compression.
    #[serde(default)]
    pub original_response_chars: usize,
    /// UTC epoch timestamp in milliseconds.
    pub timestamp_ms: u64,
}

impl AiSessionExchange {
    /// Build an exchange with normalized fields and current timestamp.
    #[must_use]
    pub fn new(
        intent: impl Into<String>,
        provider: impl Into<String>,
        user_prompt: impl Into<String>,
        assistant_response: impl Into<String>,
    ) -> Self {
        let assistant_response = assistant_response.into().trim().to_string();
        Self {
            intent: intent.into().trim().to_string(),
            provider: provider.into().trim().to_string(),
            user_prompt: user_prompt.into().trim().to_string(),
            original_response_chars: assistant_response.chars().count(),
            assistant_response,
            response_storage_mode: AiSessionCompressionMode::Off,
            timestamp_ms: now_epoch_ms(),
        }
    }

    fn new_with_storage(
        intent: impl Into<String>,
        provider: impl Into<String>,
        user_prompt: impl Into<String>,
        assistant_response: impl Into<String>,
        response_storage_mode: AiSessionCompressionMode,
        original_response_chars: usize,
    ) -> Self {
        Self {
            intent: intent.into().trim().to_string(),
            provider: provider.into().trim().to_string(),
            user_prompt: user_prompt.into().trim().to_string(),
            assistant_response: assistant_response.into().trim().to_string(),
            response_storage_mode,
            original_response_chars,
            timestamp_ms: now_epoch_ms(),
        }
    }

    fn ensure_response_metadata(&mut self) {
        if self.original_response_chars == 0 {
            self.original_response_chars = self.assistant_response.chars().count();
        }
    }
}

/// Persisted local AI conversation state.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LocalAiSessionState {
    /// Stable session identifier.
    pub id: String,
    /// Session creation time in epoch milliseconds.
    pub started_at_ms: u64,
    /// Last update time in epoch milliseconds.
    pub last_activity_ms: u64,
    /// Ordered exchange history for this session.
    pub exchanges: VecDeque<AiSessionExchange>,
}

/// Metadata for local AI session picker/list operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalAiSessionSnapshot {
    /// Session identifier.
    pub id: String,
    /// Session start time in epoch milliseconds.
    pub started_at_ms: u64,
    /// Last update time in epoch milliseconds.
    pub last_activity_ms: u64,
    /// Number of persisted exchanges.
    pub exchange_count: usize,
    /// Snapshot JSON file path.
    pub path: PathBuf,
}

/// Set the process-local active AI session id used by `/ai` commands.
///
/// Returns the sanitized identifier that was stored.
#[must_use]
pub fn set_active_ai_session_id(session_id: &str) -> String {
    let sanitized = sanitize_session_id(session_id);
    let mut guard = active_session_guard()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    *guard = Some(sanitized.clone());
    sanitized
}

/// Read the current process-local active AI session id, if available.
#[must_use]
pub fn active_ai_session_id() -> Option<String> {
    let guard = active_session_guard()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    guard.clone()
}

/// Resolve an active AI session id or create a new deterministic one.
///
/// If no active session is currently set, this generates one and stores it.
#[must_use]
pub fn resolve_active_ai_session_id() -> String {
    if let Some(active) = active_ai_session_id() {
        return active;
    }

    let generated = sanitize_session_id(&next_correlation_id("ai"));
    set_active_ai_session_id(&generated)
}

/// List local AI session snapshots sorted from newest to oldest.
///
/// Returns `Ok(None)` when no OS data directory is available.
///
/// # Errors
///
/// Returns `Err` when reading local session metadata fails.
pub fn list_local_ai_session_snapshots(
    limit: usize,
) -> io::Result<Option<Vec<LocalAiSessionSnapshot>>> {
    LocalAiSessionState::list_local_snapshots(limit)
}

/// Load a persisted AI session from an explicit snapshot path.
///
/// # Errors
///
/// Returns `Err` when file read or JSON parse fails.
pub fn load_local_ai_session_from_path(path: &Path) -> io::Result<LocalAiSessionState> {
    LocalAiSessionState::load_local_snapshot_from_path(path)
}

/// Prune local AI sessions and keep only the latest `keep_latest`.
///
/// Returns `Ok(None)` when no OS data directory is available.
///
/// # Errors
///
/// Returns `Err` when pruning operation fails.
pub fn prune_local_ai_session_snapshots(keep_latest: usize) -> io::Result<Option<usize>> {
    LocalAiSessionState::prune_local_snapshots(keep_latest)
}

fn active_session_guard() -> &'static Mutex<Option<String>> {
    ACTIVE_AI_SESSION_ID.get_or_init(|| Mutex::new(None))
}

impl LocalAiSessionState {
    /// Create an empty AI session state with current timestamps.
    #[must_use]
    pub fn new(session_id: impl Into<String>) -> Self {
        let now = now_epoch_ms();
        Self {
            id: sanitize_session_id(&session_id.into()),
            started_at_ms: now,
            last_activity_ms: now,
            exchanges: VecDeque::new(),
        }
    }

    /// Append one AI exchange to this session.
    ///
    /// Returns `false` when prompt/response are blank and nothing was recorded.
    pub fn append_exchange(
        &mut self,
        intent: &str,
        provider: &str,
        user_prompt: &str,
        assistant_response: &str,
    ) -> bool {
        let prompt = user_prompt.trim();
        let response = assistant_response.trim();
        if prompt.is_empty() || response.is_empty() {
            return false;
        }

        if self.exchanges.len() == MAX_EXCHANGES_PER_SESSION {
            self.exchanges.pop_front();
        }

        let compression_policy = ai_session_compression_policy_from_env();
        let previous_response = self
            .exchanges
            .back()
            .map(|exchange| exchange.assistant_response.as_str());
        let compressed =
            compress_ai_session_response(response, previous_response, compression_policy);
        let exchange = AiSessionExchange::new_with_storage(
            intent,
            provider,
            prompt,
            compressed.content,
            compressed.mode,
            compressed.original_chars,
        );
        self.last_activity_ms = exchange.timestamp_ms.max(self.last_activity_ms);
        self.exchanges.push_back(exchange);
        true
    }

    /// Build bounded conversation history as provider-ready messages.
    #[must_use]
    pub fn recent_messages(&self, max_messages: usize) -> Vec<AiMessage> {
        if max_messages == 0 || self.exchanges.is_empty() {
            return Vec::new();
        }

        let max_turns = (max_messages / 2).max(1);
        let start = self.exchanges.len().saturating_sub(max_turns);

        let mut messages = Vec::with_capacity(max_turns * 2);
        for exchange in self.exchanges.iter().skip(start) {
            if !exchange.user_prompt.is_empty() {
                messages.push(AiMessage::new(AiRole::User, exchange.user_prompt.clone()));
            }
            if !exchange.assistant_response.is_empty() {
                messages.push(AiMessage::new(
                    AiRole::Assistant,
                    exchange.assistant_response.clone(),
                ));
            }
        }

        if messages.len() > max_messages {
            messages.split_off(messages.len() - max_messages)
        } else {
            messages
        }
    }

    /// Save this state into local application data directory.
    ///
    /// Returns `Ok(None)` when no OS data directory is available.
    ///
    /// # Errors
    ///
    /// Returns `Err` on directory create, serialization, or file write failures.
    pub fn save_local_snapshot(&self) -> io::Result<Option<PathBuf>> {
        let Some(base_dir) = AppConfig::default_data_dir() else {
            return Ok(None);
        };

        self.save_local_snapshot_to_dir(&base_dir).map(Some)
    }

    /// Load a local AI session by id from default application data directory.
    ///
    /// Returns `Ok(None)` when no data dir is available or session file is absent.
    ///
    /// # Errors
    ///
    /// Returns `Err` when file read or parse fails.
    pub fn load_local_snapshot(session_id: &str) -> io::Result<Option<Self>> {
        let Some(base_dir) = AppConfig::default_data_dir() else {
            return Ok(None);
        };

        Self::load_local_snapshot_from_dir(&base_dir, session_id)
    }

    /// Load a local AI session snapshot from explicit file path.
    ///
    /// # Errors
    ///
    /// Returns `Err` when file read or parse fails.
    pub fn load_local_snapshot_from_path(path: &Path) -> io::Result<Self> {
        let json = fs::read_to_string(path)?;
        let mut state: Self = serde_json::from_str(&json).map_err(json_to_io_error)?;
        state.normalize_exchange_metadata();
        Ok(state)
    }

    /// List local AI session snapshots.
    ///
    /// Returns `Ok(None)` when no OS data directory is available.
    ///
    /// # Errors
    ///
    /// Returns `Err` when listing or parsing snapshots fails.
    pub fn list_local_snapshots(limit: usize) -> io::Result<Option<Vec<LocalAiSessionSnapshot>>> {
        let Some(base_dir) = AppConfig::default_data_dir() else {
            return Ok(None);
        };

        Self::list_local_snapshots_in_dir(&base_dir, limit).map(Some)
    }

    /// Prune local AI session snapshots, keeping latest `keep_latest`.
    ///
    /// Returns `Ok(None)` when no OS data directory is available.
    ///
    /// # Errors
    ///
    /// Returns `Err` when deleting old snapshots fails.
    pub fn prune_local_snapshots(keep_latest: usize) -> io::Result<Option<usize>> {
        let Some(base_dir) = AppConfig::default_data_dir() else {
            return Ok(None);
        };

        Self::prune_local_snapshots_in_dir(&base_dir, keep_latest).map(Some)
    }

    fn save_local_snapshot_to_dir(&self, base_dir: &Path) -> io::Result<PathBuf> {
        let sessions_dir = base_dir.join(LOCAL_AI_SESSIONS_DIR_NAME);
        fs::create_dir_all(&sessions_dir)?;

        let path = session_file_path(&sessions_dir, &self.id);
        let json = serde_json::to_string_pretty(self).map_err(json_to_io_error)?;
        fs::write(&path, json)?;
        Ok(path)
    }

    fn load_local_snapshot_from_dir(base_dir: &Path, session_id: &str) -> io::Result<Option<Self>> {
        let sessions_dir = base_dir.join(LOCAL_AI_SESSIONS_DIR_NAME);
        if !sessions_dir.exists() {
            return Ok(None);
        }

        let path = session_file_path(&sessions_dir, session_id);
        if !path.exists() {
            return Ok(None);
        }

        Self::load_local_snapshot_from_path(&path).map(Some)
    }

    fn list_local_snapshots_in_dir(
        base_dir: &Path,
        limit: usize,
    ) -> io::Result<Vec<LocalAiSessionSnapshot>> {
        if limit == 0 {
            return Ok(Vec::new());
        }

        let sessions_dir = base_dir.join(LOCAL_AI_SESSIONS_DIR_NAME);
        if !sessions_dir.exists() {
            return Ok(Vec::new());
        }

        let mut snapshots = Vec::new();
        for entry in fs::read_dir(&sessions_dir)? {
            let entry = entry?;
            let path = entry.path();
            if !is_json_snapshot_file(&path) {
                continue;
            }

            let state = match Self::load_local_snapshot_from_path(&path) {
                Ok(state) => state,
                Err(_) => continue,
            };
            snapshots.push(LocalAiSessionSnapshot::from_state(path, &state));
        }

        snapshots.sort_by(|left, right| {
            right
                .last_activity_ms
                .cmp(&left.last_activity_ms)
                .then_with(|| right.started_at_ms.cmp(&left.started_at_ms))
                .then_with(|| left.id.cmp(&right.id))
        });
        snapshots.truncate(limit);

        Ok(snapshots)
    }

    fn prune_local_snapshots_in_dir(base_dir: &Path, keep_latest: usize) -> io::Result<usize> {
        let snapshots = Self::list_local_snapshots_in_dir(base_dir, usize::MAX)?;
        if snapshots.len() <= keep_latest {
            return Ok(0);
        }

        let mut removed = 0usize;
        for snapshot in snapshots.into_iter().skip(keep_latest) {
            fs::remove_file(snapshot.path)?;
            removed += 1;
        }

        Ok(removed)
    }
}

impl LocalAiSessionState {
    fn normalize_exchange_metadata(&mut self) {
        for exchange in &mut self.exchanges {
            exchange.ensure_response_metadata();
        }
    }
}

impl LocalAiSessionSnapshot {
    fn from_state(path: PathBuf, state: &LocalAiSessionState) -> Self {
        Self {
            id: state.id.clone(),
            started_at_ms: state.started_at_ms,
            last_activity_ms: state.last_activity_ms,
            exchange_count: state.exchanges.len(),
            path,
        }
    }
}

fn now_epoch_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or(0)
}

fn parse_nonzero_usize(value: &str) -> Option<usize> {
    let parsed = value.trim().parse::<usize>().ok()?;
    (parsed > 0).then_some(parsed)
}

fn ai_session_compression_policy_from_env() -> AiSessionCompressionPolicy {
    let mut policy = AiSessionCompressionPolicy::default();

    if let Ok(value) = std::env::var(NTK_AI_SESSION_COMPRESSION_MODE_ENV) {
        if let Some(parsed) = AiSessionCompressionMode::parse(&value) {
            policy.mode = parsed;
        }
    }

    if let Ok(value) = std::env::var(NTK_AI_SESSION_COMPRESSION_MAX_CHARS_ENV) {
        if let Some(parsed) = parse_nonzero_usize(&value) {
            policy.summary_max_chars = parsed;
        }
    }

    if let Ok(value) = std::env::var(NTK_AI_SESSION_DELTA_MIN_SHARED_PREFIX_CHARS_ENV) {
        if let Some(parsed) = parse_nonzero_usize(&value) {
            policy.delta_min_shared_prefix_chars = parsed;
        }
    }

    policy
}

fn common_prefix_chars(left: &str, right: &str) -> usize {
    left.chars()
        .zip(right.chars())
        .take_while(|(l, r)| l == r)
        .count()
}

fn take_tail_chars(value: &str, count: usize) -> String {
    if count == 0 {
        return String::new();
    }

    let chars = value.chars().count();
    if count >= chars {
        return value.to_string();
    }

    value.chars().skip(chars - count).collect()
}

fn summarize_response_text(value: &str, max_chars: usize) -> String {
    let total_chars = value.chars().count();
    if total_chars <= max_chars {
        return value.to_string();
    }

    let omitted = total_chars.saturating_sub(max_chars);
    let marker = format!(" ... [summary:{omitted} chars omitted] ... ");
    let marker_chars = marker.chars().count();

    if marker_chars >= max_chars {
        let fallback_len = max_chars
            .saturating_sub(SUMMARY_COMPACTION_MARKER_FALLBACK.chars().count())
            .max(1);
        let mut output = value.chars().take(fallback_len).collect::<String>();
        output.push_str(SUMMARY_COMPACTION_MARKER_FALLBACK);
        return output;
    }

    let content_budget = max_chars.saturating_sub(marker_chars);
    let head_chars = ((content_budget as f64) * 0.7).round() as usize;
    let tail_chars = content_budget.saturating_sub(head_chars);

    let mut output = String::new();
    output.push_str(&value.chars().take(head_chars).collect::<String>());
    output.push_str(&marker);
    output.push_str(&take_tail_chars(value, tail_chars));
    output
}

fn compress_ai_session_response(
    assistant_response: &str,
    previous_assistant_response: Option<&str>,
    policy: AiSessionCompressionPolicy,
) -> CompressedAiSessionResponse {
    let normalized = assistant_response.trim();
    let original_chars = normalized.chars().count();
    if normalized.is_empty() {
        return CompressedAiSessionResponse {
            content: String::new(),
            mode: AiSessionCompressionMode::Off,
            original_chars,
        };
    }

    match policy.mode {
        AiSessionCompressionMode::Off => CompressedAiSessionResponse {
            content: normalized.to_string(),
            mode: AiSessionCompressionMode::Off,
            original_chars,
        },
        AiSessionCompressionMode::Summary => {
            let compacted = summarize_response_text(normalized, policy.summary_max_chars.max(1));
            let mode = if compacted == normalized {
                AiSessionCompressionMode::Off
            } else {
                AiSessionCompressionMode::Summary
            };
            CompressedAiSessionResponse {
                content: compacted,
                mode,
                original_chars,
            }
        }
        AiSessionCompressionMode::Delta => {
            let Some(previous) = previous_assistant_response.map(str::trim) else {
                return CompressedAiSessionResponse {
                    content: normalized.to_string(),
                    mode: AiSessionCompressionMode::Off,
                    original_chars,
                };
            };
            if previous.is_empty() {
                return CompressedAiSessionResponse {
                    content: normalized.to_string(),
                    mode: AiSessionCompressionMode::Off,
                    original_chars,
                };
            }

            let prefix = common_prefix_chars(previous, normalized);
            if prefix < policy.delta_min_shared_prefix_chars {
                return CompressedAiSessionResponse {
                    content: normalized.to_string(),
                    mode: AiSessionCompressionMode::Off,
                    original_chars,
                };
            }

            let delta_tail = normalized.chars().skip(prefix).collect::<String>();
            let compacted = if delta_tail.trim().is_empty() {
                format!("[delta from previous +{prefix} chars] (no additional content)")
            } else {
                format!("[delta from previous +{prefix} chars]\n{delta_tail}")
            };

            if compacted.chars().count() >= normalized.chars().count() {
                CompressedAiSessionResponse {
                    content: normalized.to_string(),
                    mode: AiSessionCompressionMode::Off,
                    original_chars,
                }
            } else {
                CompressedAiSessionResponse {
                    content: compacted,
                    mode: AiSessionCompressionMode::Delta,
                    original_chars,
                }
            }
        }
    }
}

fn sanitize_session_id(value: &str) -> String {
    let mut sanitized: String = value
        .trim()
        .chars()
        .map(|ch| match ch {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' => ch,
            _ => '_',
        })
        .collect();

    if sanitized.is_empty() {
        sanitized.push_str("ai-session");
    }

    sanitized
}

fn session_file_path(sessions_dir: &Path, session_id: &str) -> PathBuf {
    let file_name = format!(
        "{}.{LOCAL_AI_SESSION_FILE_EXTENSION}",
        sanitize_session_id(session_id)
    );
    sessions_dir.join(file_name)
}

fn is_json_snapshot_file(path: &Path) -> bool {
    path.is_file()
        && path
            .extension()
            .and_then(|value| value.to_str())
            .is_some_and(|value| value.eq_ignore_ascii_case(LOCAL_AI_SESSION_FILE_EXTENSION))
}

fn json_to_io_error(error: serde_json::Error) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, error)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, OnceLock};
    use tempfile::tempdir;

    static ENV_TEST_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

    fn env_test_guard() -> std::sync::MutexGuard<'static, ()> {
        ENV_TEST_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn clear_ai_session_compression_env_vars() {
        std::env::remove_var(NTK_AI_SESSION_COMPRESSION_MODE_ENV);
        std::env::remove_var(NTK_AI_SESSION_COMPRESSION_MAX_CHARS_ENV);
        std::env::remove_var(NTK_AI_SESSION_DELTA_MIN_SHARED_PREFIX_CHARS_ENV);
    }

    #[test]
    fn append_exchange_records_data_and_enforces_capacity() {
        let mut session = LocalAiSessionState::new("s");
        assert!(session.append_exchange("ask", "mock", "hello", "world"));
        assert!(!session.append_exchange("ask", "mock", "   ", "world"));
        assert!(!session.append_exchange("ask", "mock", "hello", "   "));

        assert_eq!(session.exchanges.len(), 1);
        assert_eq!(session.exchanges[0].intent, "ask");
        assert_eq!(session.exchanges[0].provider, "mock");
        assert_eq!(session.exchanges[0].user_prompt, "hello");
        assert_eq!(session.exchanges[0].assistant_response, "world");
        assert_eq!(
            session.exchanges[0].response_storage_mode,
            AiSessionCompressionMode::Off
        );
        assert_eq!(session.exchanges[0].original_response_chars, 5);
    }

    #[test]
    fn recent_messages_returns_ordered_pairs_with_limit() {
        let mut session = LocalAiSessionState::new("session-a");
        let _ = session.append_exchange("ask", "mock", "one", "resp-one");
        let _ = session.append_exchange("plan", "mock", "two", "resp-two");
        let _ = session.append_exchange("explain", "mock", "three", "resp-three");

        let messages = session.recent_messages(4);
        assert_eq!(messages.len(), 4);
        assert_eq!(messages[0].content, "two");
        assert_eq!(messages[1].content, "resp-two");
        assert_eq!(messages[2].content, "three");
        assert_eq!(messages[3].content, "resp-three");
    }

    #[test]
    fn local_snapshot_roundtrip_in_custom_dir() {
        let temp = tempdir().expect("temp dir");
        let mut session = LocalAiSessionState::new("session-local");
        let _ = session.append_exchange("ask", "mock", "question", "answer");

        let path = session
            .save_local_snapshot_to_dir(temp.path())
            .expect("save snapshot");
        assert!(path.exists());

        let loaded = LocalAiSessionState::load_local_snapshot_from_path(&path).expect("load");
        assert_eq!(loaded.id, "session-local");
        assert_eq!(loaded.exchanges.len(), 1);
        assert_eq!(loaded.exchanges[0].user_prompt, "question");
    }

    #[test]
    fn list_local_snapshots_sorts_by_last_activity() {
        let temp = tempdir().expect("temp dir");
        let mut first = LocalAiSessionState::new("a");
        first.started_at_ms = 10;
        first.last_activity_ms = 10;
        first
            .save_local_snapshot_to_dir(temp.path())
            .expect("save first");

        let mut second = LocalAiSessionState::new("b");
        second.started_at_ms = 20;
        second.last_activity_ms = 20;
        second
            .save_local_snapshot_to_dir(temp.path())
            .expect("save second");

        let snapshots = LocalAiSessionState::list_local_snapshots_in_dir(temp.path(), usize::MAX)
            .expect("list");
        assert_eq!(snapshots.len(), 2);
        assert_eq!(snapshots[0].id, "b");
        assert_eq!(snapshots[1].id, "a");
    }

    #[test]
    fn prune_local_snapshots_keeps_latest() {
        let temp = tempdir().expect("temp dir");
        for idx in 1_u64..=3 {
            let mut session = LocalAiSessionState::new(format!("session-{idx}"));
            session.started_at_ms = idx * 100;
            session.last_activity_ms = idx * 100;
            session
                .save_local_snapshot_to_dir(temp.path())
                .expect("save");
        }

        let removed =
            LocalAiSessionState::prune_local_snapshots_in_dir(temp.path(), 1).expect("prune");
        assert_eq!(removed, 2);
        let snapshots = LocalAiSessionState::list_local_snapshots_in_dir(temp.path(), usize::MAX)
            .expect("list");
        assert_eq!(snapshots.len(), 1);
        assert_eq!(snapshots[0].id, "session-3");
    }

    #[test]
    fn sanitize_session_id_replaces_invalid_chars() {
        assert_eq!(sanitize_session_id("my-session"), "my-session");
        assert_eq!(sanitize_session_id(" session:/x "), "session__x");
        assert_eq!(sanitize_session_id("   "), "ai-session");
    }

    #[test]
    fn parse_ai_session_compression_mode_supports_expected_variants() {
        assert_eq!(
            AiSessionCompressionMode::parse("off"),
            Some(AiSessionCompressionMode::Off)
        );
        assert_eq!(
            AiSessionCompressionMode::parse("delta"),
            Some(AiSessionCompressionMode::Delta)
        );
        assert_eq!(
            AiSessionCompressionMode::parse("summary"),
            Some(AiSessionCompressionMode::Summary)
        );
        assert_eq!(AiSessionCompressionMode::parse("invalid"), None);
    }

    #[test]
    fn append_exchange_uses_summary_compression_when_enabled() {
        let _guard = env_test_guard();
        clear_ai_session_compression_env_vars();
        std::env::set_var(NTK_AI_SESSION_COMPRESSION_MODE_ENV, "summary");
        std::env::set_var(NTK_AI_SESSION_COMPRESSION_MAX_CHARS_ENV, "64");

        let mut session = LocalAiSessionState::new("summary-session");
        let long_response = "A".repeat(180);
        let stored = session.append_exchange("ask", "mock", "hello", &long_response);

        clear_ai_session_compression_env_vars();
        assert!(stored);
        assert_eq!(session.exchanges.len(), 1);
        let exchange = &session.exchanges[0];
        assert_eq!(
            exchange.response_storage_mode,
            AiSessionCompressionMode::Summary
        );
        assert_eq!(exchange.original_response_chars, 180);
        assert!(exchange.assistant_response.chars().count() <= 64 + 32);
        assert!(exchange.assistant_response.contains("[summary:"));
    }

    #[test]
    fn append_exchange_uses_delta_compression_when_prefix_is_shared() {
        let _guard = env_test_guard();
        clear_ai_session_compression_env_vars();
        std::env::set_var(NTK_AI_SESSION_COMPRESSION_MODE_ENV, "delta");
        std::env::set_var(NTK_AI_SESSION_DELTA_MIN_SHARED_PREFIX_CHARS_ENV, "16");

        let mut session = LocalAiSessionState::new("delta-session");
        let base =
            "Shared analysis context for deployment rollout and guardrails. Stage 1 complete.";
        let follow_up = "Shared analysis context for deployment rollout and guardrails. Stage 2 adds canary validation and rollback checks.";
        assert!(session.append_exchange("plan", "mock", "step one", base));
        assert!(session.append_exchange("plan", "mock", "step two", follow_up));

        clear_ai_session_compression_env_vars();
        assert_eq!(session.exchanges.len(), 2);
        let first = &session.exchanges[0];
        let second = &session.exchanges[1];
        assert_eq!(first.response_storage_mode, AiSessionCompressionMode::Off);
        assert_eq!(
            second.response_storage_mode,
            AiSessionCompressionMode::Delta
        );
        assert!(second.assistant_response.contains("[delta from previous +"));
        assert!(
            second.assistant_response.chars().count() < follow_up.chars().count(),
            "delta payload should be smaller than original response"
        );
        assert_eq!(second.original_response_chars, follow_up.chars().count());
    }

    #[test]
    fn load_legacy_snapshot_without_compression_fields_remains_compatible() {
        let temp = tempdir().expect("temp dir");
        let path = temp.path().join("legacy-session.json");
        let legacy_json = r#"{
  "id": "legacy-session",
  "started_at_ms": 10,
  "last_activity_ms": 20,
  "exchanges": [
    {
      "intent": "ask",
      "provider": "mock",
      "user_prompt": "hello",
      "assistant_response": "legacy answer",
      "timestamp_ms": 30
    }
  ]
}"#;
        std::fs::write(&path, legacy_json).expect("legacy snapshot should be written");

        let loaded = LocalAiSessionState::load_local_snapshot_from_path(&path).expect("load");
        assert_eq!(loaded.id, "legacy-session");
        assert_eq!(loaded.exchanges.len(), 1);
        let exchange = &loaded.exchanges[0];
        assert_eq!(
            exchange.response_storage_mode,
            AiSessionCompressionMode::Off
        );
        assert_eq!(
            exchange.original_response_chars,
            "legacy answer".chars().count()
        );
    }
}
