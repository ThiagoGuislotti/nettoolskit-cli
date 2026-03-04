//! Rich interactive state for CLI sessions.
//!
//! Phase 3.1 foundation:
//! - Shared session state (`Arc<RwLock<_>>`)
//! - Structured history entries (command/text)
//! - JSON serialization/deserialization for future persistence phases

use nettoolskit_core::AppConfig;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

/// Shared state handle used by interactive runtime components.
pub type SharedCliState = Arc<RwLock<CliState>>;
const LOCAL_SESSIONS_DIR_NAME: &str = "sessions";
const LOCAL_SESSION_FILE_EXTENSION: &str = "json";

/// Entry kind for interactive history.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum HistoryEntryKind {
    /// Slash-command entry.
    Command,
    /// Free-text entry.
    Text,
}

/// Common contract for history entries.
pub trait HistoryEntry {
    /// Entry classification.
    fn kind(&self) -> HistoryEntryKind;
    /// User-facing value of this entry.
    fn value(&self) -> &str;
    /// UTC timestamp in epoch milliseconds.
    fn timestamp_ms(&self) -> u64;
}

/// Structured session history entry.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionHistoryEntry {
    kind: HistoryEntryKind,
    value: String,
    timestamp_ms: u64,
}

impl SessionHistoryEntry {
    /// Build a history entry with current UTC timestamp.
    pub fn new(kind: HistoryEntryKind, value: impl Into<String>) -> Self {
        Self {
            kind,
            value: value.into(),
            timestamp_ms: now_epoch_ms(),
        }
    }
}

impl HistoryEntry for SessionHistoryEntry {
    fn kind(&self) -> HistoryEntryKind {
        self.kind
    }

    fn value(&self) -> &str {
        &self.value
    }

    fn timestamp_ms(&self) -> u64 {
        self.timestamp_ms
    }
}

/// Session metadata tracked in state snapshots.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionMetadata {
    /// Session identifier.
    pub id: String,
    /// Session start time in epoch milliseconds.
    pub started_at_ms: u64,
    /// Maximum retained entries in history.
    pub history_capacity: usize,
}

/// Metadata for a local persisted session snapshot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalSessionSnapshot {
    /// Session identifier.
    pub id: String,
    /// Session start time in epoch milliseconds.
    pub started_at_ms: u64,
    /// Last known activity timestamp in epoch milliseconds.
    pub last_activity_ms: u64,
    /// Number of persisted history entries.
    pub history_entries: usize,
    /// Snapshot JSON file path.
    pub path: PathBuf,
}

/// Serializable interactive session state.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CliState {
    /// Session metadata.
    pub session: SessionMetadata,
    /// Effective runtime config snapshot.
    pub config: AppConfig,
    /// Structured history entries.
    pub history: VecDeque<SessionHistoryEntry>,
}

impl CliState {
    /// Create a new CLI state instance.
    pub fn new(session_id: impl Into<String>, config: AppConfig, history_capacity: usize) -> Self {
        Self {
            session: SessionMetadata {
                id: session_id.into(),
                started_at_ms: now_epoch_ms(),
                history_capacity,
            },
            config,
            history: VecDeque::with_capacity(history_capacity),
        }
    }

    /// Create a shared state handle.
    pub fn shared(
        session_id: impl Into<String>,
        config: AppConfig,
        history_capacity: usize,
    ) -> SharedCliState {
        Arc::new(RwLock::new(Self::new(session_id, config, history_capacity)))
    }

    /// Convert this state into a shared handle.
    pub fn into_shared(self) -> SharedCliState {
        Arc::new(RwLock::new(self))
    }

    /// Record command entry in bounded history.
    pub fn push_command(&mut self, value: &str) -> bool {
        self.push_entry(HistoryEntryKind::Command, value)
    }

    /// Record text entry in bounded history.
    pub fn push_text(&mut self, value: &str) -> bool {
        self.push_entry(HistoryEntryKind::Text, value)
    }

    /// Render history lines for interactive viewer.
    pub fn history_lines(&self) -> VecDeque<String> {
        self.history
            .iter()
            .map(|entry| entry.value().to_string())
            .collect()
    }

