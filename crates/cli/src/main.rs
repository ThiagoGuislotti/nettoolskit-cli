//! NetToolsKit CLI binary entry point.

use axum::{
    body::Bytes,
    error_handling::HandleErrorLayer,
    extract::{DefaultBodyLimit, Extension, Json, Request, State},
    http::{HeaderMap as AxumHeaderMap, HeaderValue, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
    BoxError, Router,
};
use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::Shell;
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use nettoolskit_cli::{interactive_mode, InteractiveOptions};
use nettoolskit_core::{
    AppConfig, ApprovalState, ColorMode, CommandEntry, ControlEnvelope, ControlPolicyContext,
    IngressTransport, OperatorContext, OperatorKind, RuntimeMode, SessionContext, SessionKind,
    TaskIntent, TaskIntentKind, UnicodeMode,
};
use nettoolskit_orchestrator::ExitStatus;
use nettoolskit_otel::{
    init_tracing_with_config, next_correlation_id, shutdown_tracing, TracingConfig,
};
use nettoolskit_ui::{set_color_override, set_unicode_override, ColorLevel};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use tower::{timeout::TimeoutLayer, ServiceBuilder};
use tracing::{info, info_span};

#[cfg(test)]
use axum::body::Body;
#[cfg(test)]
use tower::ServiceExt;

const NTK_CHATOPS_TELEGRAM_WEBHOOK_SECRET_TOKEN_ENV: &str =
    "NTK_CHATOPS_TELEGRAM_WEBHOOK_SECRET_TOKEN";
const NTK_CHATOPS_DISCORD_INTERACTIONS_PUBLIC_KEY_ENV: &str =
    "NTK_CHATOPS_DISCORD_INTERACTIONS_PUBLIC_KEY";
const NTK_SERVICE_AUTH_TOKEN_ENV: &str = "NTK_SERVICE_AUTH_TOKEN";
const NTK_CHATOPS_INGRESS_REPLAY_WINDOW_SECONDS_ENV: &str =
    "NTK_CHATOPS_INGRESS_REPLAY_WINDOW_SECONDS";
const NTK_CHATOPS_INGRESS_REPLAY_MAX_ENTRIES_ENV: &str = "NTK_CHATOPS_INGRESS_REPLAY_MAX_ENTRIES";
const NTK_CHATOPS_INGRESS_REPLAY_BACKEND_ENV: &str = "NTK_CHATOPS_INGRESS_REPLAY_BACKEND";
const NTK_CHATOPS_INGRESS_REPLAY_FILE_PATH_ENV: &str = "NTK_CHATOPS_INGRESS_REPLAY_FILE_PATH";
const NTK_SERVICE_HTTP_TIMEOUT_MS_ENV: &str = "NTK_SERVICE_HTTP_TIMEOUT_MS";
const DEFAULT_CHATOPS_INGRESS_REPLAY_WINDOW_SECONDS: u64 = 300;
const DEFAULT_CHATOPS_INGRESS_REPLAY_MAX_ENTRIES: usize = 4_096;
const DEFAULT_CHATOPS_INGRESS_REPLAY_FILE_LOCK_WAIT_MS: u64 = 1_000;
const DEFAULT_CHATOPS_INGRESS_REPLAY_FILE_LOCK_STALE_SECONDS: u64 = 30;
const DEFAULT_SERVICE_HTTP_TIMEOUT_MS: u64 = 30_000;
const MIN_SERVICE_HTTP_TIMEOUT_MS: u64 = 100;
const MAX_SERVICE_HTTP_TIMEOUT_MS: u64 = 300_000;
const DEFAULT_SERVICE_HTTP_BODY_LIMIT_BYTES: usize = 32 * 1024;

/// Global arguments available across all commands
#[derive(Debug, Clone, Parser)]
pub struct GlobalArgs {
    /// Set logging level (off, error, warn, info, debug, trace)
    #[clap(long, global = true)]
    pub log_level: Option<String>,

    /// Path to configuration file
    #[clap(long, global = true)]
    pub config: Option<String>,

    /// Enable verbose output
    #[clap(short, long, global = true)]
    pub verbose: bool,
}

/// Available CLI commands
#[derive(Debug, Parser)]
pub enum Commands {
    /// Manage and apply manifests
    Manifest {
        /// Optional manifest subcommand. If omitted, opens interactive submenu.
        #[clap(subcommand)]
        command: Option<ManifestCommand>,
    },

    /// Generate shell completions for the specified shell
    Completions {
        /// Target shell (bash, zsh, fish, powershell)
        #[clap(value_enum)]
        shell: Shell,
    },

    /// Run background service mode with HTTP health and task submission endpoints
    Service {
        /// Bind host for the service listener
        #[clap(long, default_value = "127.0.0.1")]
        host: String,

        /// Bind port for the service listener
        #[clap(long, default_value_t = 8080)]
        port: u16,
    },
}

#[derive(Debug, Deserialize)]
struct ServiceTaskSubmitRequest {
    intent: String,
    payload: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ServiceTaskSubmitResponse {
    accepted: bool,
    exit_status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    task_id: Option<String>,
    request_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    correlation_id: Option<String>,
    operator_id: String,
    operator_kind: String,
    session_id: String,
    transport: String,
}

#[derive(Debug, Serialize)]
struct ServiceTelegramWebhookResponse {
    accepted: bool,
    queued: usize,
}

#[derive(Debug, Serialize)]
struct ServiceDiscordInteractionData {
    content: String,
    flags: u64,
}

#[derive(Debug, Serialize)]
struct ServiceDiscordInteractionResponse {
    #[serde(rename = "type")]
    response_type: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<ServiceDiscordInteractionData>,
}

#[derive(Debug, Serialize)]
struct ServiceHealthResponse {
    status: String,
    runtime_mode: String,
    uptime_seconds: u64,
    version: String,
}

#[derive(Debug, Serialize)]
struct ServiceReadinessCheck {
    name: String,
    ready: bool,
    detail: String,
}

#[derive(Debug, Serialize)]
struct ServiceReadinessResponse {
    status: String,
    runtime_mode: String,
    uptime_seconds: u64,
    version: String,
    checks: Vec<ServiceReadinessCheck>,
}

#[derive(Clone)]
struct ServiceRuntimeState {
    started_at: std::time::Instant,
    chatops_runtime: InitializedChatOpsRuntime,
    ingress_security: Arc<ServiceIngressSecurityConfig>,
    replay_guard: Arc<IngressReplayGuard>,
    service_auth_token: Option<String>,
    data_dir: Option<std::path::PathBuf>,
}

#[derive(Debug, Clone)]
struct ServiceRequestContext {
    request_id: String,
    correlation_id: Option<String>,
}

#[derive(Clone)]
struct InitializedChatOpsRuntime {
    runtime: Option<Arc<nettoolskit_orchestrator::ChatOpsRuntime>>,
    config: nettoolskit_orchestrator::ChatOpsRuntimeConfig,
    startup_error: Option<String>,
}

#[derive(Debug)]
struct ServiceIngressSecurityConfig {
    telegram_secret_token: Option<String>,
    discord_verifying_key: Option<VerifyingKey>,
    replay_window: std::time::Duration,
    replay_max_entries: usize,
    replay_backend: IngressReplayBackendConfig,
}

#[derive(Debug)]
struct IngressReplayGuard {
    replay_window_ms: u64,
    max_entries: usize,
    backend: IngressReplayBackend,
}

#[derive(Debug)]
enum IngressReplayBackend {
    Memory {
        entries: Mutex<HashMap<String, u64>>,
    },
    File {
        path: std::path::PathBuf,
    },
}

#[derive(Debug, Clone)]
enum IngressReplayBackendConfig {
    Memory,
    File { path: std::path::PathBuf },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct IngressReplayRecord {
    key: String,
    seen_at_unix_ms: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum IngressSecurityError {
    Unauthorized,
    Replay,
    Unavailable,
}

#[derive(Debug, Deserialize)]
struct TelegramUpdateReplayEnvelope {
    update_id: Option<i64>,
}

/// Non-interactive manifest subcommands.
#[derive(Debug, Subcommand)]
pub enum ManifestCommand {
    /// Discover available manifests in the workspace.
    List,
    /// Validate manifest structure and dependencies.
    Check {
        /// Path to manifest file (required for deterministic validation).
        path: String,
        /// Validate as template file instead of manifest YAML.
        #[clap(long)]
        template: bool,
    },
    /// Preview generated files without applying changes.
    Render {
        /// Path to manifest file.
        path: String,
        /// Keep operation in dry-run mode (preview only).
        #[clap(long)]
        dry_run: bool,
        /// Optional output root directory for rendering preview.
        #[clap(long)]
        output: Option<String>,
    },
    /// Apply a manifest file to generate/update project files.
    Apply {
        /// Path to manifest file.
        path: String,
        /// Optional output root directory.
        #[clap(long)]
        output: Option<String>,
        /// Run without writing changes.
        #[clap(long)]
        dry_run: bool,
    },
}

impl Commands {
    /// Execute this command
    pub async fn execute(self) -> ExitStatus {
        use nettoolskit_orchestrator::{process_command, MainAction};

        match self {
            Commands::Manifest { command } => match command {
                None => process_command(&MainAction::Manifest.slash_static()).await,
                Some(ManifestCommand::List) => process_command("/manifest list").await,
                Some(ManifestCommand::Check { path, template }) => {
                    let mut command_line = format!("/manifest check {path}");
                    if template {
                        command_line.push_str(" --template");
                    }
                    process_command(&command_line).await
                }
                Some(ManifestCommand::Render {
                    path,
                    dry_run,
                    output,
                }) => {
                    let cmd = if dry_run {
                        format!("/manifest render {path} --dry-run")
                    } else {
                        format!("/manifest render {path}")
                    };
                    let mut command_line = cmd;
                    if let Some(output_dir) = output {
                        command_line.push_str(" --output ");
                        command_line.push_str(&output_dir);
                    }
                    process_command(&command_line).await
                }
                Some(ManifestCommand::Apply {
                    path,
                    output,
                    dry_run,
                }) => {
                    let mut command_line = format!("/manifest apply {path}");
                    if dry_run {
                        command_line.push_str(" --dry-run");
                    }
                    if let Some(output_dir) = output {
                        command_line.push_str(" --output ");
                        command_line.push_str(&output_dir);
                    }
                    process_command(&command_line).await
                }
            },
            Commands::Completions { shell } => {
                clap_complete::generate(shell, &mut Cli::command(), "ntk", &mut std::io::stdout());
                ExitStatus::Success
            }
            Commands::Service { host, port } => run_service_mode(host, port).await,
        }
    }
}

impl ServiceIngressSecurityConfig {
    fn from_env() -> Result<Self, String> {
        let telegram_secret_token = std::env::var(NTK_CHATOPS_TELEGRAM_WEBHOOK_SECRET_TOKEN_ENV)
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());

        let discord_verifying_key =
            match std::env::var(NTK_CHATOPS_DISCORD_INTERACTIONS_PUBLIC_KEY_ENV) {
                Ok(value) if !value.trim().is_empty() => {
                    Some(parse_discord_public_key(value.trim())?)
                }
                _ => None,
            };

        let replay_window = std::env::var(NTK_CHATOPS_INGRESS_REPLAY_WINDOW_SECONDS_ENV)
            .ok()
            .and_then(|value| value.trim().parse::<u64>().ok())
            .map(|seconds| std::time::Duration::from_secs(seconds.max(1)))
            .unwrap_or_else(|| {
                std::time::Duration::from_secs(DEFAULT_CHATOPS_INGRESS_REPLAY_WINDOW_SECONDS)
            });
        let replay_max_entries = std::env::var(NTK_CHATOPS_INGRESS_REPLAY_MAX_ENTRIES_ENV)
            .ok()
            .and_then(|value| value.trim().parse::<usize>().ok())
            .map(|entries| entries.max(128))
            .unwrap_or(DEFAULT_CHATOPS_INGRESS_REPLAY_MAX_ENTRIES);
        let replay_backend = resolve_replay_backend_from_env()?;

        Ok(Self {
            telegram_secret_token,
            discord_verifying_key,
            replay_window,
            replay_max_entries,
            replay_backend,
        })
    }

    const fn has_telegram_signature_validation(&self) -> bool {
        self.telegram_secret_token.is_some()
    }

    const fn has_discord_signature_validation(&self) -> bool {
        self.discord_verifying_key.is_some()
    }

    fn replay_backend_description(&self) -> String {
        match &self.replay_backend {
            IngressReplayBackendConfig::Memory => "memory".to_string(),
            IngressReplayBackendConfig::File { path } => format!("file ({})", path.display()),
        }
    }
}

fn resolve_replay_backend_from_env() -> Result<IngressReplayBackendConfig, String> {
    let backend_value = std::env::var(NTK_CHATOPS_INGRESS_REPLAY_BACKEND_ENV)
        .unwrap_or_else(|_| "memory".to_string());
    match backend_value.trim().to_ascii_lowercase().as_str() {
        "" | "memory" | "in_memory" | "in-memory" => Ok(IngressReplayBackendConfig::Memory),
        "file" => {
            let configured_path = std::env::var(NTK_CHATOPS_INGRESS_REPLAY_FILE_PATH_ENV)
                .ok()
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty());
            let path = configured_path
                .map(std::path::PathBuf::from)
                .unwrap_or_else(default_replay_file_path);
            Ok(IngressReplayBackendConfig::File { path })
        }
        other => Err(format!(
            "unsupported replay backend `{other}` (expected `memory` or `file`)"
        )),
    }
}

fn default_replay_file_path() -> std::path::PathBuf {
    AppConfig::default_data_dir()
        .map(|base| base.join("chatops").join("ingress-replay-cache.json"))
        .unwrap_or_else(|| {
            std::path::PathBuf::from(".temp")
                .join("chatops")
                .join("ingress-replay-cache.json")
        })
}

impl IngressReplayGuard {
    fn with_backend(
        window: std::time::Duration,
        max_entries: usize,
        backend: IngressReplayBackendConfig,
    ) -> Self {
        Self {
            replay_window_ms: u64::try_from(window.as_millis()).unwrap_or(u64::MAX).max(1),
            max_entries: max_entries.max(128),
            backend: match backend {
                IngressReplayBackendConfig::Memory => IngressReplayBackend::Memory {
                    entries: Mutex::new(HashMap::new()),
                },
                IngressReplayBackendConfig::File { path } => IngressReplayBackend::File { path },
            },
        }
    }

    fn check_and_record(
        &self,
        replay_key: &str,
        now_unix_ms: u64,
    ) -> Result<(), IngressSecurityError> {
        match &self.backend {
            IngressReplayBackend::Memory { entries } => {
                let mut entries = entries
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                entries.retain(|_, seen_at| {
                    now_unix_ms.saturating_sub(*seen_at) <= self.replay_window_ms
                });
                if entries.contains_key(replay_key) {
                    return Err(IngressSecurityError::Replay);
                }
                entries.insert(replay_key.to_string(), now_unix_ms);
                trim_replay_entries(&mut entries, self.max_entries);
                Ok(())
            }
            IngressReplayBackend::File { path } => {
                self.check_and_record_file(path, replay_key, now_unix_ms)
            }
        }
    }

    fn check_and_record_file(
        &self,
        path: &std::path::Path,
        replay_key: &str,
        now_unix_ms: u64,
    ) -> Result<(), IngressSecurityError> {
        let lock_path = replay_lock_path(path);
        let _lock = acquire_replay_file_lock(
            &lock_path,
            std::time::Duration::from_millis(DEFAULT_CHATOPS_INGRESS_REPLAY_FILE_LOCK_WAIT_MS),
            std::time::Duration::from_secs(DEFAULT_CHATOPS_INGRESS_REPLAY_FILE_LOCK_STALE_SECONDS),
        )?;

        let mut entries = load_replay_entries_from_file(path)?;
        entries.retain(|_, seen_at| now_unix_ms.saturating_sub(*seen_at) <= self.replay_window_ms);
        if entries.contains_key(replay_key) {
            return Err(IngressSecurityError::Replay);
        }
        entries.insert(replay_key.to_string(), now_unix_ms);
        trim_replay_entries(&mut entries, self.max_entries);
        save_replay_entries_to_file(path, &entries)
    }
}

fn trim_replay_entries(entries: &mut HashMap<String, u64>, max_entries: usize) {
    while entries.len() > max_entries {
        let Some(oldest_key) = entries
            .iter()
            .min_by_key(|(_, seen_at)| **seen_at)
            .map(|(key, _)| key.clone())
        else {
            break;
        };
        entries.remove(&oldest_key);
    }
}

fn replay_lock_path(path: &std::path::Path) -> std::path::PathBuf {
    let file_name = path
        .file_name()
        .and_then(std::ffi::OsStr::to_str)
        .unwrap_or("ingress-replay-cache.json");
    path.with_file_name(format!("{file_name}.lock"))
}

#[derive(Debug)]
struct ReplayFileLockGuard {
    lock_path: std::path::PathBuf,
}

impl Drop for ReplayFileLockGuard {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.lock_path);
    }
}

