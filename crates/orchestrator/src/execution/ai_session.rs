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
static ACTIVE_AI_SESSION_ID: OnceLock<Mutex<Option<String>>> = OnceLock::new();

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
        Self {
            intent: intent.into().trim().to_string(),
            provider: provider.into().trim().to_string(),
            user_prompt: user_prompt.into().trim().to_string(),
            assistant_response: assistant_response.into().trim().to_string(),
            timestamp_ms: now_epoch_ms(),
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

        let exchange = AiSessionExchange::new(intent, provider, prompt, response);
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
        serde_json::from_str(&json).map_err(json_to_io_error)
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
    use tempfile::tempdir;

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
}