    /// Serialize full state to JSON string.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Serialize full state to formatted JSON string.
    pub fn to_json_pretty(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Deserialize full state from JSON string.
    pub fn from_json(input: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(input)
    }

    /// Save this session snapshot into local application data directory.
    ///
    /// Returns `Ok(None)` when the operating system does not expose a data directory.
    ///
    /// # Errors
    ///
    /// Returns `Err` when creating directories or writing the snapshot fails.
    pub fn save_local_snapshot(&self) -> io::Result<Option<PathBuf>> {
        let Some(base_dir) = AppConfig::default_data_dir() else {
            return Ok(None);
        };

        self.save_local_snapshot_to_dir(&base_dir).map(Some)
    }

    /// Load the most recent local snapshot from the default data directory.
    ///
    /// Returns `Ok(None)` when no snapshots are available.
    ///
    /// # Errors
    ///
    /// Returns `Err` when reading the local sessions directory fails.
    pub fn load_latest_local_snapshot() -> io::Result<Option<Self>> {
        let Some(base_dir) = AppConfig::default_data_dir() else {
            return Ok(None);
        };

        Self::load_latest_local_snapshot_from_dir(&base_dir)
    }

    /// Load a local snapshot from an explicit snapshot file path.
    ///
    /// # Errors
    ///
    /// Returns `Err` when the file cannot be read or parsed as `CliState`.
    pub fn load_local_snapshot_from_path(path: &Path) -> io::Result<Self> {
        Self::load_snapshot_from_path(path)
    }

    /// List local snapshots sorted from most-recent to oldest.
    ///
    /// Returns `Ok(None)` when the operating system data directory is unavailable.
    ///
    /// # Errors
    ///
    /// Returns `Err` when listing snapshot files fails.
    pub fn list_local_snapshots(limit: usize) -> io::Result<Option<Vec<LocalSessionSnapshot>>> {
        let Some(base_dir) = AppConfig::default_data_dir() else {
            return Ok(None);
        };

        Self::list_local_snapshots_in_dir(&base_dir, limit).map(Some)
    }

    /// Prune local snapshots, keeping only the most recent `keep_latest`.
    ///
    /// Returns `Ok(None)` when the operating system data directory is unavailable.
    ///
    /// # Errors
    ///
    /// Returns `Err` when deleting old snapshot files fails.
    pub fn prune_local_snapshots(keep_latest: usize) -> io::Result<Option<usize>> {
        let Some(base_dir) = AppConfig::default_data_dir() else {
            return Ok(None);
        };

        Self::prune_local_snapshots_in_dir(&base_dir, keep_latest).map(Some)
    }

    fn push_entry(&mut self, kind: HistoryEntryKind, value: &str) -> bool {
        let normalized = value.trim();
        if normalized.is_empty() {
            return false;
        }

        if self.history.len() == self.session.history_capacity {
            self.history.pop_front();
        }

        self.history
            .push_back(SessionHistoryEntry::new(kind, normalized.to_string()));
        true
    }

    fn save_local_snapshot_to_dir(&self, base_dir: &Path) -> io::Result<PathBuf> {
        let sessions_dir = base_dir.join(LOCAL_SESSIONS_DIR_NAME);
        fs::create_dir_all(&sessions_dir)?;

        let file_name = format!(
            "{}.{LOCAL_SESSION_FILE_EXTENSION}",
            sanitize_session_id(&self.session.id)
        );
        let snapshot_path = sessions_dir.join(file_name);
        let json = self.to_json_pretty().map_err(json_to_io_error)?;
        fs::write(&snapshot_path, json)?;
        Ok(snapshot_path)
    }

    fn load_snapshot_from_path(path: &Path) -> io::Result<Self> {
        let json = fs::read_to_string(path)?;
        Self::from_json(&json).map_err(json_to_io_error)
    }

    fn load_latest_local_snapshot_from_dir(base_dir: &Path) -> io::Result<Option<Self>> {
        let snapshots = Self::list_local_snapshots_in_dir(base_dir, usize::MAX)?;
        let Some(latest) = snapshots.first() else {
            return Ok(None);
        };

        Self::load_snapshot_from_path(&latest.path).map(Some)
    }

    fn list_local_snapshots_in_dir(
        base_dir: &Path,
        limit: usize,
    ) -> io::Result<Vec<LocalSessionSnapshot>> {
        if limit == 0 {
            return Ok(Vec::new());
        }

        let sessions_dir = base_dir.join(LOCAL_SESSIONS_DIR_NAME);
        if !sessions_dir.exists() {
            return Ok(Vec::new());
        }

        let mut snapshots = Vec::new();
        for entry in fs::read_dir(sessions_dir)? {
            let entry = entry?;
            let path = entry.path();
            if !is_json_snapshot_file(&path) {
                continue;
            }

            let state = match Self::load_snapshot_from_path(&path) {
                Ok(state) => state,
                Err(_) => continue,
            };

            snapshots.push(LocalSessionSnapshot::from_state(path, &state));
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

impl LocalSessionSnapshot {
    fn from_state(path: PathBuf, state: &CliState) -> Self {
        let last_activity_ms = state
            .history
            .back()
            .map_or(state.session.started_at_ms, HistoryEntry::timestamp_ms);

        Self {
            id: state.session.id.clone(),
            started_at_ms: state.session.started_at_ms,
            last_activity_ms,
            history_entries: state.history.len(),
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

fn sanitize_session_id(session_id: &str) -> String {
    let mut sanitized: String = session_id
        .trim()
        .chars()
        .map(|value| match value {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' => value,
            _ => '_',
        })
        .collect();

    if sanitized.is_empty() {
        sanitized.push_str("session");
    }

    sanitized
}

fn is_json_snapshot_file(path: &Path) -> bool {
    path.is_file()
        && path
            .extension()
            .and_then(|value| value.to_str())
            .is_some_and(|value| value.eq_ignore_ascii_case(LOCAL_SESSION_FILE_EXTENSION))
}

fn json_to_io_error(error: serde_json::Error) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, error)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn cli_state_pushes_bounded_command_and_text_entries() {
        let mut state = CliState::new("session-x", AppConfig::default(), 3);
        assert!(state.push_command("/help"));
        assert!(state.push_text("hello"));
        assert!(state.push_command("/manifest list"));
        assert!(state.push_text("trimmed  "));

        assert_eq!(state.history.len(), 3);
        assert_eq!(state.history[0].value(), "hello");
        assert_eq!(state.history[0].kind(), HistoryEntryKind::Text);
        assert_eq!(state.history[2].value(), "trimmed");
        assert_eq!(state.history[2].kind(), HistoryEntryKind::Text);
    }

    #[test]
    fn cli_state_ignores_blank_entries() {
        let mut state = CliState::new("session-y", AppConfig::default(), 2);
        assert!(!state.push_command("   "));
        assert!(!state.push_text(""));
        assert!(state.history.is_empty());
    }

    #[test]
    fn cli_state_json_roundtrip_preserves_content() {
        let mut state = CliState::new("session-z", AppConfig::default(), 8);
        state.push_command("/help");
        state.push_text("payload");

        let json = state.to_json().expect("must serialize");
        let decoded = CliState::from_json(&json).expect("must deserialize");

        assert_eq!(decoded.session.id, "session-z");
        assert_eq!(decoded.session.history_capacity, 8);
        assert_eq!(decoded.history.len(), 2);
        assert_eq!(decoded.history[0].value(), "/help");
        assert_eq!(decoded.history[1].value(), "payload");
    }

    #[test]
    fn shared_cli_state_supports_concurrent_handle_access() {
        let shared = CliState::shared("session-shared", AppConfig::default(), 10);
        {
            let mut guard = shared.write().expect("write lock");
            guard.push_command("/help");
            guard.push_text("hello");
        }

        let guard = shared.read().expect("read lock");
        assert_eq!(guard.history.len(), 2);
        assert_eq!(guard.history_lines().len(), 2);
    }

    #[test]
    fn local_snapshot_roundtrip_in_custom_dir() {
        let temp = tempdir().expect("temp dir");
        let mut state = CliState::new("session-local", AppConfig::default(), 8);
        state.push_command("/help");
        state.push_text("hello");

        let snapshot_path = state
            .save_local_snapshot_to_dir(temp.path())
            .expect("snapshot path");
        assert!(snapshot_path.exists());

        let loaded = CliState::load_snapshot_from_path(&snapshot_path).expect("load snapshot");
        assert_eq!(loaded.session.id, "session-local");
        assert_eq!(loaded.history.len(), 2);
        assert_eq!(loaded.history[0].value(), "/help");
        assert_eq!(loaded.history[1].value(), "hello");
    }

    #[test]
    fn local_snapshot_load_latest_prefers_most_recent_activity() {
        let temp = tempdir().expect("temp dir");

        let mut older = CliState::new("session-1", AppConfig::default(), 4);
        older.session.started_at_ms = 1_000;
        older
            .save_local_snapshot_to_dir(temp.path())
            .expect("save older");

        let mut latest = CliState::new("session-2", AppConfig::default(), 4);
        latest.session.started_at_ms = 2_000;
        latest
            .save_local_snapshot_to_dir(temp.path())
            .expect("save latest");

        let loaded = CliState::load_latest_local_snapshot_from_dir(temp.path())
            .expect("load latest")
            .expect("latest snapshot");
        assert_eq!(loaded.session.id, "session-2");
    }

    #[test]
    fn local_snapshot_prune_keeps_latest_sessions() {
        let temp = tempdir().expect("temp dir");

        for idx in 1_u64..=3 {
            let mut state = CliState::new(format!("session-{idx}"), AppConfig::default(), 4);
            state.session.started_at_ms = idx * 100;
            state
                .save_local_snapshot_to_dir(temp.path())
                .expect("save snapshot");
        }

        let removed = CliState::prune_local_snapshots_in_dir(temp.path(), 1).expect("prune");
        assert_eq!(removed, 2);

        let remaining =
            CliState::list_local_snapshots_in_dir(temp.path(), usize::MAX).expect("list");
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].id, "session-3");
    }

    #[test]
    fn sanitize_session_id_replaces_invalid_characters() {
        assert_eq!(sanitize_session_id("session-1"), "session-1");
        assert_eq!(sanitize_session_id(" session:/x "), "session__x");
        assert_eq!(sanitize_session_id("   "), "session");
    }
}