fn acquire_replay_file_lock(
    lock_path: &std::path::Path,
    wait_timeout: std::time::Duration,
    stale_after: std::time::Duration,
) -> Result<ReplayFileLockGuard, IngressSecurityError> {
    let started_at = std::time::Instant::now();
    loop {
        match std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(lock_path)
        {
            Ok(mut file) => {
                let _ = std::io::Write::write_all(&mut file, b"lock");
                return Ok(ReplayFileLockGuard {
                    lock_path: lock_path.to_path_buf(),
                });
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                if is_stale_lock(lock_path, stale_after) {
                    let _ = std::fs::remove_file(lock_path);
                    continue;
                }
                if started_at.elapsed() >= wait_timeout {
                    return Err(IngressSecurityError::Unavailable);
                }
                std::thread::sleep(std::time::Duration::from_millis(15));
            }
            Err(_) => return Err(IngressSecurityError::Unavailable),
        }
    }
}

fn is_stale_lock(path: &std::path::Path, stale_after: std::time::Duration) -> bool {
    let Ok(metadata) = std::fs::metadata(path) else {
        return false;
    };
    let Ok(modified) = metadata.modified() else {
        return false;
    };
    let Ok(elapsed) = modified.elapsed() else {
        return false;
    };
    elapsed >= stale_after
}

fn load_replay_entries_from_file(
    path: &std::path::Path,
) -> Result<HashMap<String, u64>, IngressSecurityError> {
    if !path.exists() {
        return Ok(HashMap::new());
    }
    let content = std::fs::read_to_string(path).map_err(|_| IngressSecurityError::Unavailable)?;
    if content.trim().is_empty() {
        return Ok(HashMap::new());
    }
    let records: Vec<IngressReplayRecord> =
        serde_json::from_str(&content).map_err(|_| IngressSecurityError::Unavailable)?;
    let mut entries = HashMap::new();
    for record in records {
        if !record.key.trim().is_empty() {
            entries.insert(record.key, record.seen_at_unix_ms);
        }
    }
    Ok(entries)
}

fn save_replay_entries_to_file(
    path: &std::path::Path,
    entries: &HashMap<String, u64>,
) -> Result<(), IngressSecurityError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|_| IngressSecurityError::Unavailable)?;
    }
    let mut records: Vec<IngressReplayRecord> = entries
        .iter()
        .map(|(key, seen_at_unix_ms)| IngressReplayRecord {
            key: key.clone(),
            seen_at_unix_ms: *seen_at_unix_ms,
        })
        .collect();
    records.sort_by_key(|record| record.seen_at_unix_ms);
    let serialized =
        serde_json::to_string(&records).map_err(|_| IngressSecurityError::Unavailable)?;
    std::fs::write(path, serialized).map_err(|_| IngressSecurityError::Unavailable)
}

fn parse_discord_public_key(value: &str) -> Result<VerifyingKey, String> {
    let decoded =
        hex::decode(value).map_err(|error| format!("invalid Discord public key hex: {error}"))?;
    let key_bytes: [u8; 32] = decoded
        .try_into()
        .map_err(|_| "Discord public key must be 32 bytes (64 hex chars)".to_string())?;
    VerifyingKey::from_bytes(&key_bytes)
        .map_err(|error| format!("invalid Discord public key bytes: {error}"))
}

#[cfg(test)]
fn parse_http_headers(request: &str) -> HashMap<String, String> {
    let mut headers = HashMap::new();
    for line in request.lines().skip(1) {
        let line = line.trim_end_matches('\r');
        if line.is_empty() {
            break;
        }
        if let Some((name, value)) = line.split_once(':') {
            headers.insert(name.trim().to_ascii_lowercase(), value.trim().to_string());
        }
    }
    headers
}

fn normalize_http_headers(headers: &AxumHeaderMap) -> HashMap<String, String> {
    let mut normalized = HashMap::new();
    for (name, value) in headers {
        if let Ok(value) = value.to_str() {
            normalized.insert(name.as_str().to_ascii_lowercase(), value.trim().to_string());
        }
    }
    normalized
}

fn request_header<'a>(headers: &'a HashMap<String, String>, key: &str) -> Option<&'a str> {
    headers
        .get(&key.to_ascii_lowercase())
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
}

fn current_unix_timestamp_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or_default()
}

fn current_unix_timestamp_seconds() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default()
}

fn telegram_replay_key(body: &str) -> String {
    if let Ok(payload) = serde_json::from_str::<TelegramUpdateReplayEnvelope>(body) {
        if let Some(update_id) = payload.update_id {
            return format!("telegram:update:{update_id}");
        }
    }

    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    body.hash(&mut hasher);
    format!("telegram:body:{:016x}", hasher.finish())
}

fn verify_telegram_ingress_security(
    state: &ServiceRuntimeState,
    headers: &HashMap<String, String>,
    body: &str,
) -> Result<(), IngressSecurityError> {
    if let Some(expected_secret) = &state.ingress_security.telegram_secret_token {
        let Some(header_secret) = request_header(headers, "x-telegram-bot-api-secret-token") else {
            return Err(IngressSecurityError::Unauthorized);
        };
        if header_secret != expected_secret {
            return Err(IngressSecurityError::Unauthorized);
        }
    }

    let replay_key = telegram_replay_key(body);
    state
        .replay_guard
        .check_and_record(&replay_key, current_unix_timestamp_ms())
}

fn verify_discord_interaction_ingress_security(
    state: &ServiceRuntimeState,
    headers: &HashMap<String, String>,
    body: &str,
) -> Result<(), IngressSecurityError> {
    if let Some(verifying_key) = &state.ingress_security.discord_verifying_key {
        let Some(signature_hex) = request_header(headers, "x-signature-ed25519") else {
            return Err(IngressSecurityError::Unauthorized);
        };
        let Some(timestamp) = request_header(headers, "x-signature-timestamp") else {
            return Err(IngressSecurityError::Unauthorized);
        };
        let signature_bytes =
            hex::decode(signature_hex).map_err(|_| IngressSecurityError::Unauthorized)?;
        let signature = Signature::try_from(signature_bytes.as_slice())
            .map_err(|_| IngressSecurityError::Unauthorized)?;

        let mut signed_message = timestamp.as_bytes().to_vec();
        signed_message.extend_from_slice(body.as_bytes());
        verifying_key
            .verify(&signed_message, &signature)
            .map_err(|_| IngressSecurityError::Unauthorized)?;

        let timestamp_secs = timestamp
            .parse::<u64>()
            .map_err(|_| IngressSecurityError::Unauthorized)?;
        let now_secs = current_unix_timestamp_seconds();
        let allowed_drift = state.ingress_security.replay_window.as_secs().max(1);
        if now_secs.abs_diff(timestamp_secs) > allowed_drift {
            return Err(IngressSecurityError::Unauthorized);
        }

        let replay_key = format!("discord:{timestamp}:{signature_hex}");
        state
            .replay_guard
            .check_and_record(&replay_key, current_unix_timestamp_ms())?;
    }

    Ok(())
}

#[cfg(test)]
fn parse_http_request_line(request: &str) -> Option<(&str, &str)> {
    let line = request.lines().next()?.trim();
    let mut parts = line.split_whitespace();
    let method = parts.next()?;
    let path = parts.next()?;
    Some((method, path))
}

#[cfg(test)]
fn request_body(request: &str) -> &str {
    request.split_once("\r\n\r\n").map_or("", |(_, body)| body)
}

#[cfg(test)]
fn build_http_response(status: &str, content_type: &str, body: &str) -> String {
    format!(
        "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    )
}

fn service_auth_token_from_env() -> Option<String> {
    std::env::var(NTK_SERVICE_AUTH_TOKEN_ENV)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn parse_bearer_token(value: &str) -> Option<&str> {
    let (scheme, token) = value.split_once(' ')?;
    if !scheme.eq_ignore_ascii_case("bearer") {
        return None;
    }

    let token = token.trim();
    if token.is_empty() {
        None
    } else {
        Some(token)
    }
}

fn request_bearer_token(headers: &HashMap<String, String>) -> Option<&str> {
    request_header(headers, "authorization").and_then(parse_bearer_token)
}

fn service_task_intent_kind(intent: &str) -> TaskIntentKind {
    TaskIntentKind::from_alias(intent).unwrap_or(TaskIntentKind::CommandExecution)
}

fn service_request_context_from_headers(headers: &AxumHeaderMap) -> ServiceRequestContext {
    let request_id = headers
        .get("x-request-id")
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_else(next_service_request_id);
    let correlation_id = headers
        .get("x-correlation-id")
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned);

    ServiceRequestContext {
        request_id,
        correlation_id,
    }
}

fn parse_service_operator_scopes(headers: &HashMap<String, String>) -> Vec<String> {
    let mut scopes = request_header(headers, "x-ntk-operator-scopes")
        .map(|value| {
            value
                .split(',')
                .map(str::trim)
                .filter(|scope| !scope.is_empty())
                .map(ToOwned::to_owned)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    scopes.push("task.submit".to_string());
    scopes
}

fn build_service_control_policy() -> ControlPolicyContext {
    ControlPolicyContext::new(ApprovalState::NotRequired, true)
}

fn build_service_control_envelope(
    request_context: &ServiceRequestContext,
    headers: &HashMap<String, String>,
    intent: &str,
    payload: &str,
) -> ControlEnvelope {
    let has_bearer_token = request_bearer_token(headers).is_some();
    let operator_kind = if has_bearer_token {
        OperatorKind::RemoteHuman
    } else {
        OperatorKind::LocalHuman
    };
    let operator_id = request_header(headers, "x-ntk-operator-id").unwrap_or(if has_bearer_token {
        "service-http-operator"
    } else {
        "local-service-operator"
    });
    let operator = OperatorContext::new(operator_kind, operator_id, IngressTransport::ServiceHttp)
        .with_channel_id(request_header(headers, "x-ntk-channel-id").unwrap_or_default())
        .with_authentication(if has_bearer_token {
            "bearer_token"
        } else {
            "loopback_local"
        })
        .with_scopes(parse_service_operator_scopes(headers));
    let session = SessionContext::new(
        SessionKind::ServiceRequest,
        request_header(headers, "x-ntk-session-id")
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| format!("service-request-{}", request_context.request_id)),
        false,
    );
    let intent = intent.trim();
    let payload = payload.trim();
    let task = TaskIntent::new(
        service_task_intent_kind(intent),
        format!("{intent} task"),
        payload,
    );

    ControlEnvelope::new(
        request_context.request_id.clone(),
        RuntimeMode::Service,
        operator,
        session,
        task,
    )
    .with_correlation_id(request_context.correlation_id.clone().unwrap_or_default())
    .with_policy(build_service_control_policy())
}

fn is_loopback_bind_host(host: &str) -> bool {
    matches!(
        host.trim().to_ascii_lowercase().as_str(),
        "127.0.0.1" | "localhost" | "::1" | "[::1]" | "0:0:0:0:0:0:0:1"
    )
}

fn validate_service_bind_security(
    host: &str,
    service_auth_token: Option<&str>,
) -> Result<(), String> {
    if is_loopback_bind_host(host) || service_auth_token.is_some() {
        return Ok(());
    }

    Err(format!(
        "non-loopback service bind `{host}` requires {NTK_SERVICE_AUTH_TOKEN_ENV} to be configured"
    ))
}

fn service_runtime_mode_label() -> &'static str {
    "service"
}

fn build_service_health_response(state: &ServiceRuntimeState) -> ServiceHealthResponse {
    ServiceHealthResponse {
        status: "ok".to_string(),
        runtime_mode: service_runtime_mode_label().to_string(),
        uptime_seconds: state.started_at.elapsed().as_secs(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    }
}

fn readiness_check(name: &str, ready: bool, detail: impl Into<String>) -> ServiceReadinessCheck {
    ServiceReadinessCheck {
        name: name.to_string(),
        ready,
        detail: detail.into(),
    }
}

fn task_admission_readiness_check() -> ServiceReadinessCheck {
    readiness_check(
        "task_admission",
        true,
        "service task admission pipeline is available",
    )
}

fn ensure_directory_ready(path: &std::path::Path) -> Result<(), String> {
    std::fs::create_dir_all(path)
        .map_err(|error| format!("create directory `{}`: {error}", path.display()))?;
    if path.is_dir() {
        Ok(())
    } else {
        Err(format!("path `{}` is not a directory", path.display()))
    }
}

fn ensure_appendable_file_ready(path: &std::path::Path) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        ensure_directory_ready(parent)?;
    }

    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|error| format!("open file `{}` for append: {error}", path.display()))?;
    std::io::Write::flush(&mut file)
        .map_err(|error| format!("flush file `{}`: {error}", path.display()))
}

fn local_persistence_readiness_check(
    data_dir: Option<&std::path::PathBuf>,
) -> ServiceReadinessCheck {
    let Some(path) = data_dir else {
        return readiness_check(
            "local_persistence",
            false,
            "default local data directory is unavailable",
        );
    };

    match ensure_directory_ready(path) {
        Ok(()) => readiness_check(
            "local_persistence",
            true,
            format!("data directory ready at {}", path.display()),
        ),
        Err(error) => readiness_check("local_persistence", false, error),
    }
}

fn replay_backend_readiness_check(config: &IngressReplayBackendConfig) -> ServiceReadinessCheck {
    match config {
        IngressReplayBackendConfig::Memory => {
            readiness_check("replay_backend", true, "memory replay backend is ready")
        }
        IngressReplayBackendConfig::File { path } => {
            let lock_path = replay_lock_path(path);
            match ensure_appendable_file_ready(path)
                .and_then(|()| ensure_appendable_file_ready(&lock_path))
            {
                Ok(()) => readiness_check(
                    "replay_backend",
                    true,
                    format!("file replay backend ready at {}", path.display()),
                ),
                Err(error) => readiness_check("replay_backend", false, error),
            }
        }
    }
}

fn chatops_audit_store_readiness_check(
    chatops_runtime: &InitializedChatOpsRuntime,
    data_dir: Option<&std::path::PathBuf>,
) -> ServiceReadinessCheck {
    if !chatops_runtime.config.enabled {
        return readiness_check("chatops_audit_store", true, "chatops runtime is disabled");
    }

    let audit_path = chatops_runtime.config.audit_path.clone().or_else(|| {
        data_dir.map(|base| {
            base.join(nettoolskit_orchestrator::ChatOpsLocalAuditStore::DEFAULT_RELATIVE_PATH)
        })
    });

    let Some(path) = audit_path else {
        return readiness_check(
            "chatops_audit_store",
            false,
            "chatops audit path could not be resolved",
        );
    };

    match ensure_appendable_file_ready(&path) {
        Ok(()) => readiness_check(
            "chatops_audit_store",
            true,
            format!("chatops audit store ready at {}", path.display()),
        ),
        Err(error) => readiness_check("chatops_audit_store", false, error),
    }
}

fn chatops_runtime_readiness_check(
    chatops_runtime: &InitializedChatOpsRuntime,
) -> ServiceReadinessCheck {
    if !chatops_runtime.config.enabled {
        return readiness_check("chatops_runtime", true, "chatops runtime is disabled");
    }

    if let Some(error) = &chatops_runtime.startup_error {
        return readiness_check("chatops_runtime", false, error.clone());
    }

    match &chatops_runtime.runtime {
        Some(runtime) => {
            let platforms = runtime
                .enabled_platforms()
                .into_iter()
                .map(|platform| platform.to_string())
                .collect::<Vec<_>>();
            let detail = if platforms.is_empty() {
                "chatops runtime started with no active adapters".to_string()
            } else {
                format!("chatops runtime ready for {}", platforms.join(", "))
            };
            readiness_check("chatops_runtime", true, detail)
        }
        None => readiness_check(
            "chatops_runtime",
            false,
            "chatops runtime is enabled but not initialized",
        ),
    }
}

fn build_service_readiness_response(state: &ServiceRuntimeState) -> ServiceReadinessResponse {
    let checks = vec![
        task_admission_readiness_check(),
        local_persistence_readiness_check(state.data_dir.as_ref()),
        replay_backend_readiness_check(&state.ingress_security.replay_backend),
        chatops_audit_store_readiness_check(&state.chatops_runtime, state.data_dir.as_ref()),
        chatops_runtime_readiness_check(&state.chatops_runtime),
    ];
    let status = if checks.iter().all(|check| check.ready) {
        "ready"
    } else {
        "degraded"
    };

    ServiceReadinessResponse {
        status: status.to_string(),
        runtime_mode: service_runtime_mode_label().to_string(),
        uptime_seconds: state.started_at.elapsed().as_secs(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        checks,
    }
}

fn next_service_request_id() -> String {
    static COUNTER: AtomicU64 = AtomicU64::new(1);
    let sequence = COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("req-{}-{sequence:08x}", current_unix_timestamp_ms())
}

fn service_http_timeout() -> std::time::Duration {
    let millis = std::env::var(NTK_SERVICE_HTTP_TIMEOUT_MS_ENV)
        .ok()
        .and_then(|value| value.trim().parse::<u64>().ok())
        .map(|value| value.clamp(MIN_SERVICE_HTTP_TIMEOUT_MS, MAX_SERVICE_HTTP_TIMEOUT_MS))
        .unwrap_or(DEFAULT_SERVICE_HTTP_TIMEOUT_MS);
    std::time::Duration::from_millis(millis)
}

fn exit_status_label(status: ExitStatus) -> &'static str {
    match status {
        ExitStatus::Success => "success",
        ExitStatus::Error => "error",
        ExitStatus::Interrupted => "interrupted",
    }
}

async fn service_request_context_middleware(mut request: Request, next: Next) -> Response {
    let request_context = service_request_context_from_headers(request.headers());
    let method = request.method().clone();
    let path = request.uri().path().to_string();
    let started_at = std::time::Instant::now();
    request.extensions_mut().insert(request_context.clone());

    let mut response = next.run(request).await;
    if let Ok(header_value) = HeaderValue::from_str(&request_context.request_id) {
        response.headers_mut().insert("x-request-id", header_value);
    }
    if let Some(correlation_id) = request_context.correlation_id.as_deref() {
        if let Ok(header_value) = HeaderValue::from_str(correlation_id) {
            response
                .headers_mut()
                .insert("x-correlation-id", header_value);
        }
    }
    tracing::info!(
        request_id = %request_context.request_id,
        correlation_id = request_context.correlation_id.as_deref().unwrap_or(""),
        method = %method,
        path = %path,
        status = response.status().as_u16(),
        latency_ms = started_at.elapsed().as_millis() as u64,
        "service request handled"
    );

    response
}

async fn handle_service_timeout_error(_: BoxError) -> Response {
    (
        StatusCode::REQUEST_TIMEOUT,
        "service request timed out before completion",
    )
        .into_response()
}

async fn service_bearer_auth_middleware(
    State(state): State<Arc<ServiceRuntimeState>>,
    request: Request,
    next: Next,
) -> Response {
    let Some(expected_token) = state.service_auth_token.as_deref() else {
        return next.run(request).await;
    };

    let headers = normalize_http_headers(request.headers());
    if request_bearer_token(&headers) == Some(expected_token) {
        next.run(request).await
    } else {
        (
            StatusCode::UNAUTHORIZED,
            "missing or invalid bearer token for /task/submit",
        )
            .into_response()
    }
}

async fn service_root() -> impl IntoResponse {
    (
        StatusCode::OK,
        "NetToolsKit service mode is running.\nUse GET /health or GET /ready.",
    )
}

#[cfg(test)]
async fn service_test_slow() -> impl IntoResponse {
    tokio::time::sleep(std::time::Duration::from_millis(250)).await;
    (StatusCode::OK, "slow-response")
}

async fn service_health(State(state): State<Arc<ServiceRuntimeState>>) -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(build_service_health_response(state.as_ref())),
    )
}

async fn service_ready(State(state): State<Arc<ServiceRuntimeState>>) -> impl IntoResponse {
    let payload = build_service_readiness_response(state.as_ref());
    let status = if payload.status == "ready" {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };
    (status, Json(payload))
}

async fn service_task_submit(
    State(_state): State<Arc<ServiceRuntimeState>>,
    Extension(request_context): Extension<ServiceRequestContext>,
    headers: AxumHeaderMap,
    payload: Result<Json<ServiceTaskSubmitRequest>, axum::extract::rejection::JsonRejection>,
) -> Response {
    let payload = match payload {
        Ok(Json(payload)) => payload,
        Err(error) if error.status() == StatusCode::PAYLOAD_TOO_LARGE => {
            return (
                StatusCode::PAYLOAD_TOO_LARGE,
                "request body exceeded the configured limit for /task/submit",
            )
                .into_response();
        }
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                "invalid JSON payload for /task/submit",
            )
                .into_response();
        }
    };

    if payload.intent.trim().is_empty() || payload.payload.trim().is_empty() {
        return (StatusCode::BAD_REQUEST, "intent and payload are required").into_response();
    }

    let normalized_headers = normalize_http_headers(&headers);
    let control_envelope = build_service_control_envelope(
        &request_context,
        &normalized_headers,
        payload.intent.as_str(),
        payload.payload.as_str(),
    );
    tracing::info!(
        request_id = %control_envelope.request_id,
        correlation_id = control_envelope.correlation_id.as_deref().unwrap_or(""),
        operator_id = %control_envelope.operator.id,
        operator_kind = %control_envelope.operator.kind,
        session_id = %control_envelope.session.id,
        intent = %payload.intent.trim(),
        "service task submit admitted"
    );
    let submission =
        nettoolskit_orchestrator::process_control_envelope(control_envelope.clone()).await;
    let response_payload = ServiceTaskSubmitResponse {
        accepted: submission.exit_status != ExitStatus::Error,
        exit_status: exit_status_label(submission.exit_status).to_string(),
        task_id: submission.task_id,
        request_id: control_envelope.request_id,
        correlation_id: control_envelope.correlation_id,
        operator_id: control_envelope.operator.id,
        operator_kind: control_envelope.operator.kind.to_string(),
        session_id: control_envelope.session.id,
        transport: control_envelope.operator.transport.to_string(),
    };
    let status_code = if response_payload.accepted {
        StatusCode::ACCEPTED
    } else {
        StatusCode::BAD_REQUEST
    };

    (status_code, Json(response_payload)).into_response()
}

async fn service_telegram_webhook(
    State(state): State<Arc<ServiceRuntimeState>>,
    headers: AxumHeaderMap,
    body: Bytes,
) -> Response {
    let Some(runtime) = state.chatops_runtime.runtime.as_ref() else {
        return (StatusCode::NOT_FOUND, "chatops runtime is not enabled").into_response();
    };

    if !runtime.is_telegram_webhook_enabled() {
        return (StatusCode::CONFLICT, "telegram webhook mode is disabled").into_response();
    }

    let Ok(body) = std::str::from_utf8(&body) else {
        return (
            StatusCode::BAD_REQUEST,
            "invalid Telegram webhook payload: request body is not valid UTF-8",
        )
            .into_response();
    };
    let headers = normalize_http_headers(&headers);

    match verify_telegram_ingress_security(state.as_ref(), &headers, body) {
        Ok(()) => {}
        Err(IngressSecurityError::Unauthorized) => {
            return (
                StatusCode::UNAUTHORIZED,
                "telegram webhook signature/token validation failed",
            )
                .into_response();
        }
        Err(IngressSecurityError::Replay) => {
            return (StatusCode::CONFLICT, "telegram webhook replay detected").into_response();
        }
        Err(IngressSecurityError::Unavailable) => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                "telegram webhook replay backend unavailable",
            )
                .into_response();
        }
    }

    match runtime.enqueue_telegram_webhook_payload(body) {
        Ok(queued) => (
            StatusCode::ACCEPTED,
            Json(ServiceTelegramWebhookResponse {
                accepted: true,
                queued,
            }),
        )
            .into_response(),
        Err(error) => (
            StatusCode::BAD_REQUEST,
            format!("invalid Telegram webhook payload: {error}"),
        )
            .into_response(),
    }
}

async fn service_discord_interactions(
    State(state): State<Arc<ServiceRuntimeState>>,
    headers: AxumHeaderMap,
    body: Bytes,
) -> Response {
    let Some(runtime) = state.chatops_runtime.runtime.as_ref() else {
        return (StatusCode::NOT_FOUND, "chatops runtime is not enabled").into_response();
    };

    if !runtime.is_discord_interactions_enabled() {
        return (StatusCode::CONFLICT, "discord interaction mode is disabled").into_response();
    }

    let Ok(body) = std::str::from_utf8(&body) else {
        return (
            StatusCode::BAD_REQUEST,
            "invalid Discord interaction payload: request body is not valid UTF-8",
        )
            .into_response();
    };
    let headers = normalize_http_headers(&headers);

    match verify_discord_interaction_ingress_security(state.as_ref(), &headers, body) {
        Ok(()) => {}
        Err(IngressSecurityError::Unauthorized) => {
            return (
                StatusCode::UNAUTHORIZED,
                "discord interaction signature validation failed",
            )
                .into_response();
        }
        Err(IngressSecurityError::Replay) => {
            return (StatusCode::CONFLICT, "discord interaction replay detected").into_response();
        }
        Err(IngressSecurityError::Unavailable) => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                "discord interaction replay backend unavailable",
            )
                .into_response();
        }
    }

    match runtime.enqueue_discord_interaction_payload(body) {
        Ok(outcome) if outcome.ping => (
            StatusCode::OK,
            Json(ServiceDiscordInteractionResponse {
                response_type: 1,
                data: None,
            }),
        )
            .into_response(),
        Ok(outcome) => (
            StatusCode::OK,
            Json(ServiceDiscordInteractionResponse {
                response_type: 4,
                data: Some(ServiceDiscordInteractionData {
                    content: format!(
                        "Command accepted for async execution (queued: {}).",
                        outcome.queued
                    ),
                    flags: 64,
                }),
            }),
        )
            .into_response(),
        Err(error) => (
            StatusCode::BAD_REQUEST,
            format!("invalid Discord interaction payload: {error}"),
        )
            .into_response(),
    }
}

fn service_router(state: Arc<ServiceRuntimeState>) -> Router {
    let router = Router::new()
        .route("/", get(service_root))
        .route("/health", get(service_health))
        .route("/ready", get(service_ready))
        .route(
            "/task/submit",
            post(service_task_submit).route_layer(middleware::from_fn_with_state(
                state.clone(),
                service_bearer_auth_middleware,
            )),
        )
        .route("/chatops/telegram/webhook", post(service_telegram_webhook))
        .route(
            "/chatops/discord/interactions",
            post(service_discord_interactions),
        );
    #[cfg(test)]
    let router = router.route("/__test/slow", get(service_test_slow));

    router
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(handle_service_timeout_error))
                .layer(TimeoutLayer::new(service_http_timeout())),
        )
        .layer(DefaultBodyLimit::max(DEFAULT_SERVICE_HTTP_BODY_LIMIT_BYTES))
        .layer(middleware::from_fn(service_request_context_middleware))
        .with_state(state)
}

async fn run_service_mode(host: String, port: u16) -> ExitStatus {
    let service_auth_token = service_auth_token_from_env();
    if let Err(error) = validate_service_bind_security(&host, service_auth_token.as_deref()) {
        eprintln!("Refusing to start service mode: {error}");
        return ExitStatus::Error;
    }

    let bind_addr = format!("{host}:{port}");
    let listener = match TcpListener::bind(&bind_addr).await {
        Ok(listener) => listener,
        Err(error) => {
            eprintln!("Failed to bind service listener on {bind_addr}: {error}");
            return ExitStatus::Error;
        }
    };

    println!("NetToolsKit service mode listening on http://{bind_addr}");
    println!("Health endpoint: GET /health");
    println!("Readiness endpoint: GET /ready");
    println!("Task submit endpoint: POST /task/submit");
    if service_auth_token.is_some() {
        println!("Task submit auth: bearer token enabled");
    } else {
        println!("Task submit auth: disabled (loopback-only bind)");
    }

    let ingress_security = match ServiceIngressSecurityConfig::from_env() {
        Ok(config) => Arc::new(config),
        Err(error) => {
            eprintln!("Failed to load ChatOps ingress security config: {error}");
            return ExitStatus::Error;
        }
    };
    if ingress_security.has_telegram_signature_validation() {
        println!("Telegram webhook signature/token validation: enabled");
    }
    if ingress_security.has_discord_signature_validation() {
        println!("Discord interaction signature validation: enabled");
    }
    println!(
        "Ingress replay backend: {}",
        ingress_security.replay_backend_description()
    );

    let chatops_runtime = initialize_chatops_runtime();
    if let Some(runtime) = &chatops_runtime.runtime {
        if runtime.is_telegram_webhook_enabled() {
            println!("Telegram webhook endpoint: POST /chatops/telegram/webhook");
        }
        if runtime.is_discord_interactions_enabled() {
            println!("Discord interactions endpoint: POST /chatops/discord/interactions");
        }
    } else if let Some(error) = &chatops_runtime.startup_error {
        eprintln!("Warning: ChatOps runtime is enabled but could not start: {error}");
    }
    let service_state = Arc::new(ServiceRuntimeState {
        started_at: std::time::Instant::now(),
        chatops_runtime,
        replay_guard: Arc::new(IngressReplayGuard::with_backend(
            ingress_security.replay_window,
            ingress_security.replay_max_entries,
            ingress_security.replay_backend.clone(),
        )),
        service_auth_token,
        data_dir: AppConfig::default_data_dir(),
        ingress_security,
    });

    if let Err(error) = axum::serve(listener, service_router(service_state)).await {
        tracing::warn!(%error, "service listener terminated unexpectedly");
        return ExitStatus::Error;
    }

    ExitStatus::Success
}

fn initialize_chatops_runtime() -> InitializedChatOpsRuntime {
    let config = nettoolskit_orchestrator::ChatOpsRuntimeConfig::from_env();

    match nettoolskit_orchestrator::build_chatops_runtime(config.clone()) {
        Ok(Some(runtime)) => {
            let runtime = Arc::new(runtime);
            let poll_interval = runtime.poll_interval();
            let enabled_platforms = runtime
                .enabled_platforms()
                .into_iter()
                .map(|platform| platform.to_string())
                .collect::<Vec<_>>()
                .join(", ");

            println!(
                "ChatOps runtime enabled for platforms: {enabled_platforms} (poll {}ms)",
                poll_interval.as_millis()
            );
            spawn_chatops_runtime_loop(runtime.clone());
            InitializedChatOpsRuntime {
                runtime: Some(runtime),
                config,
                startup_error: None,
            }
        }
        Ok(None) => InitializedChatOpsRuntime {
            runtime: None,
            config,
            startup_error: None,
        },
        Err(error) => InitializedChatOpsRuntime {
            runtime: None,
            config,
            startup_error: Some(error.to_string()),
        },
    }
}

fn spawn_chatops_runtime_loop(runtime: Arc<nettoolskit_orchestrator::ChatOpsRuntime>) {
    let poll_interval = runtime.poll_interval();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(poll_interval);
        // Skip immediate tick to keep startup logs deterministic.
        interval.tick().await;

        loop {
            interval.tick().await;
            let summary = runtime.tick().await;
            if summary.envelopes_received > 0
                || summary.ingress_errors > 0
                || summary.notification_errors > 0
            {
                tracing::info!(
                    envelopes_received = summary.envelopes_received,
                    executed_success = summary.executed_success,
                    executed_failed = summary.executed_failed,
                    rate_limited = summary.rate_limited,
                    ingress_errors = summary.ingress_errors,
                    notification_errors = summary.notification_errors,
                    "chatops runtime tick summary"
                );
            }
        }
    });
}

/// NetToolsKit CLI
///
/// A toolkit for .NET development with templates, manifests, and automation tools.
/// If no subcommand is specified, the interactive CLI will be launched.
#[derive(Debug, Parser)]
#[clap(
    author = "NetToolsKit Team",
    version,
    bin_name = "ntk",
    override_usage = "ntk [OPTIONS] [PROMPT]\n       ntk [OPTIONS] <COMMAND> [ARGS]",
    disable_help_subcommand = true
)]
struct Cli {
    #[clap(flatten)]
    pub global: GlobalArgs,

    #[clap(subcommand)]
    pub subcommand: Option<Commands>,
}

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() {
    // Parse command line arguments
    let cli = Cli::parse();

    // Load configuration (file → env → defaults), then apply CLI overrides
    let config = match &cli.global.config {
        Some(path) => {
            let p = std::path::Path::new(path);
            match AppConfig::load_from(p) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Warning: Failed to load config from {}: {}", path, e);
                    AppConfig::load()
                }
            }
        }
        None => AppConfig::load(),
    };

    let configured_log_level = cli
        .global
        .log_level
        .clone()
        .unwrap_or_else(|| config.general.log_level.clone());
    let requested_verbose_level = matches!(
        configured_log_level.to_ascii_lowercase().as_str(),
        "debug" | "trace"
    );
    let verbose = cli.global.verbose || config.general.verbose || requested_verbose_level;

    // Wire user config into the terminal capabilities system
    match config.display.color {
        ColorMode::Always => set_color_override(Some(ColorLevel::TrueColor)),
        ColorMode::Never => set_color_override(Some(ColorLevel::None)),
        ColorMode::Auto => {} // capabilities auto-detect
    }
    match config.display.unicode {
        UnicodeMode::Always => set_unicode_override(Some(true)),
        UnicodeMode::Never => set_unicode_override(Some(false)),
        UnicodeMode::Auto => {} // capabilities auto-detect
    }

    let run_interactive = cli.subcommand.is_none();

    if !run_interactive {
        let tracing_config = TracingConfig {
            verbose,
            log_level: configured_log_level.clone(),
            ..Default::default()
        };
        if let Err(e) = init_tracing_with_config(tracing_config) {
            eprintln!("Warning: Failed to initialize tracing: {}", e);
        }
    }

    let exit_status: ExitStatus = match cli.subcommand {
        Some(command) => {
            let execution_correlation_id = next_correlation_id("exec");
            let execution_span = info_span!(
                "cli.non_interactive_execution",
                correlation_id = %execution_correlation_id
            );
            let _execution_scope = execution_span.enter();
            info!(
                correlation_id = %execution_correlation_id,
                "Starting non-interactive CLI execution"
            );

            let status = command.execute().await;
            info!(
                correlation_id = %execution_correlation_id,
                final_status = ?status,
                "Non-interactive CLI execution finished"
            );
            status
        }
        None => {
            let options = InteractiveOptions {
                verbose,
                log_level: configured_log_level,
                footer_output: config.general.footer_output,
                attention_bell: config.general.attention_bell,
                attention_desktop_notification: config.general.attention_desktop_notification,
                attention_unfocused_only: config.general.attention_unfocused_only,
                predictive_input: config.general.predictive_input,
                ai_session_retention: config.general.ai_session_retention,
            };
            interactive_mode(options).await
        }
    };

    shutdown_tracing();

    // Exit with appropriate code
    std::process::exit(exit_status.into());
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::to_bytes;
    use ed25519_dalek::{Signer, SigningKey};
    use nettoolskit_orchestrator::{build_chatops_runtime, ChatOpsRuntime, ChatOpsRuntimeConfig};
    use serde::de::DeserializeOwned;
    use serial_test::serial;
    use std::sync::{Mutex, OnceLock};

    #[test]
    fn parse_http_request_line_extracts_method_and_path() {
        let raw = "GET /health HTTP/1.1\r\nHost: localhost\r\n\r\n";
        let parsed = parse_http_request_line(raw);
        assert_eq!(parsed, Some(("GET", "/health")));
    }

    #[test]
    fn request_body_extracts_payload_segment() {
        let raw = "POST /task/submit HTTP/1.1\r\nHost: localhost\r\n\r\n{\"intent\":\"ai-plan\"}";
        assert_eq!(request_body(raw), "{\"intent\":\"ai-plan\"}");
    }

    #[test]
    fn build_http_response_contains_status_and_length() {
        let response = build_http_response("200 OK", "text/plain", "ok");
        assert!(response.starts_with("HTTP/1.1 200 OK"));
        assert!(response.contains("Content-Length: 2"));
        assert!(response.ends_with("\r\n\r\nok"));
    }

    fn default_test_ingress_security() -> Arc<ServiceIngressSecurityConfig> {
        Arc::new(ServiceIngressSecurityConfig {
            telegram_secret_token: None,
            discord_verifying_key: None,
            replay_window: std::time::Duration::from_secs(300),
            replay_max_entries: 1_024,
            replay_backend: IngressReplayBackendConfig::Memory,
        })
    }

    fn disabled_chatops_runtime() -> InitializedChatOpsRuntime {
        InitializedChatOpsRuntime {
            runtime: None,
            config: ChatOpsRuntimeConfig::default(),
            startup_error: None,
        }
    }

    fn initialized_chatops_runtime(
        runtime: Option<Arc<ChatOpsRuntime>>,
        config: ChatOpsRuntimeConfig,
        startup_error: Option<&str>,
    ) -> InitializedChatOpsRuntime {
        InitializedChatOpsRuntime {
            runtime,
            config,
            startup_error: startup_error.map(ToOwned::to_owned),
        }
    }

    fn unique_test_path(label: &str) -> std::path::PathBuf {
        let suffix = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time should be after UNIX_EPOCH")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "nettoolskit-{label}-{}-{suffix}",
            std::process::id()
        ))
    }

    fn test_service_state_with_security(
        chatops_runtime: InitializedChatOpsRuntime,
        ingress_security: Arc<ServiceIngressSecurityConfig>,
        service_auth_token: Option<&str>,
        data_dir: Option<std::path::PathBuf>,
    ) -> Arc<ServiceRuntimeState> {
        Arc::new(ServiceRuntimeState {
            started_at: std::time::Instant::now(),
            chatops_runtime,
            replay_guard: Arc::new(IngressReplayGuard::with_backend(
                ingress_security.replay_window,
                ingress_security.replay_max_entries,
                ingress_security.replay_backend.clone(),
            )),
            service_auth_token: service_auth_token.map(ToOwned::to_owned),
            data_dir,
            ingress_security,
        })
    }

    fn test_service_state(
        chatops_runtime: Option<Arc<ChatOpsRuntime>>,
    ) -> Arc<ServiceRuntimeState> {
        test_service_state_with_security(
            initialized_chatops_runtime(chatops_runtime, ChatOpsRuntimeConfig::default(), None),
            default_test_ingress_security(),
            None,
            Some(unique_test_path("service-data")),
        )
    }

    fn test_telegram_webhook_config(webhook_enabled: bool) -> ChatOpsRuntimeConfig {
        ChatOpsRuntimeConfig {
            enabled: true,
            allowed_user_ids: vec!["777".to_string()],
            allowed_channel_ids: vec!["555".to_string()],
            allowed_command_scopes: vec!["list".to_string(), "help".to_string()],
            telegram_bot_token: Some("test-token".to_string()),
            telegram_api_base: "http://127.0.0.1".to_string(),
            telegram_webhook_enabled: webhook_enabled,
            ..ChatOpsRuntimeConfig::default()
        }
    }

    fn test_telegram_webhook_runtime(
        webhook_enabled: bool,
    ) -> (Arc<ChatOpsRuntime>, ChatOpsRuntimeConfig) {
        let config = test_telegram_webhook_config(webhook_enabled);
        let runtime = build_chatops_runtime(config.clone())
            .expect("runtime should build")
            .expect("enabled runtime should be present");
        (Arc::new(runtime), config)
    }

    fn test_discord_interactions_config(interactions_enabled: bool) -> ChatOpsRuntimeConfig {
        ChatOpsRuntimeConfig {
            enabled: true,
            allowed_user_ids: vec!["777".to_string()],
            allowed_channel_ids: vec!["555".to_string()],
            allowed_command_scopes: vec!["list".to_string(), "submit".to_string()],
            discord_bot_token: Some("test-token".to_string()),
            discord_api_base: "http://127.0.0.1".to_string(),
            discord_interactions_enabled: interactions_enabled,
            discord_channel_ids: if interactions_enabled {
                Vec::new()
            } else {
                vec!["555".to_string()]
            },
            ..ChatOpsRuntimeConfig::default()
        }
    }

    fn test_discord_interactions_runtime(
        interactions_enabled: bool,
    ) -> (Arc<ChatOpsRuntime>, ChatOpsRuntimeConfig) {
        let config = test_discord_interactions_config(interactions_enabled);
        let runtime = build_chatops_runtime(config.clone())
            .expect("runtime should build")
            .expect("enabled runtime should be present");
        (Arc::new(runtime), config)
    }

    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

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

    async fn execute_service_request_with_state(
        request: &str,
        state: Arc<ServiceRuntimeState>,
    ) -> String {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("test listener should bind");
        let address = listener
            .local_addr()
            .expect("test listener should provide local address");

        let server = tokio::spawn(async move {
            axum::serve(listener, service_router(state))
                .await
                .expect("axum service router should serve test requests");
        });
        let (method, path) =
            parse_http_request_line(request).expect("request line should parse in tests");
        let headers = parse_http_headers(request);
        let body = request_body(request).to_string();
        let client = reqwest::Client::new();
        let url = format!("http://{address}{path}");
        let method =
            reqwest::Method::from_bytes(method.as_bytes()).expect("method should be valid");
        let mut request_builder = client.request(method, url);
        for (name, value) in headers {
            request_builder = request_builder.header(&name, value);
        }
        if !body.is_empty() {
            request_builder = request_builder.body(body);
        }
        let response = request_builder
            .send()
            .await
            .expect("request should succeed against test service");
        let status_line = format!(
            "{} {}",
            response.status().as_u16(),
            response.status().canonical_reason().unwrap_or("Unknown")
        );
        let body = response
            .text()
            .await
            .expect("response body should be readable");
        server.abort();
        let _ = server.await;
        build_http_response(&status_line, "text/plain; charset=utf-8", &body)
    }

    async fn execute_service_request(request: &str) -> String {
        execute_service_request_with_state(request, test_service_state(None)).await
    }

    async fn execute_service_request_direct(
        request: Request,
        state: Arc<ServiceRuntimeState>,
    ) -> Response {
        service_router(state)
            .oneshot(request)
            .await
            .expect("router should accept direct test request")
    }

    async fn parse_response_json<T: DeserializeOwned>(response: Response) -> T {
        let bytes = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("response body should be readable");
        serde_json::from_slice(&bytes).expect("response body should deserialize")
    }

    #[test]
    fn parse_http_headers_normalizes_names_and_request_header_trims_values() {
        let headers = parse_http_headers(
            "GET /health HTTP/1.1\r\nHost: localhost\r\nAuthorization:   Bearer token-123   \r\n\r\n",
        );
        assert_eq!(request_header(&headers, "host"), Some("localhost"));
        assert_eq!(
            request_header(&headers, "authorization"),
            Some("Bearer token-123")
        );
    }

    #[test]
    fn request_bearer_token_extracts_bearer_token() {
        let headers = HashMap::from([(
            "authorization".to_string(),
            "Bearer service-token".to_string(),
        )]);
        assert_eq!(request_bearer_token(&headers), Some("service-token"));
    }

    #[test]
    fn request_bearer_token_rejects_non_bearer_scheme() {
        let headers = HashMap::from([(
            "authorization".to_string(),
            "Basic service-token".to_string(),
        )]);
        assert_eq!(request_bearer_token(&headers), None);
    }

    #[test]
    fn service_runtime_mode_label_returns_service() {
        assert_eq!(service_runtime_mode_label(), "service");
    }

    #[test]
    fn build_service_health_response_uses_service_runtime_mode() {
        let state = test_service_state(None);
        let response = build_service_health_response(state.as_ref());
        assert_eq!(response.runtime_mode, "service");
        assert_eq!(response.status, "ok");
    }

    #[test]
    fn is_loopback_bind_host_accepts_supported_values() {
        assert!(is_loopback_bind_host("127.0.0.1"));
        assert!(is_loopback_bind_host("localhost"));
        assert!(is_loopback_bind_host("::1"));
        assert!(!is_loopback_bind_host("0.0.0.0"));
    }

    #[test]
    fn validate_service_bind_security_rejects_remote_bind_without_token() {
        let error = validate_service_bind_security("0.0.0.0", None)
            .expect_err("non-loopback bind should require token");
        assert!(error.contains(NTK_SERVICE_AUTH_TOKEN_ENV));
    }

    #[test]
    fn validate_service_bind_security_allows_remote_bind_with_token() {
        validate_service_bind_security("0.0.0.0", Some("service-token"))
            .expect("remote bind should be allowed when token is configured");
    }

    #[test]
    fn service_http_timeout_uses_default_when_env_is_missing() {
        let _lock = env_lock().lock().expect("env lock should not be poisoned");
        let _guard = EnvVarGuard::set(&[(NTK_SERVICE_HTTP_TIMEOUT_MS_ENV, None)]);
        assert_eq!(
            service_http_timeout(),
            std::time::Duration::from_millis(DEFAULT_SERVICE_HTTP_TIMEOUT_MS)
        );
    }

    #[test]
    fn service_http_timeout_clamps_env_value_to_supported_bounds() {
        let _lock = env_lock().lock().expect("env lock should not be poisoned");
        let _guard = EnvVarGuard::set(&[(NTK_SERVICE_HTTP_TIMEOUT_MS_ENV, Some("5"))]);
        assert_eq!(
            service_http_timeout(),
            std::time::Duration::from_millis(MIN_SERVICE_HTTP_TIMEOUT_MS)
        );
    }

    #[test]
    fn replay_lock_path_appends_lock_extension() {
        let path = std::path::Path::new("cache/ingress-replay-cache.json");
        let lock_path = replay_lock_path(path);
        assert!(lock_path.ends_with("ingress-replay-cache.json.lock"));
    }

    #[test]
    fn ensure_directory_ready_creates_missing_directory() {
        let path = unique_test_path("directory-ready");
        ensure_directory_ready(&path).expect("directory should be created");
        assert!(path.is_dir());
    }

    #[test]
    fn ensure_appendable_file_ready_creates_parent_directory_and_file() {
        let path = unique_test_path("appendable-file")
            .join("nested")
            .join("audit.jsonl");
        ensure_appendable_file_ready(&path).expect("appendable file should be created");
        assert!(path.is_file());
    }

    #[test]
    fn task_admission_readiness_check_reports_ready() {
        let check = task_admission_readiness_check();
        assert!(check.ready);
        assert_eq!(check.name, "task_admission");
    }

    #[test]
    fn local_persistence_readiness_check_reports_missing_data_directory() {
        let check = local_persistence_readiness_check(None);
        assert!(!check.ready);
        assert_eq!(check.name, "local_persistence");
    }

    #[test]
    fn chatops_audit_store_readiness_check_reports_disabled_runtime_as_ready() {
        let check = chatops_audit_store_readiness_check(&disabled_chatops_runtime(), None);
        assert!(check.ready);
        assert_eq!(check.name, "chatops_audit_store");
    }

    #[test]
    fn chatops_runtime_readiness_check_reports_disabled_runtime_as_ready() {
        let check = chatops_runtime_readiness_check(&disabled_chatops_runtime());
        assert!(check.ready);
        assert_eq!(check.name, "chatops_runtime");
    }

    #[test]
    fn readiness_check_builds_expected_payload() {
        let state = test_service_state_with_security(
            disabled_chatops_runtime(),
            default_test_ingress_security(),
            None,
            Some(unique_test_path("readiness-payload")),
        );
        let response = build_service_readiness_response(state.as_ref());
        assert_eq!(response.status, "ready");
        assert_eq!(response.runtime_mode, "service");
        assert_eq!(response.checks.len(), 5);
    }

    #[tokio::test]
    async fn service_request_context_middleware_generates_request_id_when_missing() {
        let response = execute_service_request_direct(
            Request::builder()
                .method("GET")
                .uri("/health")
                .body(Body::empty())
                .expect("request should build"),
            test_service_state(None),
        )
        .await;
        let request_id = response
            .headers()
            .get("x-request-id")
            .and_then(|value| value.to_str().ok())
            .expect("request id header should be present");
        assert!(request_id.starts_with("req-"));
    }

    #[tokio::test]
    async fn service_request_context_middleware_preserves_supplied_request_id() {
        let response = execute_service_request_direct(
            Request::builder()
                .method("GET")
                .uri("/health")
                .header("x-request-id", "test-request-123")
                .body(Body::empty())
                .expect("request should build"),
            test_service_state(None),
        )
        .await;
        assert_eq!(
            response
                .headers()
                .get("x-request-id")
                .and_then(|value| value.to_str().ok()),
            Some("test-request-123")
        );
    }

    #[tokio::test]
    async fn service_request_context_middleware_preserves_supplied_correlation_id() {
        let response = execute_service_request_direct(
            Request::builder()
                .method("GET")
                .uri("/health")
                .header("x-correlation-id", "corr-123")
                .body(Body::empty())
                .expect("request should build"),
            test_service_state(None),
        )
        .await;
        assert_eq!(
            response
                .headers()
                .get("x-correlation-id")
                .and_then(|value| value.to_str().ok()),
            Some("corr-123")
        );
    }

    #[tokio::test]
    async fn service_router_enforces_default_body_limit() {
        let oversized_payload = format!(
            "{{\"intent\":\"ai-plan\",\"payload\":\"{}\"}}",
            "x".repeat(DEFAULT_SERVICE_HTTP_BODY_LIMIT_BYTES + 1)
        );
        let response = execute_service_request_direct(
            Request::builder()
                .method("POST")
                .uri("/task/submit")
                .header("content-type", "application/json")
                .body(Body::from(oversized_payload))
                .expect("request should build"),
            test_service_state(None),
        )
        .await;
        assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);
    }

    #[tokio::test]
    async fn service_router_enforces_timeout_budget() {
        let router = {
            let _lock = env_lock().lock().expect("env lock should not be poisoned");
            let _guard = EnvVarGuard::set(&[(NTK_SERVICE_HTTP_TIMEOUT_MS_ENV, Some("100"))]);
            service_router(test_service_state(None))
        };
        let response = router
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/__test/slow")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("router should accept timeout test request");
        assert_eq!(response.status(), StatusCode::REQUEST_TIMEOUT);
    }

    #[test]
    #[serial]
    fn resolve_replay_backend_from_env_supports_file_backend_override() {
        let _guard = env_lock().lock().expect("env lock should not be poisoned");
        let expected_path = unique_test_path("replay-backend").join("cache.json");
        let expected_string = expected_path.to_string_lossy().to_string();
        let _env = EnvVarGuard::set(&[
            (NTK_CHATOPS_INGRESS_REPLAY_BACKEND_ENV, Some("file")),
            (
                NTK_CHATOPS_INGRESS_REPLAY_FILE_PATH_ENV,
                Some(expected_string.as_str()),
            ),
        ]);

        let backend = resolve_replay_backend_from_env().expect("file backend should resolve");
        match backend {
            IngressReplayBackendConfig::File { path } => assert_eq!(path, expected_path),
            IngressReplayBackendConfig::Memory => {
                panic!("expected file backend when override is configured")
            }
        }
    }

    #[test]
    #[serial]
    fn resolve_replay_backend_from_env_rejects_unknown_backend() {
        let _guard = env_lock().lock().expect("env lock should not be poisoned");
        let _env = EnvVarGuard::set(&[
            (
                NTK_CHATOPS_INGRESS_REPLAY_BACKEND_ENV,
                Some("invalid-backend"),
            ),
            (NTK_CHATOPS_INGRESS_REPLAY_FILE_PATH_ENV, None),
        ]);

        let error =
            resolve_replay_backend_from_env().expect_err("unknown backend should be rejected");
        assert!(error.contains("unsupported replay backend"));
    }

    #[tokio::test]
    async fn service_mode_health_endpoint_returns_ok_json() {
        let response =
            execute_service_request("GET /health HTTP/1.1\r\nHost: localhost\r\n\r\n").await;
        assert!(
            response.starts_with("HTTP/1.1 200 OK"),
            "health endpoint should return 200"
        );
        assert!(
            response.contains("\"status\":\"ok\""),
            "health response should expose status=ok"
        );
        assert!(
            response.contains("\"runtime_mode\":\"service\""),
            "health response should force runtime_mode=service"
        );
    }

    #[tokio::test]
    async fn service_mode_health_endpoint_remains_live_when_readiness_is_degraded() {
        let degraded_state = test_service_state_with_security(
            disabled_chatops_runtime(),
            default_test_ingress_security(),
            None,
            None,
        );

        let health = execute_service_request_with_state(
            "GET /health HTTP/1.1\r\nHost: localhost\r\n\r\n",
            degraded_state.clone(),
        )
        .await;
        assert!(health.starts_with("HTTP/1.1 200 OK"));

        let ready = execute_service_request_with_state(
            "GET /ready HTTP/1.1\r\nHost: localhost\r\n\r\n",
            degraded_state,
        )
        .await;
        assert!(ready.starts_with("HTTP/1.1 503 Service Unavailable"));
    }

    #[tokio::test]
    async fn service_mode_readiness_endpoint_returns_ready_when_dependencies_are_available() {
        let state = test_service_state_with_security(
            disabled_chatops_runtime(),
            default_test_ingress_security(),
            None,
            Some(unique_test_path("service-ready")),
        );
        let response = execute_service_request_with_state(
            "GET /ready HTTP/1.1\r\nHost: localhost\r\n\r\n",
            state,
        )
        .await;
        assert!(response.starts_with("HTTP/1.1 200 OK"));
        assert!(response.contains("\"status\":\"ready\""));
        assert!(response.contains("\"task_admission\""));
    }

    #[tokio::test]
    async fn service_mode_readiness_returns_unavailable_for_chatops_bootstrap_failure() {
        let config = ChatOpsRuntimeConfig {
            enabled: true,
            audit_path: Some(unique_test_path("chatops-audit").join("audit.jsonl")),
            ..ChatOpsRuntimeConfig::default()
        };
        let state = test_service_state_with_security(
            initialized_chatops_runtime(None, config, Some("chatops bootstrap failed")),
            default_test_ingress_security(),
            None,
            Some(unique_test_path("service-ready-chatops")),
        );
        let response = execute_service_request_with_state(
            "GET /ready HTTP/1.1\r\nHost: localhost\r\n\r\n",
            state,
        )
        .await;
        assert!(response.starts_with("HTTP/1.1 503 Service Unavailable"));
        assert!(response.contains("chatops bootstrap failed"));
    }

    #[tokio::test]
    async fn service_mode_readiness_returns_unavailable_for_broken_replay_backend() {
        let temp_dir = tempfile::tempdir().expect("temp dir should be created");
        let security = Arc::new(ServiceIngressSecurityConfig {
            telegram_secret_token: None,
            discord_verifying_key: None,
            replay_window: std::time::Duration::from_secs(300),
            replay_max_entries: 1_024,
            replay_backend: IngressReplayBackendConfig::File {
                path: temp_dir.path().to_path_buf(),
            },
        });
        let state = test_service_state_with_security(
            disabled_chatops_runtime(),
            security,
            None,
            Some(unique_test_path("service-ready-broken-replay")),
        );
        let response = execute_service_request_with_state(
            "GET /ready HTTP/1.1\r\nHost: localhost\r\n\r\n",
            state,
        )
        .await;
        assert!(response.starts_with("HTTP/1.1 503 Service Unavailable"));
        assert!(response.contains("\"replay_backend\""));
    }

    #[tokio::test]
    async fn service_mode_task_submit_rejects_invalid_json() {
        let response = execute_service_request(
            "POST /task/submit HTTP/1.1\r\nHost: localhost\r\nContent-Type: application/json\r\n\r\n{invalid",
        )
        .await;
        assert!(
            response.starts_with("HTTP/1.1 400 Bad Request"),
            "invalid payload should be rejected"
        );
        assert!(
            response.contains("invalid JSON payload"),
            "response should explain invalid JSON reason"
        );
    }

    #[tokio::test]
    #[serial]
    async fn service_mode_task_submit_accepts_valid_payload() {
        let _guard = EnvVarGuard::set(&[("NTK_TOOL_SCOPE_ALLOWED_TOOLS", Some("ai.plan"))]);
        let request = concat!(
            "POST /task/submit HTTP/1.1\r\n",
            "Host: localhost\r\n",
            "Content-Type: application/json\r\n",
            "\r\n",
            "{\"intent\":\"ai-plan\",\"payload\":\"validate dual runtime gate\"}"
        );
        let response = execute_service_request(request).await;
        assert!(
            response.starts_with("HTTP/1.1 202 Accepted"),
            "valid submit should be accepted"
        );
        assert!(
            response.contains("\"accepted\":true"),
            "accepted response should be true"
        );
    }

    #[tokio::test]
    async fn service_mode_task_submit_rejects_missing_bearer_token_when_required() {
        let request = concat!(
            "POST /task/submit HTTP/1.1\r\n",
            "Host: localhost\r\n",
            "Content-Type: application/json\r\n",
            "\r\n",
            "{\"intent\":\"ai-plan\",\"payload\":\"validate dual runtime gate\"}"
        );
        let response = execute_service_request_with_state(
            request,
            test_service_state_with_security(
                disabled_chatops_runtime(),
                default_test_ingress_security(),
                Some("expected-token"),
                Some(unique_test_path("service-auth")),
            ),
        )
        .await;
        assert!(response.starts_with("HTTP/1.1 401 Unauthorized"));
    }

    #[tokio::test]
    #[serial]
    async fn service_mode_task_submit_accepts_valid_bearer_token_when_required() {
        let _guard = EnvVarGuard::set(&[("NTK_TOOL_SCOPE_ALLOWED_TOOLS", Some("ai.plan"))]);
        let request = concat!(
            "POST /task/submit HTTP/1.1\r\n",
            "Host: localhost\r\n",
            "Content-Type: application/json\r\n",
            "Authorization: Bearer expected-token\r\n",
            "\r\n",
            "{\"intent\":\"ai-plan\",\"payload\":\"validate dual runtime gate\"}"
        );
        let response = execute_service_request_with_state(
            request,
            test_service_state_with_security(
                disabled_chatops_runtime(),
                default_test_ingress_security(),
                Some("expected-token"),
                Some(unique_test_path("service-auth-valid")),
            ),
        )
        .await;
        assert!(response.starts_with("HTTP/1.1 202 Accepted"));
    }

    #[tokio::test]
    #[serial]
    async fn service_mode_task_submit_returns_control_plane_metadata() {
        let _guard = EnvVarGuard::set(&[("NTK_TOOL_SCOPE_ALLOWED_TOOLS", Some("ai.plan"))]);
        let response = execute_service_request_direct(
            Request::builder()
                .method("POST")
                .uri("/task/submit")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"intent":"ai-plan","payload":"validate service envelope"}"#,
                ))
                .expect("request should build"),
            test_service_state(None),
        )
        .await;
        assert_eq!(response.status(), StatusCode::ACCEPTED);
        let payload: ServiceTaskSubmitResponse = parse_response_json(response).await;
        assert!(payload.accepted);
        assert!(payload
            .task_id
            .as_deref()
            .is_some_and(|task_id| task_id.starts_with("task-")));
        assert!(payload.request_id.starts_with("req-"));
        assert_eq!(payload.correlation_id, None);
        assert_eq!(payload.operator_id, "local-service-operator");
        assert_eq!(payload.operator_kind, "local_human");
        assert_eq!(
            payload.session_id,
            format!("service-request-{}", payload.request_id)
        );
        assert_eq!(payload.transport, "service_http");
    }

    #[tokio::test]
    #[serial]
    async fn service_mode_task_submit_honors_supplied_operator_session_and_correlation_headers() {
        let _guard = EnvVarGuard::set(&[("NTK_TOOL_SCOPE_ALLOWED_TOOLS", Some("ai.plan"))]);
        let response = execute_service_request_direct(
            Request::builder()
                .method("POST")
                .uri("/task/submit")
                .header("content-type", "application/json")
                .header("authorization", "Bearer expected-token")
                .header("x-request-id", "req-user-123")
                .header("x-correlation-id", "corr-user-456")
                .header("x-ntk-operator-id", "remote-dev")
                .header("x-ntk-session-id", "session-user-789")
                .body(Body::from(
                    r#"{"intent":"ai-plan","payload":"review enterprise gaps"}"#,
                ))
                .expect("request should build"),
            test_service_state_with_security(
                disabled_chatops_runtime(),
                default_test_ingress_security(),
                Some("expected-token"),
                Some(unique_test_path("service-envelope-auth")),
            ),
        )
        .await;
        assert_eq!(response.status(), StatusCode::ACCEPTED);
        let payload: ServiceTaskSubmitResponse = parse_response_json(response).await;
        assert!(payload
            .task_id
            .as_deref()
            .is_some_and(|task_id| task_id.starts_with("task-")));
        assert_eq!(payload.request_id, "req-user-123");
        assert_eq!(payload.correlation_id.as_deref(), Some("corr-user-456"));
        assert_eq!(payload.operator_id, "remote-dev");
        assert_eq!(payload.operator_kind, "remote_human");
        assert_eq!(payload.session_id, "session-user-789");
        assert_eq!(payload.transport, "service_http");
    }

    #[tokio::test]
    async fn service_mode_telegram_webhook_accepts_valid_payload_when_enabled() {
        let (runtime, config) = test_telegram_webhook_runtime(true);
        let request = concat!(
            "POST /chatops/telegram/webhook HTTP/1.1\r\n",
            "Host: localhost\r\n",
            "Content-Type: application/json\r\n",
            "\r\n",
            "{\"update_id\":10,\"message\":{\"date\":1737200000,\"text\":\"list\",\"chat\":{\"id\":555},\"from\":{\"id\":777}}}"
        );
        let response = execute_service_request_with_state(
            request,
            test_service_state_with_security(
                initialized_chatops_runtime(Some(runtime), config, None),
                default_test_ingress_security(),
                None,
                Some(unique_test_path("telegram-enabled")),
            ),
        )
        .await;
        assert!(
            response.starts_with("HTTP/1.1 202 Accepted"),
            "valid webhook payload should be accepted"
        );
        assert!(
            response.contains("\"queued\":1"),
            "webhook response should report one queued envelope"
        );
    }

    #[tokio::test]
    async fn service_mode_telegram_webhook_rejects_invalid_payload() {
        let (runtime, config) = test_telegram_webhook_runtime(true);
        let request = concat!(
            "POST /chatops/telegram/webhook HTTP/1.1\r\n",
            "Host: localhost\r\n",
            "Content-Type: application/json\r\n",
            "\r\n",
            "{invalid"
        );
        let response = execute_service_request_with_state(
            request,
            test_service_state_with_security(
                initialized_chatops_runtime(Some(runtime), config, None),
                default_test_ingress_security(),
                None,
                Some(unique_test_path("telegram-invalid")),
            ),
        )
        .await;
        assert!(
            response.starts_with("HTTP/1.1 400 Bad Request"),
            "invalid webhook payload should be rejected"
        );
        assert!(
            response.contains("invalid Telegram webhook payload"),
            "response should explain invalid webhook payload"
        );
    }

    #[tokio::test]
    async fn service_mode_telegram_webhook_returns_conflict_when_disabled() {
        let (runtime, config) = test_telegram_webhook_runtime(false);
        let request = concat!(
            "POST /chatops/telegram/webhook HTTP/1.1\r\n",
            "Host: localhost\r\n",
            "Content-Type: application/json\r\n",
            "\r\n",
            "{\"update_id\":10,\"message\":{\"date\":1737200000,\"text\":\"list\",\"chat\":{\"id\":555},\"from\":{\"id\":777}}}"
        );
        let response = execute_service_request_with_state(
            request,
            test_service_state_with_security(
                initialized_chatops_runtime(Some(runtime), config, None),
                default_test_ingress_security(),
                None,
                Some(unique_test_path("telegram-disabled")),
            ),
        )
        .await;
        assert!(
            response.starts_with("HTTP/1.1 409 Conflict"),
            "webhook endpoint should reject requests when webhook mode is disabled"
        );
    }

    #[tokio::test]
    async fn service_mode_telegram_webhook_rejects_invalid_secret_token() {
        let (runtime, config) = test_telegram_webhook_runtime(true);
        let security = Arc::new(ServiceIngressSecurityConfig {
            telegram_secret_token: Some("expected-token".to_string()),
            discord_verifying_key: None,
            replay_window: std::time::Duration::from_secs(300),
            replay_max_entries: 1_024,
            replay_backend: IngressReplayBackendConfig::Memory,
        });
        let request = concat!(
            "POST /chatops/telegram/webhook HTTP/1.1\r\n",
            "Host: localhost\r\n",
            "Content-Type: application/json\r\n",
            "X-Telegram-Bot-Api-Secret-Token: wrong-token\r\n",
            "\r\n",
            "{\"update_id\":10,\"message\":{\"date\":1737200000,\"text\":\"list\",\"chat\":{\"id\":555},\"from\":{\"id\":777}}}"
        );
        let response = execute_service_request_with_state(
            request,
            test_service_state_with_security(
                initialized_chatops_runtime(Some(runtime), config, None),
                security,
                None,
                Some(unique_test_path("telegram-secret")),
            ),
        )
        .await;
        assert!(
            response.starts_with("HTTP/1.1 401 Unauthorized"),
            "telegram webhook must reject mismatched secret token"
        );
    }

    #[tokio::test]
    async fn service_mode_telegram_webhook_rejects_replay_for_same_payload() {
        let (runtime, config) = test_telegram_webhook_runtime(true);
        let security = Arc::new(ServiceIngressSecurityConfig {
            telegram_secret_token: Some("expected-token".to_string()),
            discord_verifying_key: None,
            replay_window: std::time::Duration::from_secs(300),
            replay_max_entries: 1_024,
            replay_backend: IngressReplayBackendConfig::Memory,
        });
        let state = test_service_state_with_security(
            initialized_chatops_runtime(Some(runtime), config, None),
            security,
            None,
            Some(unique_test_path("telegram-replay")),
        );
        let request = concat!(
            "POST /chatops/telegram/webhook HTTP/1.1\r\n",
            "Host: localhost\r\n",
            "Content-Type: application/json\r\n",
            "X-Telegram-Bot-Api-Secret-Token: expected-token\r\n",
            "\r\n",
            "{\"update_id\":10,\"message\":{\"date\":1737200000,\"text\":\"list\",\"chat\":{\"id\":555},\"from\":{\"id\":777}}}"
        );

        let first = execute_service_request_with_state(request, state.clone()).await;
        assert!(
            first.starts_with("HTTP/1.1 202 Accepted"),
            "first telegram webhook request should be accepted"
        );

        let replay = execute_service_request_with_state(request, state).await;
        assert!(
            replay.starts_with("HTTP/1.1 409 Conflict"),
            "replayed telegram webhook request should be rejected"
        );
    }

    #[tokio::test]
    async fn service_mode_telegram_webhook_replay_is_shared_for_file_backend() {
        let (runtime, config) = test_telegram_webhook_runtime(true);
        let temp_dir = tempfile::tempdir().expect("temp dir should be created");
        let replay_path = temp_dir.path().join("ingress-replay-cache.json");
        let security = Arc::new(ServiceIngressSecurityConfig {
            telegram_secret_token: Some("expected-token".to_string()),
            discord_verifying_key: None,
            replay_window: std::time::Duration::from_secs(300),
            replay_max_entries: 1_024,
            replay_backend: IngressReplayBackendConfig::File {
                path: replay_path.clone(),
            },
        });
        let state_one = test_service_state_with_security(
            initialized_chatops_runtime(Some(runtime.clone()), config.clone(), None),
            security.clone(),
            None,
            Some(unique_test_path("telegram-replay-file-one")),
        );
        let state_two = test_service_state_with_security(
            initialized_chatops_runtime(Some(runtime), config, None),
            security,
            None,
            Some(unique_test_path("telegram-replay-file-two")),
        );
        let request = concat!(
            "POST /chatops/telegram/webhook HTTP/1.1\r\n",
            "Host: localhost\r\n",
            "Content-Type: application/json\r\n",
            "X-Telegram-Bot-Api-Secret-Token: expected-token\r\n",
            "\r\n",
            "{\"update_id\":10,\"message\":{\"date\":1737200000,\"text\":\"list\",\"chat\":{\"id\":555},\"from\":{\"id\":777}}}"
        );

        let first = execute_service_request_with_state(request, state_one).await;
        assert!(
            first.starts_with("HTTP/1.1 202 Accepted"),
            "first telegram webhook request should be accepted"
        );

        let replay = execute_service_request_with_state(request, state_two).await;
        assert!(
            replay.starts_with("HTTP/1.1 409 Conflict"),
            "replay must be detected across independent states when file backend is shared"
        );

        assert!(
            replay_path.exists(),
            "file replay backend should persist cache file"
        );
    }

    #[tokio::test]
    async fn service_mode_telegram_webhook_returns_unavailable_for_broken_file_backend() {
        let (runtime, config) = test_telegram_webhook_runtime(true);
        let temp_dir = tempfile::tempdir().expect("temp dir should be created");
        let security = Arc::new(ServiceIngressSecurityConfig {
            telegram_secret_token: Some("expected-token".to_string()),
            discord_verifying_key: None,
            replay_window: std::time::Duration::from_secs(300),
            replay_max_entries: 1_024,
            // Directory path causes write failure for file-backed replay cache.
            replay_backend: IngressReplayBackendConfig::File {
                path: temp_dir.path().to_path_buf(),
            },
        });
        let state = test_service_state_with_security(
            initialized_chatops_runtime(Some(runtime), config, None),
            security,
            None,
            Some(unique_test_path("telegram-replay-broken")),
        );
        let request = concat!(
            "POST /chatops/telegram/webhook HTTP/1.1\r\n",
            "Host: localhost\r\n",
            "Content-Type: application/json\r\n",
            "X-Telegram-Bot-Api-Secret-Token: expected-token\r\n",
            "\r\n",
            "{\"update_id\":10,\"message\":{\"date\":1737200000,\"text\":\"list\",\"chat\":{\"id\":555},\"from\":{\"id\":777}}}"
        );

        let response = execute_service_request_with_state(request, state).await;
        assert!(
            response.starts_with("HTTP/1.1 503 Service Unavailable"),
            "broken replay backend should return service unavailable"
        );
    }

    #[tokio::test]
    async fn service_mode_discord_interactions_returns_pong_for_ping() {
        let (runtime, config) = test_discord_interactions_runtime(true);
        let request = concat!(
            "POST /chatops/discord/interactions HTTP/1.1\r\n",
            "Host: localhost\r\n",
            "Content-Type: application/json\r\n",
            "\r\n",
            "{\"type\":1}"
        );
        let response = execute_service_request_with_state(
            request,
            test_service_state_with_security(
                initialized_chatops_runtime(Some(runtime), config, None),
                default_test_ingress_security(),
                None,
                Some(unique_test_path("discord-ping")),
            ),
        )
        .await;
        assert!(
            response.starts_with("HTTP/1.1 200 OK"),
            "ping interaction should return 200"
        );
        assert!(
            response.contains("\"type\":1"),
            "ping interaction should return Discord pong payload"
        );
    }

    #[tokio::test]
    async fn service_mode_discord_interactions_accepts_command_payload() {
        let (runtime, config) = test_discord_interactions_runtime(true);
        let request = concat!(
            "POST /chatops/discord/interactions HTTP/1.1\r\n",
            "Host: localhost\r\n",
            "Content-Type: application/json\r\n",
            "\r\n",
            "{\"type\":2,\"channel_id\":\"555\",\"member\":{\"user\":{\"id\":\"777\"}},\"data\":{\"name\":\"list\"}}"
        );
        let response = execute_service_request_with_state(
            request,
            test_service_state_with_security(
                initialized_chatops_runtime(Some(runtime), config, None),
                default_test_ingress_security(),
                None,
                Some(unique_test_path("discord-command")),
            ),
        )
        .await;
        assert!(
            response.starts_with("HTTP/1.1 200 OK"),
            "command interaction should return 200"
        );
        assert!(
            response.contains("\"type\":4"),
            "command interaction should return deferred message response"
        );
        assert!(
            response.contains("\"flags\":64"),
            "command interaction response should be ephemeral"
        );
    }

    #[tokio::test]
    async fn service_mode_discord_interactions_rejects_invalid_payload() {
        let (runtime, config) = test_discord_interactions_runtime(true);
        let request = concat!(
            "POST /chatops/discord/interactions HTTP/1.1\r\n",
            "Host: localhost\r\n",
            "Content-Type: application/json\r\n",
            "\r\n",
            "{invalid"
        );
        let response = execute_service_request_with_state(
            request,
            test_service_state_with_security(
                initialized_chatops_runtime(Some(runtime), config, None),
                default_test_ingress_security(),
                None,
                Some(unique_test_path("discord-invalid")),
            ),
        )
        .await;
        assert!(
            response.starts_with("HTTP/1.1 400 Bad Request"),
            "invalid interaction payload should be rejected"
        );
        assert!(
            response.contains("invalid Discord interaction payload"),
            "response should explain invalid interaction payload"
        );
    }

    #[tokio::test]
    async fn service_mode_discord_interactions_returns_conflict_when_disabled() {
        let (runtime, config) = test_discord_interactions_runtime(false);
        let request = concat!(
            "POST /chatops/discord/interactions HTTP/1.1\r\n",
            "Host: localhost\r\n",
            "Content-Type: application/json\r\n",
            "\r\n",
            "{\"type\":2,\"channel_id\":\"555\",\"member\":{\"user\":{\"id\":\"777\"}},\"data\":{\"name\":\"list\"}}"
        );
        let response = execute_service_request_with_state(
            request,
            test_service_state_with_security(
                initialized_chatops_runtime(Some(runtime), config, None),
                default_test_ingress_security(),
                None,
                Some(unique_test_path("discord-disabled")),
            ),
        )
        .await;
        assert!(
            response.starts_with("HTTP/1.1 409 Conflict"),
            "interaction endpoint should reject requests when mode is disabled"
        );
    }

    #[tokio::test]
    async fn service_mode_discord_interactions_rejects_missing_signature_when_key_configured() {
        let (runtime, config) = test_discord_interactions_runtime(true);
        let signing_key = SigningKey::from_bytes(&[7_u8; 32]);
        let security = Arc::new(ServiceIngressSecurityConfig {
            telegram_secret_token: None,
            discord_verifying_key: Some(signing_key.verifying_key()),
            replay_window: std::time::Duration::from_secs(300),
            replay_max_entries: 1_024,
            replay_backend: IngressReplayBackendConfig::Memory,
        });
        let request = concat!(
            "POST /chatops/discord/interactions HTTP/1.1\r\n",
            "Host: localhost\r\n",
            "Content-Type: application/json\r\n",
            "\r\n",
            "{\"type\":2,\"channel_id\":\"555\",\"member\":{\"user\":{\"id\":\"777\"}},\"data\":{\"name\":\"list\"}}"
        );
        let response = execute_service_request_with_state(
            request,
            test_service_state_with_security(
                initialized_chatops_runtime(Some(runtime), config, None),
                security,
                None,
                Some(unique_test_path("discord-signature-missing")),
            ),
        )
        .await;
        assert!(
            response.starts_with("HTTP/1.1 401 Unauthorized"),
            "discord interaction must reject missing signature when public key is configured"
        );
    }

    #[tokio::test]
    async fn service_mode_discord_interactions_accepts_valid_signed_request_and_rejects_replay() {
        let (runtime, config) = test_discord_interactions_runtime(true);
        let signing_key = SigningKey::from_bytes(&[11_u8; 32]);
        let security = Arc::new(ServiceIngressSecurityConfig {
            telegram_secret_token: None,
            discord_verifying_key: Some(signing_key.verifying_key()),
            replay_window: std::time::Duration::from_secs(300),
            replay_max_entries: 1_024,
            replay_backend: IngressReplayBackendConfig::Memory,
        });
        let state = test_service_state_with_security(
            initialized_chatops_runtime(Some(runtime), config, None),
            security,
            None,
            Some(unique_test_path("discord-replay")),
        );
        let body =
            "{\"type\":2,\"channel_id\":\"555\",\"member\":{\"user\":{\"id\":\"777\"}},\"data\":{\"name\":\"list\"}}";
        let timestamp = current_unix_timestamp_seconds().to_string();
        let mut message = timestamp.as_bytes().to_vec();
        message.extend_from_slice(body.as_bytes());
        let signature = signing_key.sign(&message);
        let signature_hex = hex::encode(signature.to_bytes());

        let request = format!(
            "POST /chatops/discord/interactions HTTP/1.1\r\nHost: localhost\r\nContent-Type: application/json\r\nX-Signature-Ed25519: {signature_hex}\r\nX-Signature-Timestamp: {timestamp}\r\n\r\n{body}"
        );

        let first = execute_service_request_with_state(&request, state.clone()).await;
        assert!(
            first.starts_with("HTTP/1.1 200 OK"),
            "first signed interaction should be accepted"
        );
        assert!(
            first.contains("\"type\":4"),
            "accepted signed interaction should return command acknowledgement"
        );

        let replay = execute_service_request_with_state(&request, state).await;
        assert!(
            replay.starts_with("HTTP/1.1 409 Conflict"),
            "replayed signed interaction should be rejected"
        );
    }
}
