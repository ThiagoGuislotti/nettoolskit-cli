//! Command processor implementation

use crate::execution::ai::{
    AiChunk, AiMessage, AiProvider, AiProviderError, AiRequest, AiResponse, AiRole, MockAiProvider,
    OpenAiCompatibleProvider, OpenAiCompatibleProviderConfig,
};
use crate::execution::ai_session::{
    prune_local_ai_session_snapshots, resolve_active_ai_session_id, set_active_ai_session_id,
    LocalAiSessionState,
};
use crate::execution::approval::{request_approval, ApprovalDecision, ApprovalRequest};
use crate::execution::cache::{CacheKey, CacheStats, CacheTtl, CacheValue, CommandResultCache};
use crate::execution::executor::{AsyncCommandExecutor, CommandProgress, ProgressSender};
use crate::execution::plugins::{
    command_plugin_count, run_after_command_plugins, run_before_command_plugins, CommandHookContext,
};
use crate::models::{ExitStatus, MainAction};
use nettoolskit_core::ai_context::{
    collect_workspace_context, render_context_system_message, AiContextBudget,
};
use nettoolskit_core::file_search::{search_files, SearchConfig};
use nettoolskit_core::{
    AppConfig, ColorMode, CommandEntry, RuntimeMode, TaskAuditEvent, TaskExecutionStatus,
    TaskIntent, TaskIntentKind, UnicodeMode,
};
use nettoolskit_otel::{next_correlation_id, Metrics, Timer};
use owo_colors::OwoColorize;
use std::collections::HashMap;
use std::future::Future;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::OnceLock;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;
use std::time::{Duration, Instant};
use strum::IntoEnumIterator;
use tokio::sync::mpsc;
use tokio::sync::mpsc::error::TrySendError;
use tokio::sync::Semaphore;
use tracing::{info, info_span, warn};

static RUNTIME_METRICS: OnceLock<Metrics> = OnceLock::new();
static COMMAND_CACHE: OnceLock<Mutex<CommandResultCache>> = OnceLock::new();
static AI_RATE_LIMITER: OnceLock<Mutex<AiRateLimitState>> = OnceLock::new();
static TASK_REGISTRY: OnceLock<Mutex<HashMap<String, TaskRecord>>> = OnceLock::new();
static TASK_AUDIT_REGISTRY: OnceLock<Mutex<HashMap<String, Vec<TaskAuditEvent>>>> = OnceLock::new();
static TASK_WORKER_SENDER: OnceLock<mpsc::Sender<QueuedTask>> = OnceLock::new();
static TASK_SEQUENCE: AtomicU64 = AtomicU64::new(1);
const COMMAND_CACHE_MAX_ENTRIES: usize = 128;
const COMMAND_CACHE_MAX_SIZE_BYTES: usize = 2 * 1024 * 1024;
const COMMAND_CACHE_LOG_INTERVAL_SECONDS: u64 = 30;
const TASK_AUDIT_MAX_EVENTS_PER_TASK: usize = 32;
const DEFAULT_TASK_QUEUE_CAPACITY: usize = 64;
const DEFAULT_TASK_MAX_CONCURRENCY: usize = 2;
const DEFAULT_TASK_MAX_RETRIES: usize = 2;
const DEFAULT_TASK_RETRY_BASE_DELAY_MS: u64 = 300;
const DEFAULT_TASK_RETRY_MAX_DELAY_MS: u64 = 1_500;
const AI_CONTEXT_DEFAULT_ALLOWLIST: &[&str] = &[
    "Cargo.toml",
    "README.md",
    "CHANGELOG.md",
    ".temp/planning/enterprise-progress-tracker.md",
];
const AI_SESSION_CONTEXT_MESSAGE_LIMIT: usize = 12;
const DEFAULT_AI_RATE_LIMIT_REQUESTS: usize = 30;
const DEFAULT_AI_RATE_LIMIT_WINDOW_SECONDS: u64 = 60;
const DEFAULT_AI_MAX_RETRIES: usize = 2;
const DEFAULT_AI_RETRY_BASE_DELAY_MS: u64 = 250;
const DEFAULT_AI_RETRY_MAX_DELAY_MS: u64 = 2_000;
const DEFAULT_AI_REQUEST_TIMEOUT_MS: u64 = 45_000;

#[derive(Debug, Clone, Copy)]
struct AiRateLimitPolicy {
    max_requests: usize,
    window: Duration,
}

impl Default for AiRateLimitPolicy {
    fn default() -> Self {
        Self {
            max_requests: DEFAULT_AI_RATE_LIMIT_REQUESTS,
            window: Duration::from_secs(DEFAULT_AI_RATE_LIMIT_WINDOW_SECONDS),
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct AiRetryPolicy {
    max_retries: usize,
    base_delay: Duration,
    max_delay: Duration,
    request_timeout: Duration,
}

impl Default for AiRetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: DEFAULT_AI_MAX_RETRIES,
            base_delay: Duration::from_millis(DEFAULT_AI_RETRY_BASE_DELAY_MS),
            max_delay: Duration::from_millis(DEFAULT_AI_RETRY_MAX_DELAY_MS),
            request_timeout: Duration::from_millis(DEFAULT_AI_REQUEST_TIMEOUT_MS),
        }
    }
}

#[derive(Debug)]
struct AiRateLimitState {
    window_started_at: Instant,
    used_requests: usize,
}

impl Default for AiRateLimitState {
    fn default() -> Self {
        Self {
            window_started_at: Instant::now(),
            used_requests: 0,
        }
    }
}

#[derive(Debug, Clone)]
struct TaskRecord {
    id: String,
    intent: TaskIntent,
    status: TaskExecutionStatus,
    runtime_mode: RuntimeMode,
    execution_target: String,
    status_message: String,
    attempts: usize,
    max_attempts: usize,
    created_at_unix_ms: u64,
    updated_at_unix_ms: u64,
}

impl TaskRecord {
    fn new(
        id: String,
        intent: TaskIntent,
        runtime_mode: RuntimeMode,
        max_attempts: usize,
        now_unix_ms: u64,
    ) -> Self {
        Self {
            id,
            intent,
            status: TaskExecutionStatus::Queued,
            runtime_mode,
            execution_target: "local-fallback".to_string(),
            status_message: "Queued for execution".to_string(),
            attempts: 0,
            max_attempts: max_attempts.max(1),
            created_at_unix_ms: now_unix_ms,
            updated_at_unix_ms: now_unix_ms,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct TaskWorkerPolicy {
    queue_capacity: usize,
    max_concurrency: usize,
    max_retries: usize,
    retry_base_delay: Duration,
    retry_max_delay: Duration,
}

impl Default for TaskWorkerPolicy {
    fn default() -> Self {
        Self {
            queue_capacity: DEFAULT_TASK_QUEUE_CAPACITY,
            max_concurrency: DEFAULT_TASK_MAX_CONCURRENCY,
            max_retries: DEFAULT_TASK_MAX_RETRIES,
            retry_base_delay: Duration::from_millis(DEFAULT_TASK_RETRY_BASE_DELAY_MS),
            retry_max_delay: Duration::from_millis(DEFAULT_TASK_RETRY_MAX_DELAY_MS),
        }
    }
}

#[derive(Debug, Clone)]
struct QueuedTask {
    id: String,
    intent: TaskIntent,
    runtime_mode: RuntimeMode,
}

fn runtime_metrics() -> &'static Metrics {
    RUNTIME_METRICS.get_or_init(Metrics::new)
}

fn command_cache() -> &'static Mutex<CommandResultCache> {
    COMMAND_CACHE.get_or_init(|| {
        Mutex::new(CommandResultCache::new(
            COMMAND_CACHE_MAX_ENTRIES,
            COMMAND_CACHE_MAX_SIZE_BYTES,
            CacheTtl::default(),
        ))
    })
}

fn ai_rate_limiter() -> &'static Mutex<AiRateLimitState> {
    AI_RATE_LIMITER.get_or_init(|| Mutex::new(AiRateLimitState::default()))
}

fn task_registry() -> &'static Mutex<HashMap<String, TaskRecord>> {
    TASK_REGISTRY.get_or_init(|| Mutex::new(HashMap::new()))
}

fn task_audit_registry() -> &'static Mutex<HashMap<String, Vec<TaskAuditEvent>>> {
    TASK_AUDIT_REGISTRY.get_or_init(|| Mutex::new(HashMap::new()))
}

fn with_command_cache<T>(f: impl FnOnce(&mut CommandResultCache) -> T) -> T {
    let mut guard = command_cache()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    f(&mut guard)
}

fn with_task_registry<T>(f: impl FnOnce(&mut HashMap<String, TaskRecord>) -> T) -> T {
    let mut guard = task_registry()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    f(&mut guard)
}

fn with_task_audit_registry<T>(
    f: impl FnOnce(&mut HashMap<String, Vec<TaskAuditEvent>>) -> T,
) -> T {
    let mut guard = task_audit_registry()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    f(&mut guard)
}

fn append_task_audit_event(
    task_id: &str,
    runtime_mode: RuntimeMode,
    status: TaskExecutionStatus,
    message: impl Into<String>,
) {
    let event = TaskAuditEvent::new(
        task_id.to_string(),
        runtime_mode,
        status,
        message,
        current_unix_timestamp_ms(),
    );
    with_task_audit_registry(|registry| {
        let events = registry.entry(task_id.to_string()).or_default();
        events.push(event);
        if events.len() > TASK_AUDIT_MAX_EVENTS_PER_TASK {
            let extra = events.len() - TASK_AUDIT_MAX_EVENTS_PER_TASK;
            events.drain(0..extra);
        }
    });
}

fn list_task_audit_events(task_id: &str) -> Vec<TaskAuditEvent> {
    with_task_audit_registry(|registry| registry.get(task_id).cloned().unwrap_or_default())
}

fn maybe_log_command_cache_stats(stats: CacheStats, metrics: &Metrics) {
    metrics.set_gauge("runtime_command_cache_entries", stats.entries as f64);
    metrics.set_gauge("runtime_command_cache_size_bytes", stats.size_bytes as f64);
    metrics.set_gauge("runtime_command_cache_hits_total", stats.hits as f64);
    metrics.set_gauge("runtime_command_cache_misses_total", stats.misses as f64);
    metrics.set_gauge(
        "runtime_command_cache_evictions_total",
        stats.evictions as f64,
    );

    let uptime_seconds = metrics
        .get_gauge("runtime_uptime_seconds")
        .unwrap_or(0.0)
        .max(0.0);
    let log_interval = COMMAND_CACHE_LOG_INTERVAL_SECONDS as f64;
    let should_log = COMMAND_CACHE_LOG_INTERVAL_SECONDS > 0
        && (uptime_seconds as u64) % COMMAND_CACHE_LOG_INTERVAL_SECONDS == 0
        && uptime_seconds > 0.0;

    if should_log && uptime_seconds % log_interval < 1.0 {
        tracing::debug!(
            cache_entries = stats.entries,
            cache_size_bytes = stats.size_bytes,
            cache_hits = stats.hits,
            cache_misses = stats.misses,
            cache_evictions = stats.evictions,
            "Runtime command cache stats snapshot"
        );
    }
}

fn sanitize_metric_component(input: &str) -> String {
    let mut normalized = String::with_capacity(input.len());
    let mut previous_was_separator = false;

    for ch in input.trim().to_ascii_lowercase().chars() {
        if ch.is_ascii_alphanumeric() {
            normalized.push(ch);
            previous_was_separator = false;
        } else if !previous_was_separator {
            normalized.push('_');
            previous_was_separator = true;
        }
    }

    let trimmed = normalized.trim_matches('_');
    if trimmed.is_empty() {
        "unknown".to_string()
    } else {
        trimmed.to_string()
    }
}

fn command_metric_key(parsed: Option<MainAction>, subcommand: Option<&str>, cmd: &str) -> String {
    match parsed {
        Some(MainAction::Manifest) => {
            if let Some(sub) = subcommand {
                format!("manifest_{}", sanitize_metric_component(sub))
            } else {
                "manifest_menu".to_string()
            }
        }
        Some(action) => sanitize_metric_component(action.slash_static().trim_start_matches('/')),
        None => {
            let token = cmd
                .trim()
                .trim_start_matches('/')
                .split_whitespace()
                .next()
                .unwrap_or("unknown");
            format!("unknown_{}", sanitize_metric_component(token))
        }
    }
}

fn record_command_outcome_metrics(
    metrics: &Metrics,
    command_key: &str,
    status: ExitStatus,
) -> &'static str {
    let status_label = match status {
        ExitStatus::Success => "success",
        ExitStatus::Error => "error",
        ExitStatus::Interrupted => "interrupted",
    };

    metrics.increment_counter(format!("runtime_commands_{status_label}_total"));
    metrics.increment_counter(format!(
        "runtime_command_{command_key}_{status_label}_total"
    ));
    status_label
}

fn update_runtime_rate_gauges(metrics: &Metrics) {
    let total = metrics.get_counter("runtime_commands_total");
    if total == 0 {
        return;
    }

    let total_f64 = total as f64;
    let successes = metrics.get_counter("runtime_commands_success_total") as f64;
    let errors = metrics.get_counter("runtime_commands_error_total") as f64;
    let interrupted = metrics.get_counter("runtime_commands_interrupted_total") as f64;

    metrics.set_gauge(
        "runtime_command_success_rate_pct",
        (successes / total_f64) * 100.0,
    );
    metrics.set_gauge(
        "runtime_command_error_rate_pct",
        (errors / total_f64) * 100.0,
    );
    metrics.set_gauge(
        "runtime_command_cancellation_rate_pct",
        (interrupted / total_f64) * 100.0,
    );
}

fn update_runtime_latency_gauges(
    metrics: &Metrics,
    command_key: &str,
    command_timing_name: &str,
    duration: Duration,
) {
    metrics.record_timing("runtime_command_latency_all", duration);
    metrics.set_gauge(
        "runtime_last_command_duration_ms",
        duration.as_secs_f64() * 1000.0,
    );

    if let Some(avg_all) = metrics.get_average_timing("runtime_command_latency_all") {
        metrics.set_gauge(
            "runtime_command_avg_latency_ms",
            avg_all.as_secs_f64() * 1000.0,
        );
    }

    if let Some(avg_cmd) = metrics.get_average_timing(command_timing_name) {
        metrics.set_gauge(
            format!("runtime_command_{command_key}_avg_latency_ms"),
            avg_cmd.as_secs_f64() * 1000.0,
        );
    }
}

fn has_flag(parts: &[&str], flag: &str) -> bool {
    parts.iter().any(|part| *part == flag)
}

fn first_positional_path(parts: &[&str], start_index: usize) -> Option<PathBuf> {
    parts
        .iter()
        .skip(start_index)
        .find(|part| !part.starts_with("--"))
        .map(|part| PathBuf::from(*part))
}

fn first_manifest_positional_path(parts: &[&str]) -> Option<PathBuf> {
    first_positional_path(parts, 2)
}

fn parse_output_root(parts: &[&str]) -> Option<PathBuf> {
    parts.windows(2).find_map(|window| {
        if window[0] == "--output" && !window[1].starts_with("--") {
            Some(PathBuf::from(window[1]))
        } else {
            None
        }
    })
}

fn discover_manifest_files(root: &Path) -> Result<Vec<PathBuf>, String> {
    let config = SearchConfig {
        include_patterns: vec![
            "*manifest*.yml".to_string(),
            "*manifest*.yaml".to_string(),
            "ntk-*.yml".to_string(),
            "ntk-*.yaml".to_string(),
        ],
        exclude_patterns: vec!["**/target/**".to_string(), "**/node_modules/**".to_string()],
        max_depth: Some(8),
        follow_links: false,
        include_hidden: false,
    };

    let mut manifests =
        search_files(root, &config).map_err(|err| format!("manifest discovery failed: {err}"))?;
    manifests.sort();
    manifests.dedup();
    Ok(manifests)
}

fn relative_path_for_display(root: &Path, path: &Path) -> String {
    match path.strip_prefix(root) {
        Ok(relative) => relative.display().to_string(),
        Err(_) => path.display().to_string(),
    }
}

fn print_manifest_validation(
    path: &Path,
    validation: &nettoolskit_manifest::handlers::ValidationResult,
) {
    use nettoolskit_ui::Color;

    let location_text = path.display().to_string();
    let location = location_text.color(Color::CYAN);
    if validation.is_valid() {
        println!(
            "{} {} {}",
            "✓".color(Color::GREEN),
            location.bold(),
            "is valid".color(Color::GREEN)
        );
    } else {
        println!(
            "{} {} {}",
            "✗".color(Color::RED),
            location.bold(),
            "has validation errors".color(Color::RED)
        );

        for error in &validation.errors {
            if let Some(line) = error.line {
                println!(
                    "  {} [line {}] {}",
                    "error".color(Color::RED),
                    line,
                    error.message
                );
            } else {
                println!("  {} {}", "error".color(Color::RED), error.message);
            }
        }
    }

    if !validation.warnings.is_empty() {
        for warning in &validation.warnings {
            if let Some(line) = warning.line {
                println!(
                    "  {} [line {}] {}",
                    "warning".color(Color::YELLOW),
                    line,
                    warning.message
                );
            } else {
                println!("  {} {}", "warning".color(Color::YELLOW), warning.message);
            }
        }
    }

    println!(
        "  errors: {}, warnings: {}",
        validation.error_count(),
        validation.warning_count()
    );
    println!();
}

fn resolve_manifest_target_path_from(
    parts: &[&str],
    action_label: &str,
    start_index: usize,
) -> Result<PathBuf, ExitStatus> {
    use nettoolskit_ui::Color;

    if let Some(path) = first_positional_path(parts, start_index) {
        return Ok(path);
    }

    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let manifests = match discover_manifest_files(&cwd) {
        Ok(found) => found,
        Err(err) => {
            println!("{} {}", "✗".color(Color::RED), err.color(Color::RED));
            return Err(ExitStatus::Error);
        }
    };

    match manifests.len() {
        0 => {
            println!(
                "{}",
                format!("No manifest files found for {action_label}.").color(Color::YELLOW)
            );
            println!(
                "{}",
                format!("Provide an explicit path: /manifest {action_label} <manifest-file>")
                    .color(Color::YELLOW)
            );
            Err(ExitStatus::Error)
        }
        1 => Ok(manifests[0].clone()),
        _ => {
            println!(
                "{}",
                format!(
                    "Multiple manifests detected ({}). Specify which one to use:",
                    manifests.len()
                )
                .color(Color::YELLOW)
            );
            for path in manifests {
                println!("  - {}", relative_path_for_display(&cwd, &path));
            }
            println!();
            println!(
                "{}",
                format!(
                    "Use: /manifest {action_label} <manifest-file> [--dry-run] [--output <dir>]"
                )
                .color(Color::YELLOW)
            );
            Err(ExitStatus::Error)
        }
    }
}

fn resolve_manifest_target_path(parts: &[&str], action_label: &str) -> Result<PathBuf, ExitStatus> {
    resolve_manifest_target_path_from(parts, action_label, 2)
}

fn print_execution_summary(summary: &nettoolskit_manifest::core::models::ExecutionSummary) {
    use nettoolskit_ui::Color;

    if !summary.created.is_empty() {
        println!(
            "{}",
            format!("Files to create: {}", summary.created.len()).color(Color::GREEN)
        );
        for path in &summary.created {
            println!("  + {}", path.display());
        }
        println!();
    }

    if !summary.updated.is_empty() {
        println!(
            "{}",
            format!("Files to update: {}", summary.updated.len()).color(Color::GREEN)
        );
        for path in &summary.updated {
            println!("  ~ {}", path.display());
        }
        println!();
    }

    if !summary.skipped.is_empty() {
        println!(
            "{}",
            format!("Files to skip: {}", summary.skipped.len()).color(Color::YELLOW)
        );
        for (path, reason) in &summary.skipped {
            println!("  - {} ({reason})", path.display());
        }
        println!();
    }

    if !summary.notes.is_empty() {
        println!("{}", "Notes:".color(Color::CYAN));
        for note in &summary.notes {
            println!("  • {note}");
        }
        println!();
    }

    println!(
        "Total artifacts: {}",
        summary.created.len() + summary.updated.len()
    );
}

fn parse_translate_request(
    parts: &[&str],
) -> Result<nettoolskit_translate::TranslateRequest, String> {
    let mut from: Option<String> = None;
    let mut to: Option<String> = None;
    let mut path: Option<String> = None;
    let mut index = 1;

    while index < parts.len() {
        match parts[index] {
            "--from" => {
                let value = parts
                    .get(index + 1)
                    .ok_or_else(|| "missing value for --from".to_string())?;
                if value.starts_with("--") {
                    return Err("missing value for --from".to_string());
                }
                from = Some((*value).to_string());
                index += 2;
            }
            "--to" => {
                let value = parts
                    .get(index + 1)
                    .ok_or_else(|| "missing value for --to".to_string())?;
                if value.starts_with("--") {
                    return Err("missing value for --to".to_string());
                }
                to = Some((*value).to_string());
                index += 2;
            }
            other if other.starts_with("--") => {
                return Err(format!("unknown flag '{other}'"));
            }
            positional => {
                if path.is_some() {
                    return Err(format!(
                        "unexpected positional argument '{positional}' (only one template path is allowed)"
                    ));
                }
                path = Some(positional.to_string());
                index += 1;
            }
        }
    }

    let from = from.ok_or_else(|| "missing --from <language>".to_string())?;
    let to = to.ok_or_else(|| "missing --to <language>".to_string())?;
    let path = path.ok_or_else(|| "missing template path".to_string())?;

    Ok(nettoolskit_translate::TranslateRequest { from, to, path })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AiIntent {
    Ask,
    Plan,
    Explain,
    ApplyDryRun,
}

impl AiIntent {
    fn from_subcommand(subcommand: &str) -> Option<Self> {
        match subcommand.trim().to_ascii_lowercase().as_str() {
            "ask" => Some(Self::Ask),
            "plan" => Some(Self::Plan),
            "explain" => Some(Self::Explain),
            "apply" => Some(Self::ApplyDryRun),
            _ => None,
        }
    }

    fn as_label(self) -> &'static str {
        match self {
            Self::Ask => "ask",
            Self::Plan => "plan",
            Self::Explain => "explain",
            Self::ApplyDryRun => "apply",
        }
    }

    fn system_prompt(self) -> &'static str {
        match self {
            Self::Ask => {
                "You are NetToolsKit CLI assistant. Provide concise, actionable engineering answers."
            }
            Self::Plan => {
                "You are NetToolsKit CLI planning assistant. Return step-by-step implementation plans with validation and risks."
            }
            Self::Explain => {
                "You are NetToolsKit CLI explainer. Clarify technical behavior with practical examples."
            }
            Self::ApplyDryRun => {
                "You are NetToolsKit CLI dry-run assistant. Propose safe, non-destructive patch steps and explain expected impact."
            }
        }
    }
}

fn ai_prompt_start(parts: &[&str]) -> usize {
    if parts.first().copied() == Some("/") {
        3
    } else {
        2
    }
}

fn collect_ai_prompt(parts: &[&str], start_index: usize) -> String {
    parts
        .iter()
        .skip(start_index)
        .filter(|part| !part.starts_with("--"))
        .copied()
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .to_string()
}

fn parse_timeout_millis(value: &str) -> Option<u64> {
    let parsed = value.trim().parse::<u64>().ok()?;
    (parsed > 0).then_some(parsed)
}

fn parse_nonzero_usize(value: &str) -> Option<usize> {
    let parsed = value.trim().parse::<usize>().ok()?;
    (parsed > 0).then_some(parsed)
}

fn parse_positive_f64(value: &str) -> Option<f64> {
    let parsed = value.trim().parse::<f64>().ok()?;
    (parsed > 0.0).then_some(parsed)
}

fn ai_retry_policy_from_env() -> AiRetryPolicy {
    let mut policy = AiRetryPolicy::default();

    if let Ok(value) = std::env::var("NTK_AI_MAX_RETRIES") {
        if let Some(parsed) = parse_nonzero_usize(&value) {
            policy.max_retries = parsed;
        }
    }

    if let Ok(value) = std::env::var("NTK_AI_RETRY_BASE_MS") {
        if let Some(parsed) = parse_timeout_millis(&value) {
            policy.base_delay = Duration::from_millis(parsed);
        }
    }

    if let Ok(value) = std::env::var("NTK_AI_RETRY_MAX_MS") {
        if let Some(parsed) = parse_timeout_millis(&value) {
            policy.max_delay = Duration::from_millis(parsed);
        }
    }

    if policy.max_delay < policy.base_delay {
        policy.max_delay = policy.base_delay;
    }

    if let Ok(value) = std::env::var("NTK_AI_REQUEST_TIMEOUT_MS") {
        if let Some(parsed) = parse_timeout_millis(&value) {
            policy.request_timeout = Duration::from_millis(parsed);
        }
    }

    policy
}

fn ai_rate_limit_policy_from_env() -> AiRateLimitPolicy {
    let mut policy = AiRateLimitPolicy::default();

    if let Ok(value) = std::env::var("NTK_AI_RATE_LIMIT_REQUESTS") {
        if let Some(parsed) = parse_nonzero_usize(&value) {
            policy.max_requests = parsed;
        }
    }

    if let Ok(value) = std::env::var("NTK_AI_RATE_LIMIT_WINDOW_SECONDS") {
        if let Some(parsed) = parse_timeout_millis(&value) {
            policy.window = Duration::from_secs(parsed);
        }
    }

    policy
}

fn ai_retry_delay(policy: AiRetryPolicy, retry_number: usize) -> Duration {
    let exponent = retry_number.saturating_sub(1).min(16);
    let multiplier = 1_u64 << exponent;
    let base_ms = policy.base_delay.as_millis() as u64;
    let max_ms = policy.max_delay.as_millis() as u64;
    let delay_ms = base_ms.saturating_mul(multiplier).min(max_ms).max(base_ms);
    Duration::from_millis(delay_ms)
}

fn is_retriable_ai_error(error: &AiProviderError) -> bool {
    matches!(
        error,
        AiProviderError::Timeout { .. }
            | AiProviderError::Unavailable(_)
            | AiProviderError::Transport(_)
    )
}

fn ai_provider_health_metric_name(provider_id: &str) -> String {
    format!(
        "runtime_ai_provider_{}_health",
        sanitize_metric_component(provider_id)
    )
}

fn set_ai_provider_health(metrics: &Metrics, provider_id: &str, healthy: bool) {
    let value = if healthy { 1.0 } else { 0.0 };
    metrics.set_gauge("runtime_ai_provider_health", value);
    metrics.set_gauge(ai_provider_health_metric_name(provider_id), value);
}

fn evaluate_ai_rate_limit(
    state: &mut AiRateLimitState,
    policy: AiRateLimitPolicy,
    now: Instant,
) -> Result<usize, u64> {
    let elapsed = now
        .checked_duration_since(state.window_started_at)
        .unwrap_or(Duration::ZERO);

    if elapsed >= policy.window {
        state.window_started_at = now;
        state.used_requests = 0;
    }

    let window_elapsed = now
        .checked_duration_since(state.window_started_at)
        .unwrap_or(Duration::ZERO);
    if state.used_requests >= policy.max_requests {
        let retry_after = policy
            .window
            .saturating_sub(window_elapsed)
            .as_secs()
            .max(1);
        return Err(retry_after);
    }

    state.used_requests = state.used_requests.saturating_add(1);
    Ok(policy.max_requests.saturating_sub(state.used_requests))
}

fn enforce_ai_rate_limit(metrics: &Metrics) -> Result<(), u64> {
    let policy = ai_rate_limit_policy_from_env();
    let mut limiter = ai_rate_limiter()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    match evaluate_ai_rate_limit(&mut limiter, policy, Instant::now()) {
        Ok(remaining) => {
            metrics.set_gauge("runtime_ai_rate_limit_remaining", remaining as f64);
            metrics.set_gauge("runtime_ai_rate_limit_retry_after_seconds", 0.0);
            Ok(())
        }
        Err(retry_after) => {
            metrics.increment_counter("runtime_ai_rate_limited_total");
            metrics.set_gauge("runtime_ai_rate_limit_remaining", 0.0);
            metrics.set_gauge(
                "runtime_ai_rate_limit_retry_after_seconds",
                retry_after as f64,
            );
            Err(retry_after)
        }
    }
}

fn update_ai_approval_ratio(metrics: &Metrics) {
    let total = metrics.get_counter("runtime_ai_approvals_total");
    if total == 0 {
        return;
    }

    let approved = metrics.get_counter("runtime_ai_approvals_approved_total");
    let ratio = (approved as f64 / total as f64) * 100.0;
    metrics.set_gauge("runtime_ai_approval_ratio_pct", ratio);
}

fn update_ai_approval_metrics(metrics: &Metrics, approved: bool) {
    metrics.increment_counter("runtime_ai_approvals_total");
    if approved {
        metrics.increment_counter("runtime_ai_approvals_approved_total");
    } else {
        metrics.increment_counter("runtime_ai_approvals_denied_total");
    }
    update_ai_approval_ratio(metrics);
}

fn update_ai_request_rate_gauges(metrics: &Metrics) {
    let total = metrics.get_counter("runtime_ai_requests_total");
    if total == 0 {
        return;
    }

    let total_f64 = total as f64;
    let success = metrics.get_counter("runtime_ai_requests_success_total") as f64;
    let errors = metrics.get_counter("runtime_ai_requests_error_total") as f64;
    let timeouts = metrics.get_counter("runtime_ai_requests_timeout_total") as f64;

    metrics.set_gauge("runtime_ai_success_rate_pct", (success / total_f64) * 100.0);
    metrics.set_gauge("runtime_ai_error_rate_pct", (errors / total_f64) * 100.0);
    metrics.set_gauge(
        "runtime_ai_timeout_rate_pct",
        (timeouts / total_f64) * 100.0,
    );
}

fn estimate_token_count(text: &str) -> u64 {
    let chars = text.chars().count() as u64;
    chars.div_ceil(4).max(1)
}

fn estimate_request_input_tokens(request: &AiRequest) -> u64 {
    request
        .messages
        .iter()
        .map(|message| estimate_token_count(&message.content))
        .sum()
}

fn add_gauge_value(metrics: &Metrics, name: &str, delta: f64) {
    let updated = metrics.get_gauge(name).unwrap_or(0.0) + delta;
    metrics.set_gauge(name, updated);
}

fn ai_input_cost_rate_per_1k_from_env() -> f64 {
    std::env::var("NTK_AI_COST_PER_1K_INPUT_USD")
        .ok()
        .and_then(|value| parse_positive_f64(&value))
        .unwrap_or(0.0)
}

fn ai_output_cost_rate_per_1k_from_env() -> f64 {
    std::env::var("NTK_AI_COST_PER_1K_OUTPUT_USD")
        .ok()
        .and_then(|value| parse_positive_f64(&value))
        .unwrap_or(0.0)
}

fn record_ai_usage_estimates(metrics: &Metrics, request: &AiRequest, output: &str) {
    let input_tokens = estimate_request_input_tokens(request);
    let output_tokens = estimate_token_count(output);
    let total_tokens = input_tokens.saturating_add(output_tokens);

    metrics.increment_counter("runtime_ai_usage_samples_total");
    metrics.set_gauge("runtime_ai_last_input_tokens_estimate", input_tokens as f64);
    metrics.set_gauge(
        "runtime_ai_last_output_tokens_estimate",
        output_tokens as f64,
    );
    metrics.set_gauge("runtime_ai_last_total_tokens_estimate", total_tokens as f64);
    add_gauge_value(
        metrics,
        "runtime_ai_input_tokens_estimated_total",
        input_tokens as f64,
    );
    add_gauge_value(
        metrics,
        "runtime_ai_output_tokens_estimated_total",
        output_tokens as f64,
    );
    add_gauge_value(
        metrics,
        "runtime_ai_tokens_estimated_total",
        total_tokens as f64,
    );

    let input_rate = ai_input_cost_rate_per_1k_from_env();
    let output_rate = ai_output_cost_rate_per_1k_from_env();
    let estimated_cost = ((input_tokens as f64 / 1000.0) * input_rate)
        + ((output_tokens as f64 / 1000.0) * output_rate);

    metrics.set_gauge("runtime_ai_last_cost_estimate_usd", estimated_cost);
    add_gauge_value(
        metrics,
        "runtime_ai_cost_estimate_total_usd",
        estimated_cost,
    );
}

fn ai_error_guidance_message(error: &AiProviderError) -> Option<&'static str> {
    match error {
        AiProviderError::Timeout { .. } => Some(
            "Provider timeout reached. Try lowering context size or adjusting NTK_AI_TIMEOUT_MS / NTK_AI_REQUEST_TIMEOUT_MS.",
        ),
        AiProviderError::Unavailable(_) => Some(
            "Provider is unavailable. You can retry, or switch to NTK_AI_PROVIDER=mock while service recovers.",
        ),
        AiProviderError::Transport(_) => Some(
            "Provider transport failed. Verify endpoint/network/API key or set NTK_AI_FALLBACK_TEXT for degraded mode.",
        ),
        AiProviderError::InvalidResponse(_) => Some(
            "Provider returned malformed output. Check model compatibility and endpoint schema.",
        ),
        AiProviderError::InvalidRequest(_) => None,
    }
}

async fn request_ai_stream_with_retry(
    provider: &dyn AiProvider,
    request: &AiRequest,
    retry_policy: AiRetryPolicy,
    metrics: &Metrics,
    intent: AiIntent,
) -> Result<(Vec<AiChunk>, usize), AiProviderError> {
    let mut retries = 0usize;

    loop {
        let attempt_started = Instant::now();
        let result = match tokio::time::timeout(
            retry_policy.request_timeout,
            provider.stream(request.clone()),
        )
        .await
        {
            Ok(inner) => inner,
            Err(_) => Err(AiProviderError::Timeout {
                timeout: retry_policy.request_timeout,
            }),
        };
        metrics.record_timing("runtime_ai_attempt_latency", attempt_started.elapsed());

        match result {
            Ok(chunks) => return Ok((chunks, retries)),
            Err(error) => {
                let can_retry = retries < retry_policy.max_retries && is_retriable_ai_error(&error);
                if !can_retry {
                    return Err(error);
                }

                retries = retries.saturating_add(1);
                metrics.increment_counter("runtime_ai_retries_total");
                metrics.increment_counter(format!(
                    "runtime_ai_intent_{}_retries_total",
                    intent.as_label()
                ));
                let delay = ai_retry_delay(retry_policy, retries);
                metrics.set_gauge("runtime_ai_last_retry_delay_ms", delay.as_millis() as f64);
                let _ = nettoolskit_ui::append_footer_log(&format!(
                    "ai: transient failure, retry {}/{} in {}ms",
                    retries,
                    retry_policy.max_retries,
                    delay.as_millis()
                ));
                tokio::time::sleep(delay).await;
            }
        }
    }
}

fn parse_ai_context_paths(value: &str) -> Vec<PathBuf> {
    value
        .split([',', ';'])
        .map(str::trim)
        .filter(|path| !path.is_empty())
        .map(PathBuf::from)
        .collect()
}

fn ai_context_budget_from_env() -> AiContextBudget {
    let mut budget = AiContextBudget::default();

    if let Ok(value) = std::env::var("NTK_AI_CONTEXT_MAX_FILES") {
        if let Some(parsed) = parse_nonzero_usize(&value) {
            budget.max_files = parsed;
        }
    }

    if let Ok(value) = std::env::var("NTK_AI_CONTEXT_MAX_FILE_BYTES") {
        if let Some(parsed) = parse_nonzero_usize(&value) {
            budget.max_file_bytes = parsed;
        }
    }

    if let Ok(value) = std::env::var("NTK_AI_CONTEXT_MAX_BYTES") {
        if let Some(parsed) = parse_nonzero_usize(&value) {
            budget.max_total_bytes = parsed;
        }
    }

    budget
}

fn ai_context_allowlist_paths() -> Vec<PathBuf> {
    let mut paths = AI_CONTEXT_DEFAULT_ALLOWLIST
        .iter()
        .map(PathBuf::from)
        .collect::<Vec<_>>();

    if let Ok(extra_paths) = std::env::var("NTK_AI_CONTEXT_PATHS") {
        paths.extend(parse_ai_context_paths(&extra_paths));
    }

    paths
}

fn build_ai_context_system_message() -> Option<String> {
    let workspace_root = std::env::current_dir().ok()?;
    let allowlist = ai_context_allowlist_paths();
    if allowlist.is_empty() {
        return None;
    }

    let bundle =
        collect_workspace_context(&workspace_root, &allowlist, ai_context_budget_from_env());
    if bundle.is_empty() {
        return None;
    }

    render_context_system_message(&bundle)
}

fn mocked_ai_response(intent: AiIntent, prompt: &str) -> AiResponse {
    let mut preview = prompt.chars().take(96).collect::<String>();
    if prompt.chars().count() > 96 {
        preview.push_str("...");
    }

    let content = match intent {
        AiIntent::Ask => format!(
            "Mock AI answer: `{preview}`\nConfigure NTK_AI_PROVIDER=openai and NTK_AI_API_KEY to use a live model."
        ),
        AiIntent::Plan => format!(
            "Mock AI plan for `{preview}`:\n1. Define scope\n2. Implement in slices\n3. Validate with tests and clippy"
        ),
        AiIntent::Explain => format!(
            "Mock AI explanation for `{preview}`:\n- Inputs parsed\n- Action executed\n- Output reported"
        ),
        AiIntent::ApplyDryRun => format!(
            "Mock AI dry-run apply for `{preview}`:\n- Proposed file changes only\n- No mutation performed"
        ),
    };

    AiResponse::new("mock-assistant", content)
}

fn ai_provider_from_env(intent: AiIntent, prompt: &str) -> Result<Box<dyn AiProvider>, String> {
    let provider_name = std::env::var("NTK_AI_PROVIDER")
        .unwrap_or_else(|_| "mock".to_string())
        .trim()
        .to_ascii_lowercase();

    if provider_name == "openai" || provider_name == "openai-compatible" {
        let mut config = OpenAiCompatibleProviderConfig::default();
        if let Ok(endpoint) = std::env::var("NTK_AI_ENDPOINT") {
            if !endpoint.trim().is_empty() {
                config.endpoint = endpoint;
            }
        }
        if let Ok(api_key) = std::env::var("NTK_AI_API_KEY") {
            if !api_key.trim().is_empty() {
                config.api_key = Some(api_key);
            }
        }
        if let Ok(model) = std::env::var("NTK_AI_MODEL") {
            if !model.trim().is_empty() {
                config.default_model = model;
            }
        }
        if let Ok(timeout_ms) = std::env::var("NTK_AI_TIMEOUT_MS") {
            if let Some(value) = parse_timeout_millis(&timeout_ms) {
                config.timeout = Duration::from_millis(value);
            }
        }
        if let Ok(fallback) = std::env::var("NTK_AI_FALLBACK_TEXT") {
            if !fallback.trim().is_empty() {
                config.fallback_output_text = Some(fallback);
            }
        }

        let provider = OpenAiCompatibleProvider::new(config).map_err(|error| error.to_string())?;
        return Ok(Box::new(provider));
    }

    Ok(Box::new(MockAiProvider::new(mocked_ai_response(
        intent, prompt,
    ))))
}

fn build_ai_request(intent: AiIntent, prompt: &str) -> AiRequest {
    let mut request = AiRequest::from_user_prompt(prompt.to_string());
    request.stream = true;
    request.messages.insert(
        0,
        AiMessage::new(AiRole::System, intent.system_prompt().to_string()),
    );

    match intent {
        AiIntent::Ask => {
            request.temperature = Some(0.4);
            request.max_output_tokens = Some(1024);
        }
        AiIntent::Plan => {
            request.temperature = Some(0.2);
            request.max_output_tokens = Some(1400);
        }
        AiIntent::Explain => {
            request.temperature = Some(0.3);
            request.max_output_tokens = Some(1200);
        }
        AiIntent::ApplyDryRun => {
            request.temperature = Some(0.1);
            request.max_output_tokens = Some(1600);
        }
    }

    request
}

fn ai_session_retention_limit() -> usize {
    AppConfig::load().general.ai_session_retention.max(1)
}

fn default_ai_session_retention() -> usize {
    AppConfig::default().general.ai_session_retention.max(1)
}

fn load_or_initialize_ai_session(session_id: &str) -> LocalAiSessionState {
    match LocalAiSessionState::load_local_snapshot(session_id) {
        Ok(Some(session)) => session,
        Ok(None) => LocalAiSessionState::new(session_id),
        Err(err) => {
            warn!(
                error = %err,
                session_id,
                "Failed to load AI session snapshot; starting a new one"
            );
            LocalAiSessionState::new(session_id)
        }
    }
}

fn inject_ai_session_context(request: &mut AiRequest, session: &LocalAiSessionState) {
    let history_messages = session.recent_messages(AI_SESSION_CONTEXT_MESSAGE_LIMIT);
    if history_messages.is_empty() {
        return;
    }

    let insertion_index = request.messages.len().saturating_sub(1);
    request
        .messages
        .splice(insertion_index..insertion_index, history_messages);
}

fn persist_ai_session_exchange(
    session_id: &str,
    intent: AiIntent,
    provider_id: &str,
    prompt: &str,
    output: &str,
) {
    let mut session = load_or_initialize_ai_session(session_id);
    if !session.append_exchange(intent.as_label(), provider_id, prompt, output) {
        return;
    }

    match session.save_local_snapshot() {
        Ok(Some(path)) => {
            let _ = nettoolskit_ui::append_footer_log(&format!(
                "ai: session persisted id={} path={}",
                session.id,
                path.display()
            ));
        }
        Ok(None) => {
            let _ = nettoolskit_ui::append_footer_log(
                "ai: local data dir unavailable; session persistence skipped",
            );
        }
        Err(err) => {
            warn!(
                error = %err,
                session_id,
                "Failed to persist AI session snapshot"
            );
            let _ = nettoolskit_ui::append_footer_log(&format!(
                "ai: failed to persist session snapshot - {err}"
            ));
        }
    }

    let retention = ai_session_retention_limit();
    match prune_local_ai_session_snapshots(retention) {
        Ok(Some(removed)) if removed > 0 => {
            let _ = nettoolskit_ui::append_footer_log(&format!(
                "ai: pruned {removed} old session snapshot(s)"
            ));
        }
        Ok(Some(_)) | Ok(None) => {}
        Err(err) => {
            warn!(error = %err, "Failed to prune AI session snapshots");
            let _ = nettoolskit_ui::append_footer_log(&format!(
                "ai: failed to prune session snapshots - {err}"
            ));
        }
    }
}

fn handle_ai_resume_subcommand(parts: &[&str]) -> ExitStatus {
    use nettoolskit_ui::Color;

    let prompt_start = ai_prompt_start(parts);
    let session_id = collect_ai_prompt(parts, prompt_start);
    if session_id.is_empty() {
        println!(
            "{}",
            "Usage: /ai resume <session-id> (interactive picker available with `/ai resume` in CLI mode)"
                .color(Color::YELLOW)
        );
        return ExitStatus::Error;
    }

    let active_session = set_active_ai_session_id(&session_id);
    match LocalAiSessionState::load_local_snapshot(&active_session) {
        Ok(Some(session)) => {
            println!(
                "{} {}",
                "✅ Active AI session resumed:".color(Color::GREEN).bold(),
                session.id.color(Color::CYAN)
            );
            println!(
                "  exchanges:{} last:{}",
                session.exchanges.len(),
                session.last_activity_ms
            );
        }
        Ok(None) => {
            println!(
                "{} {}",
                "✅ Active AI session set:".color(Color::GREEN).bold(),
                active_session.color(Color::CYAN)
            );
            println!(
                "{}",
                "No persisted history found for this id yet; a new local session snapshot will be created on next AI response."
                    .color(Color::YELLOW)
            );
        }
        Err(err) => {
            println!(
                "{} {}",
                "⚠ Active AI session set, but failed to read local snapshot:"
                    .color(Color::YELLOW)
                    .bold(),
                err.to_string().color(Color::YELLOW)
            );
        }
    }

    let _ =
        nettoolskit_ui::append_footer_log(&format!("ai: active session set id={active_session}"));
    ExitStatus::Success
}

async fn process_ai_command(parts: &[&str], subcommand: Option<&str>) -> ExitStatus {
    use nettoolskit_ui::Color;

    let Some(raw_subcommand) = subcommand else {
        println!("{}", "🤖 AI Assistant Commands".color(Color::CYAN).bold());
        println!("  {}", "/ai ask <prompt>".color(Color::GREEN));
        println!("  {}", "/ai plan <goal>".color(Color::GREEN));
        println!("  {}", "/ai explain <topic>".color(Color::GREEN));
        println!("  {}", "/ai resume <session-id>".color(Color::GREEN));
        println!(
            "  {}",
            "/ai apply --dry-run <instruction>".color(Color::GREEN)
        );
        println!(
            "  {}",
            "/ai apply --approve-write <instruction>".color(Color::YELLOW)
        );
        println!();
        println!(
            "{}",
            "Use NTK_AI_PROVIDER=openai to enable live provider calls (defaults to mock)."
                .color(Color::YELLOW)
        );
        println!(
            "{}",
            "Operational controls: NTK_AI_MAX_RETRIES, NTK_AI_REQUEST_TIMEOUT_MS, NTK_AI_RATE_LIMIT_REQUESTS, NTK_AI_RATE_LIMIT_WINDOW_SECONDS."
                .color(Color::YELLOW)
        );
        return ExitStatus::Success;
    };

    if raw_subcommand.trim().eq_ignore_ascii_case("resume") {
        return handle_ai_resume_subcommand(parts);
    }

    let Some(intent) = AiIntent::from_subcommand(raw_subcommand) else {
        println!(
            "{} {}",
            "✗ Unknown /ai subcommand:".color(Color::RED).bold(),
            raw_subcommand.color(Color::YELLOW)
        );
        println!(
            "{}",
            "Valid subcommands: ask, plan, explain, resume, apply".color(Color::YELLOW)
        );
        return ExitStatus::Error;
    };

    let dry_run = has_flag(parts, "--dry-run");
    let explicit_write_approval = has_flag(parts, "--approve-write");

    let prompt = collect_ai_prompt(parts, ai_prompt_start(parts));
    if prompt.is_empty() {
        let usage = if matches!(intent, AiIntent::ApplyDryRun) {
            "Usage: /ai apply --dry-run <instruction> | /ai apply --approve-write <instruction>"
                .to_string()
        } else {
            format!("Usage: /ai {} <prompt>", intent.as_label())
        };

        println!(
            "{}",
            format!("✗ Missing prompt for /ai {}.", intent.as_label())
                .color(Color::RED)
                .bold()
        );
        println!("{}", usage.color(Color::YELLOW));
        return ExitStatus::Error;
    }

    let ai_metrics = runtime_metrics().clone();
    ai_metrics.increment_counter("runtime_ai_requests_total");
    ai_metrics.increment_counter(format!("runtime_ai_intent_{}_total", intent.as_label()));
    if let Err(retry_after_seconds) = enforce_ai_rate_limit(&ai_metrics) {
        ai_metrics.increment_counter("runtime_ai_requests_error_total");
        update_ai_request_rate_gauges(&ai_metrics);
        println!(
            "{}",
            format!(
                "✗ AI rate limit reached. Try again in ~{retry_after_seconds}s, or adjust NTK_AI_RATE_LIMIT_REQUESTS/NTK_AI_RATE_LIMIT_WINDOW_SECONDS."
            )
            .color(Color::RED)
            .bold()
        );
        return ExitStatus::Error;
    }

    if matches!(intent, AiIntent::ApplyDryRun) {
        let mut reason_preview = prompt.chars().take(96).collect::<String>();
        if prompt.chars().count() > 96 {
            reason_preview.push_str("...");
        }

        let decision = request_approval(ApprovalRequest::file_write(
            "workspace",
            format!("ai apply request: {reason_preview}"),
            dry_run,
            explicit_write_approval,
            "cli:/ai apply",
        ));

        match decision {
            ApprovalDecision::Approved { reason } => {
                update_ai_approval_metrics(&ai_metrics, true);
                let _ = nettoolskit_ui::append_footer_log(&format!(
                    "ai approval: action=file_write status=approved reason={reason}"
                ));
                if !dry_run {
                    println!(
                        "{}",
                        "⚠ Explicit write approval acknowledged; current /ai apply flow remains advisory and does not mutate files directly."
                            .color(Color::YELLOW)
                    );
                }
            }
            ApprovalDecision::Denied { reason } => {
                update_ai_approval_metrics(&ai_metrics, false);
                println!(
                    "{} {}",
                    "✗ AI apply blocked by approval gateway:"
                        .color(Color::RED)
                        .bold(),
                    reason.color(Color::RED)
                );
                println!(
                    "{}",
                    "Use `--dry-run` (recommended) or `--approve-write` for explicit mutating approval."
                        .color(Color::YELLOW)
                );
                let _ = nettoolskit_ui::append_footer_log(&format!(
                    "ai approval: action=file_write status=denied reason={reason}"
                ));
                ai_metrics.increment_counter("runtime_ai_requests_error_total");
                update_ai_request_rate_gauges(&ai_metrics);
                return ExitStatus::Error;
            }
        }
    }

    let provider = match ai_provider_from_env(intent, &prompt) {
        Ok(provider) => provider,
        Err(error) => {
            ai_metrics.increment_counter("runtime_ai_requests_error_total");
            ai_metrics.set_gauge("runtime_ai_provider_health", 0.0);
            update_ai_request_rate_gauges(&ai_metrics);
            println!(
                "{} {}",
                "✗ Failed to initialize AI provider:"
                    .color(Color::RED)
                    .bold(),
                error.color(Color::RED)
            );
            return ExitStatus::Error;
        }
    };

    let session_id = resolve_active_ai_session_id();
    let active_session = load_or_initialize_ai_session(&session_id);
    let mut request = build_ai_request(intent, &prompt);
    inject_ai_session_context(&mut request, &active_session);
    if let Some(context_message) = build_ai_context_system_message() {
        request
            .messages
            .insert(1, AiMessage::new(AiRole::System, context_message));
        let _ = nettoolskit_ui::append_footer_log("ai: context bundle attached");
    }

    println!(
        "{}",
        format!("🤖 AI {} (provider: {})", intent.as_label(), provider.id())
            .color(Color::CYAN)
            .bold()
    );

    let provider_id = provider.id();
    let provider_metric = sanitize_metric_component(provider_id);
    ai_metrics.increment_counter(format!(
        "runtime_ai_provider_{}_requests_total",
        provider_metric
    ));
    let retry_policy = ai_retry_policy_from_env();
    let request_started = Instant::now();

    let _ = nettoolskit_ui::append_footer_log(&format!(
        "ai: mode={} provider={} stream=start",
        intent.as_label(),
        provider_id
    ));
    let _ = nettoolskit_ui::append_footer_log(&format!("ai: active_session={session_id}"));

    match request_ai_stream_with_retry(
        provider.as_ref(),
        &request,
        retry_policy,
        &ai_metrics,
        intent,
    )
    .await
    {
        Ok((chunks, retries)) => {
            ai_metrics.record_timing("runtime_ai_request_latency", request_started.elapsed());
            ai_metrics.set_gauge("runtime_ai_last_retry_count", retries as f64);
            if chunks.is_empty() {
                ai_metrics.increment_counter("runtime_ai_requests_error_total");
                ai_metrics.increment_counter("runtime_ai_requests_empty_stream_total");
                set_ai_provider_health(&ai_metrics, provider_id, false);
                update_ai_request_rate_gauges(&ai_metrics);
                println!(
                    "{}",
                    "✗ AI provider returned an empty stream"
                        .color(Color::RED)
                        .bold()
                );
                let _ = nettoolskit_ui::append_footer_log("ai: empty stream");
                return ExitStatus::Error;
            }

            let mut output = String::new();
            for chunk in chunks {
                if !chunk.content.is_empty() {
                    print!("{}", chunk.content);
                    let _ = io::stdout().flush();
                    output.push_str(&chunk.content);
                    let _ = nettoolskit_ui::append_footer_log(&format!(
                        "ai: stream chunk ({} chars)",
                        chunk.content.chars().count()
                    ));
                }
                if chunk.done {
                    break;
                }
            }

            println!();
            if output.trim().is_empty() {
                ai_metrics.increment_counter("runtime_ai_requests_error_total");
                ai_metrics.increment_counter("runtime_ai_requests_empty_output_total");
                set_ai_provider_health(&ai_metrics, provider_id, false);
                update_ai_request_rate_gauges(&ai_metrics);
                println!(
                    "{}",
                    "✗ AI provider returned empty output content"
                        .color(Color::RED)
                        .bold()
                );
                let _ = nettoolskit_ui::append_footer_log("ai: empty output content");
                return ExitStatus::Error;
            }

            persist_ai_session_exchange(&session_id, intent, provider_id, &prompt, &output);
            record_ai_usage_estimates(&ai_metrics, &request, &output);
            ai_metrics.increment_counter("runtime_ai_requests_success_total");
            ai_metrics.increment_counter(format!(
                "runtime_ai_provider_{}_success_total",
                provider_metric
            ));
            set_ai_provider_health(&ai_metrics, provider_id, true);
            update_ai_request_rate_gauges(&ai_metrics);
            let _ = nettoolskit_ui::append_footer_log("ai: stream completed");
            ExitStatus::Success
        }
        Err(error) => {
            ai_metrics.increment_counter("runtime_ai_requests_error_total");
            ai_metrics.increment_counter(format!(
                "runtime_ai_provider_{}_error_total",
                provider_metric
            ));
            ai_metrics.record_timing("runtime_ai_request_latency", request_started.elapsed());
            if matches!(error, AiProviderError::Timeout { .. }) {
                ai_metrics.increment_counter("runtime_ai_requests_timeout_total");
            }
            set_ai_provider_health(&ai_metrics, provider_id, false);
            update_ai_request_rate_gauges(&ai_metrics);
            println!(
                "{} {}",
                "✗ AI request failed:".color(Color::RED).bold(),
                error.to_string().color(Color::RED)
            );
            if let Some(guidance) = ai_error_guidance_message(&error) {
                println!("{}", guidance.color(Color::YELLOW));
            }
            let _ = nettoolskit_ui::append_footer_log(&format!("ai: request failed - {error}"));
            ExitStatus::Error
        }
    }
}

fn current_unix_timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or(0)
}

fn next_task_id() -> String {
    let sequence = TASK_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    format!("task-{}-{sequence:08x}", current_unix_timestamp_ms())
}

fn task_intent_kind_label(kind: TaskIntentKind) -> &'static str {
    match kind {
        TaskIntentKind::CommandExecution => "command",
        TaskIntentKind::AiAsk => "ai-ask",
        TaskIntentKind::AiPlan => "ai-plan",
        TaskIntentKind::AiExplain => "ai-explain",
        TaskIntentKind::AiApplyDryRun => "ai-apply-dry-run",
    }
}

fn parse_task_intent_kind(value: &str) -> Option<TaskIntentKind> {
    match value.trim().to_ascii_lowercase().as_str() {
        "command" | "cmd" => Some(TaskIntentKind::CommandExecution),
        "ai-ask" | "ask" => Some(TaskIntentKind::AiAsk),
        "ai-plan" | "plan" => Some(TaskIntentKind::AiPlan),
        "ai-explain" | "explain" => Some(TaskIntentKind::AiExplain),
        "ai-apply-dry-run" | "apply-dry-run" | "apply" => Some(TaskIntentKind::AiApplyDryRun),
        _ => None,
    }
}

fn task_status_label(status: TaskExecutionStatus) -> &'static str {
    match status {
        TaskExecutionStatus::Queued => "queued",
        TaskExecutionStatus::Running => "running",
        TaskExecutionStatus::Succeeded => "succeeded",
        TaskExecutionStatus::Failed => "failed",
        TaskExecutionStatus::Cancelled => "cancelled",
    }
}

fn task_service_endpoint_from_env() -> Option<String> {
    std::env::var("NTK_TASK_SERVICE_ENDPOINT")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn resolve_task_execution_target(runtime_mode: RuntimeMode) -> (String, Option<String>) {
    match runtime_mode {
        RuntimeMode::Service => match task_service_endpoint_from_env() {
            Some(endpoint) => (
                "background-worker-local".to_string(),
                Some(format!(
                    "Background service endpoint is configured ({endpoint}) but remote dispatch is not enabled yet; using embedded background worker."
                )),
            ),
            None => (
                "background-worker-local".to_string(),
                Some(
                    "Runtime mode is set to service without remote endpoint; using embedded background worker."
                        .to_string(),
                ),
            ),
        },
        RuntimeMode::Cli => (
            "local-fallback".to_string(),
            task_service_endpoint_from_env().map(|endpoint| {
                format!(
                    "CLI runtime keeps execution local; service endpoint ({endpoint}) ignored for now."
                )
            }),
        ),
    }
}

fn task_worker_policy_from_env() -> TaskWorkerPolicy {
    let mut policy = TaskWorkerPolicy::default();

    if let Ok(value) = std::env::var("NTK_TASK_QUEUE_CAPACITY") {
        if let Some(parsed) = parse_nonzero_usize(&value) {
            policy.queue_capacity = parsed;
        }
    }

    if let Ok(value) = std::env::var("NTK_TASK_MAX_CONCURRENCY") {
        if let Some(parsed) = parse_nonzero_usize(&value) {
            policy.max_concurrency = parsed;
        }
    }

    if let Ok(value) = std::env::var("NTK_TASK_MAX_RETRIES") {
        if let Ok(parsed) = value.trim().parse::<usize>() {
            policy.max_retries = parsed;
        }
    }

    if let Ok(value) = std::env::var("NTK_TASK_RETRY_BASE_MS") {
        if let Some(parsed) = parse_timeout_millis(&value) {
            policy.retry_base_delay = Duration::from_millis(parsed);
        }
    }

    if let Ok(value) = std::env::var("NTK_TASK_RETRY_MAX_MS") {
        if let Some(parsed) = parse_timeout_millis(&value) {
            policy.retry_max_delay = Duration::from_millis(parsed);
        }
    }

    if policy.retry_max_delay < policy.retry_base_delay {
        policy.retry_max_delay = policy.retry_base_delay;
    }

    policy
}

fn task_retry_delay(policy: TaskWorkerPolicy, attempt: usize) -> Duration {
    let exponent = attempt.saturating_sub(1) as u32;
    let factor = 2u128.saturating_pow(exponent);
    let base_ms = policy.retry_base_delay.as_millis();
    let max_ms = policy.retry_max_delay.as_millis();
    let delay_ms = (base_ms.saturating_mul(factor)).min(max_ms) as u64;
    Duration::from_millis(delay_ms)
}

fn task_worker_sender() -> mpsc::Sender<QueuedTask> {
    TASK_WORKER_SENDER
        .get_or_init(|| {
            let policy = task_worker_policy_from_env();
            let (sender, receiver) = mpsc::channel(policy.queue_capacity);
            start_task_worker_dispatcher(receiver, policy);
            sender
        })
        .clone()
}

fn start_task_worker_dispatcher(
    mut receiver: mpsc::Receiver<QueuedTask>,
    policy: TaskWorkerPolicy,
) {
    tokio::spawn(async move {
        let semaphore = Arc::new(Semaphore::new(policy.max_concurrency));

        while let Some(task) = receiver.recv().await {
            let permit = match semaphore.clone().acquire_owned().await {
                Ok(permit) => permit,
                Err(_) => {
                    warn!("task worker semaphore closed; stopping dispatcher");
                    break;
                }
            };
            let policy_for_task = policy;
            tokio::spawn(async move {
                let _permit = permit;
                run_task_in_background(task, policy_for_task).await;
            });
        }
    });
}

fn submit_task_to_worker(
    task_id: String,
    intent: TaskIntent,
    runtime_mode: RuntimeMode,
) -> Result<(), String> {
    match task_worker_sender().try_send(QueuedTask {
        id: task_id,
        intent,
        runtime_mode,
    }) {
        Ok(()) => Ok(()),
        Err(TrySendError::Full(_)) => Err(
            "Task queue is at capacity. Retry later or increase NTK_TASK_QUEUE_CAPACITY."
                .to_string(),
        ),
        Err(TrySendError::Closed(_)) => Err(
            "Background worker queue is unavailable. Restart the session and retry.".to_string(),
        ),
    }
}

fn update_task_attempt(task_id: &str, attempts: usize) -> Option<TaskRecord> {
    let updated = with_task_registry(|registry| {
        let record = registry.get_mut(task_id)?;
        record.attempts = attempts;
        record.updated_at_unix_ms = current_unix_timestamp_ms();
        Some(record.clone())
    });
    if let Some(record) = &updated {
        append_task_audit_event(
            &record.id,
            record.runtime_mode,
            record.status,
            format!("Attempt set to {}/{}", record.attempts, record.max_attempts),
        );
    }
    updated
}

fn is_task_cancelled(task_id: &str) -> bool {
    with_task_registry(|registry| {
        registry
            .get(task_id)
            .map(|record| record.status == TaskExecutionStatus::Cancelled)
            .unwrap_or(false)
    })
}

async fn run_task_in_background(task: QueuedTask, policy: TaskWorkerPolicy) {
    let max_attempts = policy.max_retries.saturating_add(1);

    for attempt in 1..=max_attempts {
        if is_task_cancelled(&task.id) {
            let _ = update_task_record_status(
                &task.id,
                TaskExecutionStatus::Cancelled,
                "Task was cancelled before worker execution",
            );
            return;
        }

        let _ = update_task_attempt(&task.id, attempt);
        let _ = update_task_record_status(
            &task.id,
            TaskExecutionStatus::Running,
            format!(
                "Background worker ({}) attempt {attempt}/{max_attempts} in progress",
                task.runtime_mode
            ),
        );

        let (status, detail) = execute_task_locally(&task.intent).await;
        if is_task_cancelled(&task.id) {
            let _ = update_task_record_status(
                &task.id,
                TaskExecutionStatus::Cancelled,
                "Cancellation requested while task was running",
            );
            return;
        }

        if status == TaskExecutionStatus::Failed && attempt < max_attempts {
            let delay = task_retry_delay(policy, attempt);
            let _ = update_task_record_status(
                &task.id,
                TaskExecutionStatus::Running,
                format!(
                    "Attempt {attempt}/{max_attempts} failed; retrying in {}ms ({detail})",
                    delay.as_millis()
                ),
            );
            tokio::time::sleep(delay).await;
            continue;
        }

        let _ = update_task_record_status(
            &task.id,
            status,
            format!("{detail} (attempt {attempt}/{max_attempts})"),
        );
        return;
    }
}

fn update_task_record_status(
    task_id: &str,
    status: TaskExecutionStatus,
    message: impl Into<String>,
) -> Option<TaskRecord> {
    let message = message.into();
    let updated = with_task_registry(|registry| {
        let record = registry.get_mut(task_id)?;
        if !record.status.can_transition_to(status) {
            warn!(
                task_id,
                from = task_status_label(record.status),
                to = task_status_label(status),
                "invalid task status transition ignored"
            );
            return Some(record.clone());
        }
        record.status = status;
        record.status_message = message;
        record.updated_at_unix_ms = current_unix_timestamp_ms();
        Some(record.clone())
    });

    if let Some(record) = &updated {
        append_task_audit_event(
            &record.id,
            record.runtime_mode,
            record.status,
            record.status_message.clone(),
        );
    }
    updated
}

fn set_task_execution_target(task_id: &str, execution_target: String) -> Option<TaskRecord> {
    let updated = with_task_registry(|registry| {
        let record = registry.get_mut(task_id)?;
        record.execution_target = execution_target;
        record.updated_at_unix_ms = current_unix_timestamp_ms();
        Some(record.clone())
    });
    if let Some(record) = &updated {
        append_task_audit_event(
            &record.id,
            record.runtime_mode,
            record.status,
            format!("Execution target set to {}", record.execution_target),
        );
    }
    updated
}

fn task_status_to_exit_status(status: TaskExecutionStatus) -> ExitStatus {
    match status {
        TaskExecutionStatus::Queued
        | TaskExecutionStatus::Running
        | TaskExecutionStatus::Succeeded => ExitStatus::Success,
        TaskExecutionStatus::Cancelled => ExitStatus::Interrupted,
        TaskExecutionStatus::Failed => ExitStatus::Error,
    }
}

async fn execute_task_locally(intent: &TaskIntent) -> (TaskExecutionStatus, String) {
    let payload = intent.payload.trim().to_string();
    match intent.kind {
        TaskIntentKind::AiAsk => {
            let owned_parts = ["/ai".to_string(), "ask".to_string(), payload];
            let refs = owned_parts.iter().map(String::as_str).collect::<Vec<_>>();
            let status = process_ai_command(&refs, Some("ask")).await;
            let outcome = match status {
                ExitStatus::Success => TaskExecutionStatus::Succeeded,
                ExitStatus::Interrupted => TaskExecutionStatus::Cancelled,
                ExitStatus::Error => TaskExecutionStatus::Failed,
            };
            (
                outcome,
                format!("Delegated to `/ai ask` (status: {status:?})"),
            )
        }
        TaskIntentKind::AiPlan => {
            let owned_parts = ["/ai".to_string(), "plan".to_string(), payload];
            let refs = owned_parts.iter().map(String::as_str).collect::<Vec<_>>();
            let status = process_ai_command(&refs, Some("plan")).await;
            let outcome = match status {
                ExitStatus::Success => TaskExecutionStatus::Succeeded,
                ExitStatus::Interrupted => TaskExecutionStatus::Cancelled,
                ExitStatus::Error => TaskExecutionStatus::Failed,
            };
            (
                outcome,
                format!("Delegated to `/ai plan` (status: {status:?})"),
            )
        }
        TaskIntentKind::AiExplain => {
            let owned_parts = ["/ai".to_string(), "explain".to_string(), payload];
            let refs = owned_parts.iter().map(String::as_str).collect::<Vec<_>>();
            let status = process_ai_command(&refs, Some("explain")).await;
            let outcome = match status {
                ExitStatus::Success => TaskExecutionStatus::Succeeded,
                ExitStatus::Interrupted => TaskExecutionStatus::Cancelled,
                ExitStatus::Error => TaskExecutionStatus::Failed,
            };
            (
                outcome,
                format!("Delegated to `/ai explain` (status: {status:?})"),
            )
        }
        TaskIntentKind::AiApplyDryRun => {
            let owned_parts = [
                "/ai".to_string(),
                "apply".to_string(),
                "--dry-run".to_string(),
                payload,
            ];
            let refs = owned_parts.iter().map(String::as_str).collect::<Vec<_>>();
            let status = process_ai_command(&refs, Some("apply")).await;
            let outcome = match status {
                ExitStatus::Success => TaskExecutionStatus::Succeeded,
                ExitStatus::Interrupted => TaskExecutionStatus::Cancelled,
                ExitStatus::Error => TaskExecutionStatus::Failed,
            };
            (
                outcome,
                format!("Delegated to `/ai apply --dry-run` (status: {status:?})"),
            )
        }
        TaskIntentKind::CommandExecution => {
            if payload.trim_start().starts_with("/task") || payload.starts_with("task ") {
                return (
                    TaskExecutionStatus::Failed,
                    "Nested `/task` command execution is not allowed".to_string(),
                );
            }

            (
                TaskExecutionStatus::Failed,
                "Local fallback for `command` intent is not enabled yet; use ai-* intents or run the command directly."
                    .to_string(),
            )
        }
    }
}

fn print_task_usage() {
    use nettoolskit_ui::Color;
    println!("{}", "🗂️ Task Manager Commands".color(Color::CYAN).bold());
    println!(
        "  {}",
        "/task submit <intent> <payload>".color(Color::GREEN)
    );
    println!("  {}", "/task list".color(Color::GREEN));
    println!("  {}", "/task watch <task-id>".color(Color::GREEN));
    println!("  {}", "/task cancel <task-id>".color(Color::GREEN));
    println!();
    println!("{}", "Supported intents:".color(Color::WHITE).bold());
    println!("  {}", "command".color(Color::CYAN));
    println!("  {}", "ai-ask".color(Color::CYAN));
    println!("  {}", "ai-plan".color(Color::CYAN));
    println!("  {}", "ai-explain".color(Color::CYAN));
    println!("  {}", "ai-apply-dry-run".color(Color::CYAN));
}

fn print_task_list(records: &[TaskRecord]) {
    use nettoolskit_ui::Color;
    if records.is_empty() {
        println!("{}", "No task records found.".color(Color::YELLOW));
        return;
    }

    println!("{}", "🗂️ Local Task Records".color(Color::CYAN).bold());
    for record in records {
        println!(
            "  {} | {} | {} | mode:{} | target:{} | attempts:{}/{}",
            record.id.color(Color::WHITE),
            task_status_label(record.status).color(Color::GREEN),
            task_intent_kind_label(record.intent.kind).color(Color::CYAN),
            record.runtime_mode.to_string().color(Color::WHITE),
            record.execution_target.color(Color::YELLOW),
            record.attempts,
            record.max_attempts,
        );
    }
}

async fn handle_task_submit(parts: &[&str]) -> ExitStatus {
    use nettoolskit_ui::Color;

    if parts.len() < 4 {
        println!(
            "{}",
            "Usage: /task submit <intent> <payload>".color(Color::YELLOW)
        );
        return ExitStatus::Error;
    }

    let Some(intent_kind) = parse_task_intent_kind(parts[2]) else {
        println!(
            "{} {}",
            "✗ Unsupported task intent:".color(Color::RED).bold(),
            parts[2].color(Color::YELLOW)
        );
        println!(
            "{}",
            "Supported intents: command, ai-ask, ai-plan, ai-explain, ai-apply-dry-run"
                .color(Color::YELLOW)
        );
        return ExitStatus::Error;
    };

    let payload = parts[3..].join(" ");
    if payload.trim().is_empty() {
        println!(
            "{}",
            "✗ Task payload cannot be empty.".color(Color::RED).bold()
        );
        return ExitStatus::Error;
    }

    let runtime_mode = AppConfig::load().general.runtime_mode;
    let worker_policy = task_worker_policy_from_env();
    let max_attempts = match runtime_mode {
        RuntimeMode::Service => worker_policy.max_retries.saturating_add(1),
        RuntimeMode::Cli => 1,
    };
    let task_id = next_task_id();
    let now = current_unix_timestamp_ms();
    let title = format!("{} task", task_intent_kind_label(intent_kind));
    let intent = TaskIntent::new(intent_kind, title, payload);
    let record = TaskRecord::new(
        task_id.clone(),
        intent.clone(),
        runtime_mode,
        max_attempts,
        now,
    );

    with_task_registry(|registry| {
        registry.insert(task_id.clone(), record);
    });
    append_task_audit_event(
        &task_id,
        runtime_mode,
        TaskExecutionStatus::Queued,
        format!(
            "Task submitted (intent: {}, mode: {})",
            task_intent_kind_label(intent_kind),
            runtime_mode
        ),
    );

    let (execution_target, fallback_reason) = resolve_task_execution_target(runtime_mode);
    let _ = set_task_execution_target(&task_id, execution_target.clone());
    if let Some(reason) = fallback_reason {
        println!(
            "{} {}",
            "⚠".color(Color::YELLOW),
            reason.color(Color::YELLOW)
        );
        let _ = nettoolskit_ui::append_footer_log(&format!("task: {reason}"));
    }

    let intent_fallback = intent.clone();
    let final_record = match runtime_mode {
        RuntimeMode::Service => {
            let submit_result = submit_task_to_worker(task_id.clone(), intent, runtime_mode);
            match submit_result {
                Ok(()) => with_task_registry(|registry| registry.get(&task_id).cloned())
                    .unwrap_or_else(|| {
                        let mut missing = TaskRecord::new(
                            task_id.clone(),
                            intent_fallback.clone(),
                            runtime_mode,
                            max_attempts,
                            now,
                        );
                        missing.execution_target = execution_target.clone();
                        missing.status_message =
                            "Task queued for background worker execution".to_string();
                        missing
                    }),
                Err(error) => {
                    let failed =
                        update_task_record_status(&task_id, TaskExecutionStatus::Failed, &error)
                            .unwrap_or_else(|| {
                                let mut missing = TaskRecord::new(
                                    task_id.clone(),
                                    intent_fallback.clone(),
                                    runtime_mode,
                                    max_attempts,
                                    now,
                                );
                                missing.status = TaskExecutionStatus::Failed;
                                missing.execution_target = execution_target.clone();
                                missing.status_message = error.clone();
                                missing
                            });
                    println!(
                        "{} {}",
                        "✗".color(Color::RED).bold(),
                        error.color(Color::RED)
                    );
                    return task_status_to_exit_status(failed.status);
                }
            }
        }
        RuntimeMode::Cli => {
            let _ = update_task_record_status(
                &task_id,
                TaskExecutionStatus::Running,
                format!(
                    "Executing intent {} locally",
                    task_intent_kind_label(intent_kind)
                ),
            );
            let _ = update_task_attempt(&task_id, 1);
            let (final_status, status_message) = execute_task_locally(&intent).await;
            update_task_record_status(&task_id, final_status, status_message).unwrap_or_else(|| {
                let mut missing = TaskRecord::new(
                    task_id.clone(),
                    intent_fallback,
                    runtime_mode,
                    max_attempts,
                    now,
                );
                missing.status = final_status;
                missing.attempts = 1;
                missing.execution_target = execution_target;
                missing
            })
        }
    };

    println!("{}", "✅ Task submitted".color(Color::GREEN).bold());
    println!("  id: {}", final_record.id.color(Color::CYAN));
    println!(
        "  intent: {}",
        task_intent_kind_label(final_record.intent.kind).color(Color::WHITE)
    );
    println!(
        "  status: {}",
        task_status_label(final_record.status).color(Color::WHITE)
    );
    println!(
        "  target: {}",
        final_record.execution_target.color(Color::WHITE)
    );
    println!(
        "  attempts: {}/{}",
        final_record.attempts, final_record.max_attempts
    );
    println!(
        "  detail: {}",
        final_record.status_message.color(Color::WHITE)
    );

    task_status_to_exit_status(final_record.status)
}

fn handle_task_list() -> ExitStatus {
    let mut records: Vec<TaskRecord> =
        with_task_registry(|registry| registry.values().cloned().collect());
    records.sort_by(|left, right| right.updated_at_unix_ms.cmp(&left.updated_at_unix_ms));
    print_task_list(&records);
    ExitStatus::Success
}

fn handle_task_watch(parts: &[&str]) -> ExitStatus {
    use nettoolskit_ui::Color;

    if parts.len() < 3 {
        println!("{}", "Usage: /task watch <task-id>".color(Color::YELLOW));
        return ExitStatus::Error;
    }

    let task_id = parts[2].trim();
    let record = with_task_registry(|registry| registry.get(task_id).cloned());
    let Some(record) = record else {
        println!(
            "{} {}",
            "✗ Task not found:".color(Color::RED).bold(),
            task_id.color(Color::YELLOW)
        );
        return ExitStatus::Error;
    };

    println!("{}", "🔎 Task Details".color(Color::CYAN).bold());
    println!("  id: {}", record.id.color(Color::CYAN));
    println!(
        "  intent: {}",
        task_intent_kind_label(record.intent.kind).color(Color::WHITE)
    );
    println!("  payload: {}", record.intent.payload.color(Color::WHITE));
    println!(
        "  status: {}",
        task_status_label(record.status).color(Color::WHITE)
    );
    println!(
        "  mode: {}",
        record.runtime_mode.to_string().color(Color::WHITE)
    );
    println!("  target: {}", record.execution_target.color(Color::WHITE));
    println!("  attempts: {}/{}", record.attempts, record.max_attempts);
    println!("  detail: {}", record.status_message.color(Color::WHITE));
    println!("  created_at_ms: {}", record.created_at_unix_ms);
    println!("  updated_at_ms: {}", record.updated_at_unix_ms);
    let audits = list_task_audit_events(task_id);
    if !audits.is_empty() {
        println!("  audit_events: {}", audits.len());
        for event in audits.iter().rev().take(5).rev() {
            println!(
                "    - [{}] {}",
                task_status_label(event.status),
                event.message
            );
        }
    }

    ExitStatus::Success
}

fn handle_task_cancel(parts: &[&str]) -> ExitStatus {
    use nettoolskit_ui::Color;

    if parts.len() < 3 {
        println!("{}", "Usage: /task cancel <task-id>".color(Color::YELLOW));
        return ExitStatus::Error;
    }

    let task_id = parts[2].trim().to_string();
    let current = with_task_registry(|registry| registry.get(&task_id).cloned());
    let cancelled = match current {
        Some(record) => {
            if !matches!(
                record.status,
                TaskExecutionStatus::Queued | TaskExecutionStatus::Running
            ) {
                Err(format!(
                    "Task is already terminal (status: {})",
                    task_status_label(record.status)
                ))
            } else {
                update_task_record_status(
                    &task_id,
                    TaskExecutionStatus::Cancelled,
                    "Cancelled by user request",
                )
                .ok_or_else(|| format!("Task not found: {task_id}"))
            }
        }
        None => Err(format!("Task not found: {task_id}")),
    };

    match cancelled {
        Ok(record) => {
            println!(
                "{} {}",
                "✅ Task cancelled:".color(Color::GREEN).bold(),
                record.id.color(Color::CYAN)
            );
            ExitStatus::Success
        }
        Err(error) => {
            println!(
                "{} {}",
                "✗".color(Color::RED).bold(),
                error.color(Color::RED)
            );
            ExitStatus::Error
        }
    }
}

async fn process_task_command(parts: &[&str]) -> ExitStatus {
    let Some(subcommand) = parts.get(1).copied() else {
        print_task_usage();
        return ExitStatus::Success;
    };

    match subcommand.trim().to_ascii_lowercase().as_str() {
        "help" => {
            print_task_usage();
            ExitStatus::Success
        }
        "submit" => handle_task_submit(parts).await,
        "list" => handle_task_list(),
        "watch" => handle_task_watch(parts),
        "cancel" => handle_task_cancel(parts),
        _ => {
            use nettoolskit_ui::Color;
            println!(
                "{} {}",
                "✗ Unknown /task subcommand:".color(Color::RED).bold(),
                subcommand.color(Color::YELLOW)
            );
            print_task_usage();
            ExitStatus::Error
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AsyncManifestAlias {
    New,
    Render,
    Apply,
}

fn detect_async_manifest_alias(
    parsed: Option<MainAction>,
    parts: &[&str],
    subcommand: Option<&str>,
) -> Option<AsyncManifestAlias> {
    if matches!(parsed, Some(MainAction::Manifest)) {
        return match subcommand {
            Some("new-async") => Some(AsyncManifestAlias::New),
            Some("render-async") => Some(AsyncManifestAlias::Render),
            Some("apply-async") => Some(AsyncManifestAlias::Apply),
            _ => None,
        };
    }

    let top_level = parts
        .first()
        .copied()
        .unwrap_or_default()
        .trim()
        .trim_start_matches('/')
        .to_ascii_lowercase();

    match top_level.as_str() {
        "new-async" => Some(AsyncManifestAlias::New),
        "render-async" => Some(AsyncManifestAlias::Render),
        "apply-async" => Some(AsyncManifestAlias::Apply),
        _ => None,
    }
}

fn async_alias_arg_start(parts: &[&str]) -> usize {
    if parts.first().copied() == Some("/") {
        if parts.get(1).copied() == Some("manifest") {
            return 3;
        }
        return 2;
    }

    if matches!(parts.first().copied(), Some("/manifest" | "manifest")) {
        2
    } else {
        1
    }
}

fn encode_exit_status(status: ExitStatus) -> &'static str {
    match status {
        ExitStatus::Success => "success",
        ExitStatus::Error => "error",
        ExitStatus::Interrupted => "interrupted",
    }
}

fn decode_exit_status(token: &str) -> ExitStatus {
    match token {
        "success" => ExitStatus::Success,
        "interrupted" => ExitStatus::Interrupted,
        _ => ExitStatus::Error,
    }
}

fn format_async_progress(operation: &str, progress: &CommandProgress) -> String {
    let mut segments = vec![format!("{operation}: {}", progress.message)];

    if let Some(percent) = progress.percent {
        segments.push(format!("{percent}%"));
    }

    if let (Some(completed), Some(total)) = (progress.completed, progress.total) {
        segments.push(format!("{completed}/{total}"));
    }

    segments.join(" | ")
}

fn interrupt_requested(interrupted: Option<&AtomicBool>) -> bool {
    interrupted.is_some_and(|flag| flag.load(Ordering::SeqCst))
}

async fn cancel_async_alias_execution(
    operation: &str,
    executor: &mut AsyncCommandExecutor,
) -> ExitStatus {
    let _ = nettoolskit_ui::append_footer_log(&format!(
        "{operation}: Ctrl+C detected, cancelling async operation"
    ));
    executor.cancel_all().await;
    let _ = nettoolskit_ui::append_footer_log(&format!("{operation}: async operation cancelled"));
    ExitStatus::Interrupted
}

async fn run_async_alias_with_progress<F, Fut>(
    operation: &str,
    interrupted: Option<&AtomicBool>,
    factory: F,
) -> ExitStatus
where
    F: FnOnce(ProgressSender) -> Fut + Send + 'static,
    Fut: Future<Output = ExitStatus> + Send + 'static,
{
    let mut executor = AsyncCommandExecutor::new();
    let (handle, mut progress_rx) = executor.spawn_with_progress(move |progress_tx| async move {
        let status = factory(progress_tx).await;
        Ok(encode_exit_status(status).to_string())
    });

    if interrupt_requested(interrupted) {
        return cancel_async_alias_execution(operation, &mut executor).await;
    }

    loop {
        tokio::select! {
            progress = progress_rx.recv() => {
                match progress {
                    Some(progress) => {
                        let _ = nettoolskit_ui::append_footer_log(&format_async_progress(operation, &progress));
                    }
                    None => break,
                }
            }
            _ = tokio::time::sleep(Duration::from_millis(25)), if interrupted.is_some() => {
                if interrupt_requested(interrupted) {
                    return cancel_async_alias_execution(operation, &mut executor).await;
                }
            }
        }
    }

    if interrupt_requested(interrupted) {
        return cancel_async_alias_execution(operation, &mut executor).await;
    }

    let status = match handle.wait().await {
        Ok(Ok(token)) => decode_exit_status(&token),
        Ok(Err(err)) => {
            let _ = nettoolskit_ui::append_footer_log(&format!(
                "{operation}: async worker error: {err}"
            ));
            ExitStatus::Error
        }
        Err(err) => {
            let _ = nettoolskit_ui::append_footer_log(&format!(
                "{operation}: async result channel failed: {err}"
            ));
            ExitStatus::Error
        }
    };

    executor.wait_all().await;
    status
}

async fn process_async_manifest_alias(
    alias: AsyncManifestAlias,
    parts: &[&str],
    interrupted: Option<&AtomicBool>,
) -> ExitStatus {
    use nettoolskit_ui::Color;

    let arg_start = async_alias_arg_start(parts);
    let output_override = parse_output_root(parts);
    let dry_run = has_flag(parts, "--dry-run");

    match alias {
        AsyncManifestAlias::Render => {
            let manifest_path = match resolve_manifest_target_path_from(parts, "render", arg_start)
            {
                Ok(path) => path,
                Err(status) => return status,
            };
            let output_root =
                output_override.unwrap_or_else(|| std::env::current_dir().unwrap_or_default());

            println!(
                "{}",
                "🎨 Rendering Preview (Async)".color(Color::CYAN).bold()
            );
            println!("Manifest: {}", manifest_path.display());
            println!("Output root: {}", output_root.display());
            println!(
                "{}",
                "DRY-RUN mode enabled (preview only)".color(Color::YELLOW)
            );
            println!();

            run_async_alias_with_progress(
                "render-async",
                interrupted,
                move |progress_tx| async move {
                    let _ = progress_tx
                        .send(CommandProgress::percent("Preparing manifest context", 15));
                    let _ = progress_tx.send(CommandProgress::steps("Render stages", 1, 3));

                    let config = nettoolskit_manifest::ExecutionConfig {
                        manifest_path,
                        output_root,
                        dry_run: true,
                    };

                    let _ = progress_tx.send(CommandProgress::percent("Rendering preview", 60));
                    let _ = progress_tx.send(CommandProgress::steps("Render stages", 2, 3));

                    let executor = nettoolskit_manifest::ManifestExecutor::new();
                    match executor.execute(config).await {
                        Ok(summary) => {
                            let _ = progress_tx.send(CommandProgress::percent("Completed", 100));
                            let _ = progress_tx.send(CommandProgress::steps("Render stages", 3, 3));
                            println!("{}", "✓ Render preview completed".color(Color::GREEN));
                            println!();
                            print_execution_summary(&summary);
                            ExitStatus::Success
                        }
                        Err(err) => {
                            let _ = progress_tx.send(CommandProgress::percent("Failed", 100));
                            println!(
                                "{} {err}",
                                "✗ Render preview failed:".color(Color::RED).bold()
                            );
                            ExitStatus::Error
                        }
                    }
                },
            )
            .await
        }
        AsyncManifestAlias::Apply | AsyncManifestAlias::New => {
            let Some(manifest_path) = first_positional_path(parts, arg_start) else {
                let command_name = if alias == AsyncManifestAlias::New {
                    "/new-async"
                } else {
                    "/apply-async"
                };
                println!(
                    "{} {}",
                    "✗ Missing manifest path for".color(Color::RED).bold(),
                    command_name.color(Color::CYAN)
                );
                println!(
                    "{}",
                    format!("Usage: {command_name} <manifest-file> [--dry-run] [--output <dir>]")
                        .color(Color::YELLOW)
                );
                return ExitStatus::Error;
            };

            let operation = if alias == AsyncManifestAlias::New {
                "new-async"
            } else {
                "apply-async"
            };

            run_async_alias_with_progress(operation, interrupted, move |progress_tx| async move {
                let _ = progress_tx.send(CommandProgress::percent("Preparing apply plan", 20));
                let _ = progress_tx.send(CommandProgress::steps("Apply stages", 1, 3));

                let _ = progress_tx.send(CommandProgress::percent("Applying manifest", 65));
                let _ = progress_tx.send(CommandProgress::steps("Apply stages", 2, 3));

                let status =
                    nettoolskit_manifest::execute_apply(manifest_path, output_override, dry_run)
                        .await;

                let completion = if matches!(status, ExitStatus::Success) {
                    "Completed"
                } else {
                    "Finished with errors"
                };
                let _ = progress_tx.send(CommandProgress::percent(completion, 100));
                let _ = progress_tx.send(CommandProgress::steps("Apply stages", 3, 3));
                status
            })
            .await
        }
    }
}

/// Process slash commands from CLI and return appropriate status
///
/// This function handles the mapping between CLI slash commands and the actual
/// command implementations, providing telemetry and logging for all operations.
/// It serves as the main dispatcher for interactive CLI commands.
///
/// # Arguments
///
/// * `cmd` - The slash command string (e.g., "/list", "/new", etc.)
///
/// # Returns
///
/// Returns `ExitStatus` indicating the result of command execution
pub async fn process_command(cmd: &str) -> ExitStatus {
    process_command_with_interrupt(cmd, None).await
}

/// Process slash commands from CLI and return appropriate status, with optional interrupt flag.
///
/// When `interrupted` is provided, long-running async aliases can react to Ctrl+C and
/// return [`ExitStatus::Interrupted`] deterministically.
pub async fn process_command_with_interrupt(
    cmd: &str,
    interrupted: Option<&AtomicBool>,
) -> ExitStatus {
    let correlation_id = next_correlation_id("cmd");
    let execution_span =
        info_span!("orchestrator.command", correlation_id = %correlation_id, command = %cmd);
    let _execution_scope = execution_span.enter();

    let metrics = runtime_metrics().clone();

    // Log command usage with structured data
    info!(
        correlation_id = %correlation_id,
        command = %cmd,
        command_type = %cmd.trim_start_matches('/'),
        "Processing CLI command"
    );
    metrics.increment_counter(format!("command_{}_usage", cmd.trim_start_matches('/')));

    // Parse command - pass full command string to get_main_action
    // It will handle "/ help", "/help", or "help" formats
    let parts: Vec<&str> = cmd.split_whitespace().collect();

    // If command is "/ help" (with space), parts = ["/", "help"], subcommand = parts[2]
    // If command is "/help list", parts = ["/help", "list"], subcommand = parts[1]
    let subcommand = if parts.first().copied() == Some("/") {
        parts.get(2).copied()
    } else {
        parts.get(1).copied()
    };

    // Parse command using full original string
    let parsed = crate::models::get_main_action(cmd);
    let command_key = command_metric_key(parsed, subcommand, cmd);
    let command_timing_name = format!("runtime_command_latency_{command_key}");
    let timer = Timer::start(command_timing_name.clone(), metrics.clone());

    metrics.increment_counter("runtime_commands_total");
    metrics.increment_counter(format!("runtime_command_{command_key}_total"));
    metrics.set_gauge(
        "runtime_command_plugins_enabled",
        command_plugin_count() as f64,
    );

    let before_hook_stats = run_before_command_plugins(&CommandHookContext {
        correlation_id: correlation_id.clone(),
        command: cmd.to_string(),
        command_key: command_key.clone(),
        status: None,
    });
    metrics.set_gauge(
        "runtime_command_plugins_last_before_invoked",
        before_hook_stats.invoked as f64,
    );
    metrics.set_gauge(
        "runtime_command_plugins_last_before_failures",
        before_hook_stats.failures as f64,
    );
    if before_hook_stats.failures > 0 {
        metrics.increment_counter("runtime_command_plugins_hook_errors_total");
    }

    let result = if let Some(alias) = detect_async_manifest_alias(parsed, &parts, subcommand) {
        process_async_manifest_alias(alias, &parts, interrupted).await
    } else {
        match parsed {
            Some(MainAction::Help) => {
                let help_key = CacheKey::help();
                let help_markdown = match with_command_cache(|cache| cache.get(&help_key)) {
                    Some(CacheValue::HelpMarkdown(markdown)) => {
                        metrics.increment_counter("runtime_command_cache_hits_total");
                        metrics.increment_counter("runtime_command_cache_help_hits_total");
                        markdown
                    }
                    _ => {
                        metrics.increment_counter("runtime_command_cache_misses_total");
                        metrics.increment_counter("runtime_command_cache_help_misses_total");

                        let mut commands_section = String::new();
                        for command in MainAction::iter() {
                            commands_section.push_str(&format!(
                                "- `{}` - {}\n",
                                command.slash_static(),
                                command.description()
                            ));
                        }

                        let generated = format!(
                            "\
# NetToolsKit CLI - Help

## Available Commands
{commands_section}
## Usage
- Type `/` to open the command palette
- Type a command directly (for example `/help`)
- Use `↑↓` to navigate in the palette
- Press `Enter` to select a command

## Examples
- `/help` - Show this help
- `/manifest` - Manage manifests
- `/render-async <manifest>` - Run async render preview with progress
- `/apply-async <manifest>` - Run async apply with progress
- `/new-async <manifest>` - Run async scaffolding alias
- `/config` - View or edit configuration
- `/ai ask <prompt>` - Ask the AI assistant
- `/ai plan <goal>` - Generate an implementation plan
- `/ai explain <topic>` - Get a technical explanation
- `/ai resume <session-id>` - Set active local AI session id for conversation continuity
- `/ai apply --dry-run <instruction>` - Generate non-destructive patch guidance
- `/ai apply --approve-write <instruction>` - Explicitly approve mutating apply intent
- `/task submit <intent> <payload>` - Submit a task for managed execution (local fallback)
- `/task list` - List local task records
- `/task watch <task-id>` - Inspect task status/details
- `/task cancel <task-id>` - Cancel queued/running task record
- `/clear` - Clear and redraw terminal layout
- `/quit` - Exit the CLI
"
                        );

                        let _ = with_command_cache(|cache| {
                            cache.insert(help_key, CacheValue::HelpMarkdown(generated.clone()))
                        });

                        generated
                    }
                };

                println!("{}", nettoolskit_ui::render_markdown(&help_markdown));

                ExitStatus::Success
            }
            Some(MainAction::Manifest) => {
                use nettoolskit_ui::Color;
                match subcommand {
                    Some("list") => {
                        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
                        println!("{}", "📋 Manifest Discovery".color(Color::CYAN).bold());
                        println!("Root: {}", cwd.display().to_string().color(Color::WHITE));
                        println!();

                        let cache_key = CacheKey::manifest_list(&cwd);
                        let discovery = match with_command_cache(|cache| cache.get(&cache_key)) {
                            Some(CacheValue::ManifestListEntries(entries)) => {
                                metrics.increment_counter("runtime_command_cache_hits_total");
                                metrics
                                    .increment_counter("runtime_command_cache_manifest_hits_total");
                                Ok(entries)
                            }
                            _ => {
                                metrics.increment_counter("runtime_command_cache_misses_total");
                                metrics.increment_counter(
                                    "runtime_command_cache_manifest_misses_total",
                                );
                                match discover_manifest_files(&cwd) {
                                    Ok(found) => {
                                        let _ = with_command_cache(|cache| {
                                            cache.insert(
                                                cache_key,
                                                CacheValue::ManifestListEntries(found.clone()),
                                            )
                                        });
                                        Ok(found)
                                    }
                                    Err(err) => Err(err),
                                }
                            }
                        };

                        match discovery {
                            Ok(found) if found.is_empty() => {
                                println!("{}", "No manifest files found.".color(Color::YELLOW));
                                ExitStatus::Error
                            }
                            Ok(found) => {
                                println!(
                                    "{}",
                                    format!("Found {} manifest file(s):", found.len())
                                        .color(Color::GREEN)
                                );
                                for path in found {
                                    println!("  - {}", relative_path_for_display(&cwd, &path));
                                }
                                ExitStatus::Success
                            }
                            Err(err) => {
                                println!(
                                    "{} {err}",
                                    "✗ Discovery failed:".color(Color::RED).bold()
                                );
                                ExitStatus::Error
                            }
                        }
                    }
                    Some("check") => {
                        let is_template = has_flag(&parts, "--template");
                        if is_template && first_manifest_positional_path(&parts).is_none() {
                            println!(
                            "{}",
                            "Template validation requires a file path: /manifest check <file> --template"
                                .color(Color::RED)
                                .bold()
                        );
                            ExitStatus::Error
                        } else {
                            match resolve_manifest_target_path(&parts, "check") {
                                Ok(target_path) => {
                                    println!(
                                        "{}",
                                        "✅ Validating Manifest".color(Color::CYAN).bold()
                                    );
                                    println!("Target: {}", target_path.display());
                                    println!();

                                    match nettoolskit_manifest::handlers::check::check_file(
                                        &target_path,
                                        is_template,
                                    )
                                    .await
                                    {
                                        Ok(validation) => {
                                            print_manifest_validation(&target_path, &validation);
                                            if validation.is_valid() {
                                                ExitStatus::Success
                                            } else {
                                                ExitStatus::Error
                                            }
                                        }
                                        Err(err) => {
                                            println!(
                                                "{} {err}",
                                                "✗ Manifest validation failed:"
                                                    .color(Color::RED)
                                                    .bold()
                                            );
                                            ExitStatus::Error
                                        }
                                    }
                                }
                                Err(status) => status,
                            }
                        }
                    }
                    Some("render") => match resolve_manifest_target_path(&parts, "render") {
                        Ok(manifest_path) => {
                            let output_root = parse_output_root(&parts).unwrap_or_else(|| {
                                std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
                            });
                            let dry_run = true;

                            println!("{}", "🎨 Rendering Preview".color(Color::CYAN).bold());
                            println!("Manifest: {}", manifest_path.display());
                            println!("Output root: {}", output_root.display());
                            println!(
                                "{}",
                                "DRY-RUN mode enabled (preview only)".color(Color::YELLOW)
                            );
                            println!();

                            let config = nettoolskit_manifest::ExecutionConfig {
                                manifest_path,
                                output_root,
                                dry_run,
                            };

                            let executor = nettoolskit_manifest::ManifestExecutor::new();
                            match executor.execute(config).await {
                                Ok(summary) => {
                                    println!(
                                        "{}",
                                        "✓ Render preview completed".color(Color::GREEN)
                                    );
                                    println!();
                                    print_execution_summary(&summary);
                                    ExitStatus::Success
                                }
                                Err(err) => {
                                    println!(
                                        "{} {err}",
                                        "✗ Render preview failed:".color(Color::RED).bold()
                                    );
                                    ExitStatus::Error
                                }
                            }
                        }
                        Err(status) => status,
                    },
                    Some("apply") => {
                        // Parse apply command arguments
                        // Format: /manifest apply <PATH> [--dry-run] [--output DIR]

                        let manifest_path = first_manifest_positional_path(&parts);
                        let dry_run = has_flag(&parts, "--dry-run");
                        let output_root = parse_output_root(&parts);

                        match manifest_path {
                            Some(path) => {
                                // Execute apply handler
                                nettoolskit_manifest::execute_apply(path, output_root, dry_run)
                                    .await
                            }
                            None => {
                                info!(
                                "Manifest apply called without path; opening interactive apply flow"
                            );
                                nettoolskit_manifest::show_apply_menu().await
                            }
                        }
                    }
                    None => {
                        // No subcommand provided - show interactive menu from manifest crate
                        info!("Opening manifest interactive menu (no subcommand)");
                        nettoolskit_manifest::show_menu().await
                    }
                    _ => {
                        println!("{}", "📋 Manifest Commands".color(Color::CYAN).bold());
                        println!("\nAvailable subcommands:");
                        println!(
                            "  {} - Discover available manifests in the workspace",
                            "/manifest list".color(Color::GREEN)
                        );
                        println!(
                            "  {} - Validate manifest structure and dependencies",
                            "/manifest check".color(Color::GREEN)
                        );
                        println!(
                            "  {} - Preview generated files without creating them",
                            "/manifest render".color(Color::GREEN)
                        );
                        println!(
                            "  {} - Async preview with progress updates",
                            "/manifest render-async".color(Color::GREEN)
                        );
                        println!(
                            "  {} - Apply manifest to generate/update project files",
                            "/manifest apply".color(Color::GREEN)
                        );
                        println!(
                            "  {} - Async apply with progress updates",
                            "/manifest apply-async".color(Color::GREEN)
                        );
                        println!("\n{}", "💡 Type a subcommand to continue or just type /manifest for interactive menu".color(Color::YELLOW));
                        ExitStatus::Success
                    }
                }
            }
            Some(MainAction::Translate) => {
                use nettoolskit_ui::Color;
                match parse_translate_request(&parts) {
                    Ok(request) => nettoolskit_translate::handle_translate(request).await,
                    Err(reason) => {
                        println!("{}", "🔄 Translate Command".color(Color::CYAN).bold());
                        println!();
                        println!(
                            "{} {reason}",
                            "✗ Invalid translate arguments:".color(Color::RED).bold()
                        );
                        println!();
                        println!("{}", "Usage:".color(Color::WHITE).bold());
                        println!(
                            "  {} --from <language> --to <language> <template-path>",
                            "/translate".color(Color::GREEN)
                        );
                        println!();
                        println!("{}", "Examples:".color(Color::WHITE).bold());
                        println!(
                            "  {} --from dotnet --to rust ./templates/entity.cs.hbs",
                            "/translate".color(Color::GREEN)
                        );
                        println!(
                            "  {} --from python --to java ./templates/service.py.hbs",
                            "/translate".color(Color::GREEN)
                        );
                        println!();
                        println!(
                        "{}",
                        "Supported languages: dotnet, java, go, python, rust, clojure, typescript"
                            .color(Color::YELLOW)
                    );
                        println!(
                            "{}",
                            "Note: clojure/typescript currently use baseline convention mapping."
                                .color(Color::YELLOW)
                        );
                        ExitStatus::Error
                    }
                }
            }
            Some(MainAction::Ai) => process_ai_command(&parts, subcommand).await,
            Some(MainAction::Task) => process_task_command(&parts).await,
            Some(MainAction::Config) => process_config_command(&parts),
            Some(MainAction::Clear) => match nettoolskit_ui::reset_layout() {
                Ok(()) => ExitStatus::Success,
                Err(err) => {
                    use nettoolskit_ui::Color;
                    println!(
                        "{}: {err}",
                        "Failed to reset terminal layout".color(Color::RED).bold()
                    );
                    ExitStatus::Error
                }
            },
            Some(MainAction::Quit) => ExitStatus::Success, // Handled by CLI loop
            None => {
                use nettoolskit_ui::Color;
                tracing::warn!("Unknown command attempted: {}", cmd);
                metrics.increment_counter("unknown_command_attempts");
                println!("{}: {}", "Unknown command".color(Color::RED), cmd);
                ExitStatus::Error
            }
        }
    };

    let after_hook_stats = run_after_command_plugins(&CommandHookContext {
        correlation_id: correlation_id.clone(),
        command: cmd.to_string(),
        command_key: command_key.clone(),
        status: Some(result),
    });
    metrics.set_gauge(
        "runtime_command_plugins_last_after_invoked",
        after_hook_stats.invoked as f64,
    );
    metrics.set_gauge(
        "runtime_command_plugins_last_after_failures",
        after_hook_stats.failures as f64,
    );
    if after_hook_stats.failures > 0 {
        metrics.increment_counter("runtime_command_plugins_hook_errors_total");
    }

    // Stop timer and log result with structured data
    let duration = timer.stop();
    update_runtime_latency_gauges(&metrics, &command_key, &command_timing_name, duration);

    // Log and convert result to CLI status
    let counter_name = match result {
        ExitStatus::Success => "successful_commands",
        ExitStatus::Error => "failed_commands",
        ExitStatus::Interrupted => "interrupted_commands",
    };
    let status_str = record_command_outcome_metrics(&metrics, &command_key, result);
    update_runtime_rate_gauges(&metrics);
    let cache_stats = with_command_cache(|cache| cache.stats());
    maybe_log_command_cache_stats(cache_stats, &metrics);

    info!(
        correlation_id = %correlation_id,
        command = %cmd,
        command_key = %command_key,
        duration_ms = duration.as_millis(),
        status = status_str,
        "Command execution completed"
    );
    metrics.increment_counter(counter_name);

    // Log metrics summary for this command
    metrics.log_summary();
    result
}

fn process_config_command(parts: &[&str]) -> ExitStatus {
    use nettoolskit_ui::Color;

    let Some(config_path) = AppConfig::default_config_path() else {
        println!(
            "{}",
            "Could not determine configuration path on this system."
                .color(Color::RED)
                .bold()
        );
        return ExitStatus::Error;
    };

    match parts.get(1).copied() {
        None | Some("show") => {
            print_effective_config(&config_path);
            ExitStatus::Success
        }
        Some("path") => {
            println!("{}", "📍 Configuration Path".color(Color::CYAN).bold());
            println!(
                "  {}",
                config_path.display().to_string().color(Color::GREEN)
            );
            println!(
                "  Exists: {}",
                if config_path.exists() {
                    "yes".color(Color::GREEN)
                } else {
                    "no".color(Color::YELLOW)
                }
            );
            ExitStatus::Success
        }
        Some("init") => {
            if config_path.exists() {
                println!(
                    "{}",
                    "Configuration file already exists."
                        .color(Color::YELLOW)
                        .bold()
                );
                println!("  {}", config_path.display());
                return ExitStatus::Success;
            }

            let config = AppConfig::default();
            match config.save_to(&config_path) {
                Ok(()) => {
                    apply_runtime_ui_config(&config);
                    println!(
                        "{}",
                        "✅ Configuration initialized".color(Color::GREEN).bold()
                    );
                    println!("  {}", config_path.display());
                    ExitStatus::Success
                }
                Err(err) => {
                    println!(
                        "{}: {}",
                        "Failed to initialize config".color(Color::RED),
                        err
                    );
                    ExitStatus::Error
                }
            }
        }
        Some("set") => {
            if parts.len() < 4 {
                print_config_usage();
                return ExitStatus::Error;
            }

            let key = parts[2];
            let value = parts[3..].join(" ");
            let mut config = load_persisted_or_default(&config_path);

            match set_config_value(&mut config, key, &value) {
                Ok(()) => match config.save_to(&config_path) {
                    Ok(()) => {
                        apply_runtime_ui_config(&config);
                        if is_ai_session_retention_key(key) {
                            apply_ai_session_retention_policy(config.general.ai_session_retention);
                        }
                        println!(
                            "{} {}={}",
                            "✅ Updated".color(Color::GREEN).bold(),
                            key.color(Color::CYAN),
                            value.color(Color::WHITE)
                        );
                        println!("  {}", config_path.display());
                        ExitStatus::Success
                    }
                    Err(err) => {
                        println!("{}: {}", "Failed to save config".color(Color::RED), err);
                        ExitStatus::Error
                    }
                },
                Err(err) => {
                    println!("{}: {}", "Invalid config value".color(Color::RED), err);
                    print_supported_config_keys();
                    ExitStatus::Error
                }
            }
        }
        Some("unset") => {
            if parts.len() < 3 {
                print_config_usage();
                return ExitStatus::Error;
            }

            let key = parts[2];
            let mut config = load_persisted_or_default(&config_path);

            match unset_config_value(&mut config, key) {
                Ok(()) => match config.save_to(&config_path) {
                    Ok(()) => {
                        apply_runtime_ui_config(&config);
                        if is_ai_session_retention_key(key) {
                            apply_ai_session_retention_policy(config.general.ai_session_retention);
                        }
                        println!(
                            "{} {}",
                            "✅ Reset".color(Color::GREEN).bold(),
                            key.color(Color::CYAN)
                        );
                        println!("  {}", config_path.display());
                        ExitStatus::Success
                    }
                    Err(err) => {
                        println!("{}: {}", "Failed to save config".color(Color::RED), err);
                        ExitStatus::Error
                    }
                },
                Err(err) => {
                    println!("{}: {}", "Invalid config key".color(Color::RED), err);
                    print_supported_config_keys();
                    ExitStatus::Error
                }
            }
        }
        Some("reset") => {
            let config = AppConfig::default();
            match config.save_to(&config_path) {
                Ok(()) => {
                    apply_runtime_ui_config(&config);
                    apply_ai_session_retention_policy(config.general.ai_session_retention);
                    println!(
                        "{}",
                        "✅ Configuration reset to defaults"
                            .color(Color::GREEN)
                            .bold()
                    );
                    println!("  {}", config_path.display());
                    ExitStatus::Success
                }
                Err(err) => {
                    println!("{}: {}", "Failed to reset config".color(Color::RED), err);
                    ExitStatus::Error
                }
            }
        }
        Some("help") => {
            print_config_usage();
            ExitStatus::Success
        }
        Some(_) => {
            println!("{}", "Unknown /config subcommand".color(Color::RED).bold());
            print_config_usage();
            ExitStatus::Error
        }
    }
}

fn print_effective_config(config_path: &Path) {
    use nettoolskit_ui::Color;

    let effective = AppConfig::load();
    println!(
        "{}",
        "⚙️  NetToolsKit Configuration".color(Color::CYAN).bold()
    );
    println!(
        "  File: {} ({})",
        config_path.display().to_string().color(Color::GREEN),
        if config_path.exists() {
            "exists".color(Color::GREEN)
        } else {
            "not found, using defaults/env".color(Color::YELLOW)
        }
    );
    println!();
    println!("{}", "[general]".color(Color::WHITE).bold());
    println!("  verbose = {}", effective.general.verbose);
    println!("  log_level = {}", effective.general.log_level);
    println!("  footer_output = {}", effective.general.footer_output);
    println!("  runtime_mode = {}", effective.general.runtime_mode);
    println!("  attention_bell = {}", effective.general.attention_bell);
    println!(
        "  attention_desktop_notification = {}",
        effective.general.attention_desktop_notification
    );
    println!(
        "  attention_unfocused_only = {}",
        effective.general.attention_unfocused_only
    );
    println!(
        "  predictive_input = {}",
        effective.general.predictive_input
    );
    println!(
        "  ai_session_retention = {}",
        effective.general.ai_session_retention
    );
    println!("{}", "[display]".color(Color::WHITE).bold());
    println!("  color = {}", effective.display.color);
    println!("  unicode = {}", effective.display.unicode);
    println!("{}", "[templates]".color(Color::WHITE).bold());
    println!(
        "  directory = {}",
        effective
            .templates
            .directory
            .as_deref()
            .unwrap_or("(default)")
    );
    println!("{}", "[shell]".color(Color::WHITE).bold());
    println!(
        "  default_shell = {}",
        effective.shell.default_shell.as_deref().unwrap_or("(none)")
    );
    println!();
    print_config_usage();
}

fn print_supported_config_keys() {
    use nettoolskit_ui::Color;
    println!("{}", "Supported keys:".color(Color::WHITE).bold());
    println!("  {}", "verbose".color(Color::CYAN));
    println!("  {}", "log_level".color(Color::CYAN));
    println!("  {}", "footer_output".color(Color::CYAN));
    println!("  {}", "runtime_mode".color(Color::CYAN));
    println!("  {}", "attention_bell".color(Color::CYAN));
    println!("  {}", "attention_desktop_notification".color(Color::CYAN));
    println!("  {}", "attention_unfocused_only".color(Color::CYAN));
    println!("  {}", "predictive_input".color(Color::CYAN));
    println!("  {}", "ai_session_retention".color(Color::CYAN));
    println!("  {}", "color".color(Color::CYAN));
    println!("  {}", "unicode".color(Color::CYAN));
    println!("  {}", "template_dir".color(Color::CYAN));
    println!("  {}", "shell".color(Color::CYAN));
}

fn print_config_usage() {
    use nettoolskit_ui::Color;
    println!("{}", "Usage:".color(Color::WHITE).bold());
    println!("  {}", "/config [show]".color(Color::GREEN));
    println!("  {}", "/config path".color(Color::GREEN));
    println!("  {}", "/config init".color(Color::GREEN));
    println!("  {}", "/config set <key> <value>".color(Color::GREEN));
    println!("  {}", "/config unset <key>".color(Color::GREEN));
    println!("  {}", "/config reset".color(Color::GREEN));
    println!();
    print_supported_config_keys();
}

fn load_persisted_or_default(config_path: &Path) -> AppConfig {
    if config_path.exists() {
        AppConfig::load_from(config_path).unwrap_or_default()
    } else {
        AppConfig::default()
    }
}

fn apply_runtime_ui_config(config: &AppConfig) {
    nettoolskit_ui::set_footer_output_enabled(config.general.footer_output);
}

fn is_ai_session_retention_key(key: &str) -> bool {
    matches!(
        key.trim().to_ascii_lowercase().as_str(),
        "ai_session_retention"
            | "ai-session-retention"
            | "general.ai_session_retention"
            | "general.ai-session-retention"
    )
}

fn apply_ai_session_retention_policy(retention: usize) {
    let keep_latest = retention.max(1);
    match prune_local_ai_session_snapshots(keep_latest) {
        Ok(Some(removed)) if removed > 0 => {
            println!(
                "  {}",
                format!("Pruned {removed} old local AI session snapshot(s)")
                    .color(nettoolskit_ui::Color::YELLOW)
            );
        }
        Ok(Some(_)) | Ok(None) => {}
        Err(err) => {
            warn!(error = %err, "Failed to apply AI session retention policy");
            println!(
                "  {}",
                format!("Warning: failed to prune local AI sessions: {err}")
                    .color(nettoolskit_ui::Color::YELLOW)
            );
        }
    }
}

fn set_config_value(config: &mut AppConfig, key: &str, value: &str) -> Result<(), String> {
    let normalized = key.trim().to_ascii_lowercase();
    match normalized.as_str() {
        "verbose" | "general.verbose" => {
            let parsed = parse_bool(value).ok_or_else(|| {
                "verbose must be one of: true, false, 1, 0, yes, no, on, off".to_string()
            })?;
            config.general.verbose = parsed;
            Ok(())
        }
        "log_level" | "log-level" | "general.log_level" | "general.log-level" => {
            config.general.log_level = parse_log_level(value)?;
            Ok(())
        }
        "footer_output" | "footer-output" | "general.footer_output" | "general.footer-output" => {
            let parsed = parse_bool(value).ok_or_else(|| {
                "footer_output must be one of: true, false, 1, 0, yes, no, on, off".to_string()
            })?;
            config.general.footer_output = parsed;
            Ok(())
        }
        "runtime_mode" | "runtime-mode" | "general.runtime_mode" | "general.runtime-mode" => {
            let parsed = parse_runtime_mode(value)?;
            config.general.runtime_mode = parsed;
            Ok(())
        }
        "attention_bell"
        | "attention-bell"
        | "general.attention_bell"
        | "general.attention-bell" => {
            let parsed = parse_bool(value).ok_or_else(|| {
                "attention_bell must be one of: true, false, 1, 0, yes, no, on, off".to_string()
            })?;
            config.general.attention_bell = parsed;
            Ok(())
        }
        "attention_desktop_notification"
        | "attention-desktop-notification"
        | "general.attention_desktop_notification"
        | "general.attention-desktop-notification" => {
            let parsed = parse_bool(value).ok_or_else(|| {
                "attention_desktop_notification must be one of: true, false, 1, 0, yes, no, on, off"
                    .to_string()
            })?;
            config.general.attention_desktop_notification = parsed;
            Ok(())
        }
        "attention_unfocused_only"
        | "attention-unfocused-only"
        | "general.attention_unfocused_only"
        | "general.attention-unfocused-only" => {
            let parsed = parse_bool(value).ok_or_else(|| {
                "attention_unfocused_only must be one of: true, false, 1, 0, yes, no, on, off"
                    .to_string()
            })?;
            config.general.attention_unfocused_only = parsed;
            Ok(())
        }
        "predictive_input"
        | "predictive-input"
        | "general.predictive_input"
        | "general.predictive-input" => {
            let parsed = parse_bool(value).ok_or_else(|| {
                "predictive_input must be one of: true, false, 1, 0, yes, no, on, off".to_string()
            })?;
            config.general.predictive_input = parsed;
            Ok(())
        }
        "ai_session_retention"
        | "ai-session-retention"
        | "general.ai_session_retention"
        | "general.ai-session-retention" => {
            let parsed = parse_positive_usize(value).ok_or_else(|| {
                "ai_session_retention must be a positive integer (>= 1)".to_string()
            })?;
            config.general.ai_session_retention = parsed;
            Ok(())
        }
        "color" | "display.color" => {
            let parsed = parse_color_mode(value)?;
            config.display.color = parsed;
            Ok(())
        }
        "unicode" | "display.unicode" => {
            let parsed = parse_unicode_mode(value)?;
            config.display.unicode = parsed;
            Ok(())
        }
        "template_dir" | "template-dir" | "templates.directory" => {
            config.templates.directory = Some(value.trim().to_string());
            Ok(())
        }
        "shell" | "default_shell" | "default-shell" | "shell.default_shell" => {
            config.shell.default_shell = Some(value.trim().to_string());
            Ok(())
        }
        _ => Err(format!("unsupported key '{key}'")),
    }
}

fn unset_config_value(config: &mut AppConfig, key: &str) -> Result<(), String> {
    let normalized = key.trim().to_ascii_lowercase();
    match normalized.as_str() {
        "verbose" | "general.verbose" => {
            config.general.verbose = false;
            Ok(())
        }
        "log_level" | "log-level" | "general.log_level" | "general.log-level" => {
            config.general.log_level = "info".to_string();
            Ok(())
        }
        "footer_output" | "footer-output" | "general.footer_output" | "general.footer-output" => {
            config.general.footer_output = true;
            Ok(())
        }
        "runtime_mode" | "runtime-mode" | "general.runtime_mode" | "general.runtime-mode" => {
            config.general.runtime_mode = AppConfig::default().general.runtime_mode;
            Ok(())
        }
        "attention_bell"
        | "attention-bell"
        | "general.attention_bell"
        | "general.attention-bell" => {
            config.general.attention_bell = false;
            Ok(())
        }
        "attention_desktop_notification"
        | "attention-desktop-notification"
        | "general.attention_desktop_notification"
        | "general.attention-desktop-notification" => {
            config.general.attention_desktop_notification = false;
            Ok(())
        }
        "attention_unfocused_only"
        | "attention-unfocused-only"
        | "general.attention_unfocused_only"
        | "general.attention-unfocused-only" => {
            config.general.attention_unfocused_only = false;
            Ok(())
        }
        "predictive_input"
        | "predictive-input"
        | "general.predictive_input"
        | "general.predictive-input" => {
            config.general.predictive_input = true;
            Ok(())
        }
        "ai_session_retention"
        | "ai-session-retention"
        | "general.ai_session_retention"
        | "general.ai-session-retention" => {
            config.general.ai_session_retention = default_ai_session_retention();
            Ok(())
        }
        "color" | "display.color" => {
            config.display.color = ColorMode::Auto;
            Ok(())
        }
        "unicode" | "display.unicode" => {
            config.display.unicode = UnicodeMode::Auto;
            Ok(())
        }
        "template_dir" | "template-dir" | "templates.directory" => {
            config.templates.directory = None;
            Ok(())
        }
        "shell" | "default_shell" | "default-shell" | "shell.default_shell" => {
            config.shell.default_shell = None;
            Ok(())
        }
        _ => Err(format!("unsupported key '{key}'")),
    }
}

fn parse_bool(value: &str) -> Option<bool> {
    match value.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Some(true),
        "0" | "false" | "no" | "off" => Some(false),
        _ => None,
    }
}

fn parse_positive_usize(value: &str) -> Option<usize> {
    let parsed = value.trim().parse::<usize>().ok()?;
    (parsed > 0).then_some(parsed)
}

fn parse_log_level(value: &str) -> Result<String, String> {
    match value.trim().to_ascii_lowercase().as_str() {
        "off" | "error" | "warn" | "info" | "debug" | "trace" => {
            Ok(value.trim().to_ascii_lowercase())
        }
        _ => Err("log_level must be one of: off, error, warn, info, debug, trace".to_string()),
    }
}

fn parse_runtime_mode(value: &str) -> Result<RuntimeMode, String> {
    value
        .trim()
        .parse::<RuntimeMode>()
        .map_err(|_| "runtime_mode must be one of: cli, service".to_string())
}

fn parse_color_mode(value: &str) -> Result<ColorMode, String> {
    match value.trim().to_ascii_lowercase().as_str() {
        "auto" => Ok(ColorMode::Auto),
        "always" => Ok(ColorMode::Always),
        "never" => Ok(ColorMode::Never),
        _ => Err("color must be one of: auto, always, never".to_string()),
    }
}

fn parse_unicode_mode(value: &str) -> Result<UnicodeMode, String> {
    match value.trim().to_ascii_lowercase().as_str() {
        "auto" => Ok(UnicodeMode::Auto),
        "always" => Ok(UnicodeMode::Always),
        "never" => Ok(UnicodeMode::Never),
        _ => Err("unicode must be one of: auto, always, never".to_string()),
    }
}

fn infer_command_from_text(text: &str) -> Option<String> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return None;
    }

    if trimmed.starts_with('/') {
        return Some(trimmed.to_string());
    }

    let tokens: Vec<&str> = trimmed.split_whitespace().collect();
    if tokens.is_empty() {
        return None;
    }

    let first = tokens[0].to_ascii_lowercase();
    let lowered: Vec<String> = tokens
        .iter()
        .map(|token| token.to_ascii_lowercase())
        .collect();
    let has = |keyword: &str| lowered.iter().any(|token| token == keyword);

    match first.as_str() {
        // direct command aliases (without slash)
        "help" | "ajuda" => Some("/help".to_string()),
        "manifest" | "manifests" => Some(format!("/{}", trimmed)),
        "translate" => Some(format!("/{}", trimmed)),
        "ai" => Some(format!("/{}", trimmed)),
        "task" => Some(format!("/{}", trimmed)),
        "tasks" | "tarefa" | "tarefas" => {
            if tokens.len() > 1 {
                Some(format!("/task {}", tokens[1..].join(" ")))
            } else {
                Some("/task".to_string())
            }
        }
        "assistant" | "copilot" => {
            if tokens.len() > 1 {
                Some(format!("/ai {}", tokens[1..].join(" ")))
            } else {
                Some("/ai".to_string())
            }
        }
        "clear" | "cls" | "limpar" => Some("/clear".to_string()),
        "traduzir" => {
            if tokens.len() > 1 {
                Some(format!("/translate {}", tokens[1..].join(" ")))
            } else {
                Some("/translate".to_string())
            }
        }
        "perguntar" | "ask" => {
            if tokens.len() > 1 {
                Some(format!("/ai ask {}", tokens[1..].join(" ")))
            } else {
                Some("/ai ask".to_string())
            }
        }
        "explain" | "explicar" | "explique" => {
            if tokens.len() > 1 {
                Some(format!("/ai explain {}", tokens[1..].join(" ")))
            } else {
                Some("/ai explain".to_string())
            }
        }
        "plan" | "planejar" | "planeje" => {
            if tokens.len() > 1 {
                Some(format!("/ai plan {}", tokens[1..].join(" ")))
            } else {
                Some("/ai plan".to_string())
            }
        }
        "config" | "configuracao" | "configuração" | "settings" => Some(format!("/{}", trimmed)),
        _ => {
            // keyword-based intent routing
            if has("help") || has("ajuda") {
                Some("/help".to_string())
            } else if has("manifest") || has("manifests") || has("manifests:") {
                if has("list") || has("listar") || has("discover") {
                    Some("/manifest list".to_string())
                } else if has("check") || has("validate") || has("validar") {
                    Some("/manifest check".to_string())
                } else if has("render") || has("preview") || has("previa") || has("prévia") {
                    Some("/manifest render".to_string())
                } else if has("apply") || has("aplicar") {
                    Some("/manifest apply".to_string())
                } else {
                    Some("/manifest".to_string())
                }
            } else if has("config") || has("configuracao") || has("configuração") || has("settings")
            {
                Some("/config".to_string())
            } else if has("ai") || has("assistant") || has("copilot") {
                if has("plan") || has("planejar") || has("planeje") {
                    Some("/ai plan".to_string())
                } else if has("explain") || has("explicar") || has("explique") {
                    Some("/ai explain".to_string())
                } else if has("apply") || has("aplicar") {
                    Some("/ai apply --dry-run".to_string())
                } else {
                    Some("/ai ask".to_string())
                }
            } else if has("clear") || has("limpar") || has("cls") {
                Some("/clear".to_string())
            } else {
                None
            }
        }
    }
}

/// Process non-command text input from CLI
///
/// Handles regular text input that is not a slash command.
/// Since NetToolsKit CLI is a command-driven tool, free-text input
/// is treated as unrecognized and the user is guided toward
/// available slash commands.
///
/// # Arguments
///
/// * `text` - The input text to process
///
/// # Returns
///
/// * `ExitStatus::Success` for empty/whitespace-only input (silently ignored)
/// * `ExitStatus::Success` for non-empty text (hint displayed)
pub async fn process_text(text: &str) -> ExitStatus {
    let metrics = runtime_metrics().clone();
    metrics.increment_counter("runtime_text_inputs_total");

    let trimmed = text.trim();

    // Silently ignore empty or whitespace-only input
    if trimmed.is_empty() {
        metrics.increment_counter("runtime_text_inputs_ignored_total");
        tracing::trace!("Empty text input ignored");
        return ExitStatus::Success;
    }

    if let Some(resolved_command) = infer_command_from_text(trimmed) {
        metrics.increment_counter("runtime_text_inputs_routed_total");
        tracing::info!(
            input = %trimmed,
            resolved_command = %resolved_command,
            "Routed free-text input to command"
        );
        return process_command(&resolved_command).await;
    }

    metrics.increment_counter("runtime_text_inputs_unrecognized_total");
    tracing::debug!(input = %trimmed, "Unrecognized text input");

    use nettoolskit_ui::Color;
    use owo_colors::OwoColorize;

    println!(
        "{}: {}",
        "Unrecognized input".color(Color::YELLOW),
        trimmed.color(Color::GRAY)
    );
    println!(
        "  {} Type {} to see available commands, or {} to open the palette.",
        "💡".color(Color::CYAN),
        "/help".color(Color::GREEN),
        "/".color(Color::GREEN)
    );

    ExitStatus::Success
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::sync::atomic::AtomicBool;
    use std::sync::OnceLock;
    use std::time::Duration;
    use tokio::sync::Mutex;

    static ENV_TEST_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

    async fn env_test_guard() -> tokio::sync::MutexGuard<'static, ()> {
        ENV_TEST_LOCK.get_or_init(|| Mutex::new(())).lock().await
    }

    #[test]
    fn sanitize_metric_component_normalizes_symbols() {
        assert_eq!(
            sanitize_metric_component(" Manifest Apply "),
            "manifest_apply"
        );
        assert_eq!(sanitize_metric_component("/x-y.z"), "x_y_z");
        assert_eq!(sanitize_metric_component("___"), "unknown");
    }

    #[test]
    fn command_metric_key_resolves_manifest_subcommands() {
        let key = command_metric_key(
            Some(MainAction::Manifest),
            Some("apply"),
            "/manifest apply a.yaml",
        );
        assert_eq!(key, "manifest_apply");

        let menu_key = command_metric_key(Some(MainAction::Manifest), None, "/manifest");
        assert_eq!(menu_key, "manifest_menu");
    }

    #[test]
    fn command_metric_key_resolves_unknown_commands() {
        let key = command_metric_key(None, None, "/custom-op --x");
        assert_eq!(key, "unknown_custom_op");
    }

    #[test]
    fn detect_async_manifest_alias_matches_top_level_commands() {
        let parts = vec!["/render-async", "sample.manifest.yaml"];
        let alias = detect_async_manifest_alias(None, &parts, None);
        assert_eq!(alias, Some(AsyncManifestAlias::Render));

        let parts = vec!["/apply-async", "sample.manifest.yaml"];
        let alias = detect_async_manifest_alias(None, &parts, None);
        assert_eq!(alias, Some(AsyncManifestAlias::Apply));

        let parts = vec!["/new-async", "sample.manifest.yaml"];
        let alias = detect_async_manifest_alias(None, &parts, None);
        assert_eq!(alias, Some(AsyncManifestAlias::New));
    }

    #[test]
    fn detect_async_manifest_alias_matches_manifest_subcommands() {
        let parts = vec!["/manifest", "render-async", "sample.manifest.yaml"];
        let alias =
            detect_async_manifest_alias(Some(MainAction::Manifest), &parts, Some("render-async"));
        assert_eq!(alias, Some(AsyncManifestAlias::Render));

        let parts = vec!["/manifest", "apply-async", "sample.manifest.yaml"];
        let alias =
            detect_async_manifest_alias(Some(MainAction::Manifest), &parts, Some("apply-async"));
        assert_eq!(alias, Some(AsyncManifestAlias::Apply));
    }

    #[test]
    fn first_positional_path_respects_start_index() {
        let parts = vec!["/render-async", "sample.manifest.yaml", "--output", "./out"];
        let path = first_positional_path(&parts, 1);
        assert_eq!(
            path.as_deref(),
            Some(std::path::Path::new("sample.manifest.yaml"))
        );

        let path = first_positional_path(&parts, 2);
        assert_eq!(path.as_deref(), Some(std::path::Path::new("./out")));
    }

    #[test]
    fn format_async_progress_includes_percent_and_steps() {
        let progress = CommandProgress::steps("Applying files", 2, 5);
        let message = format_async_progress("apply-async", &progress);
        assert!(message.contains("apply-async"));
        assert!(message.contains("Applying files"));
        assert!(message.contains("40%"));
        assert!(message.contains("2/5"));
    }

    #[test]
    fn ai_intent_from_subcommand_maps_known_values() {
        assert_eq!(AiIntent::from_subcommand("ask"), Some(AiIntent::Ask));
        assert_eq!(AiIntent::from_subcommand("PLAN"), Some(AiIntent::Plan));
        assert_eq!(
            AiIntent::from_subcommand("explain"),
            Some(AiIntent::Explain)
        );
        assert_eq!(
            AiIntent::from_subcommand("apply"),
            Some(AiIntent::ApplyDryRun)
        );
        assert_eq!(AiIntent::from_subcommand("unknown"), None);
    }

    #[test]
    fn collect_ai_prompt_skips_flags_and_joins_prompt_tokens() {
        let parts = vec!["/ai", "apply", "--dry-run", "create", "service", "layer"];
        let prompt = collect_ai_prompt(&parts, 2);
        assert_eq!(prompt, "create service layer");
    }

    #[test]
    fn parse_ai_context_paths_supports_comma_and_semicolon() {
        let paths = parse_ai_context_paths(
            "Cargo.toml, README.md; .temp/planning/enterprise-progress-tracker.md",
        );
        assert_eq!(paths.len(), 3);
        assert_eq!(paths[0], PathBuf::from("Cargo.toml"));
        assert_eq!(paths[1], PathBuf::from("README.md"));
        assert_eq!(
            paths[2],
            PathBuf::from(".temp/planning/enterprise-progress-tracker.md")
        );
    }

    #[test]
    fn parse_nonzero_usize_rejects_zero_and_invalid_values() {
        assert_eq!(parse_nonzero_usize("64"), Some(64));
        assert_eq!(parse_nonzero_usize("0"), None);
        assert_eq!(parse_nonzero_usize("-1"), None);
        assert_eq!(parse_nonzero_usize("abc"), None);
    }

    #[test]
    fn build_ai_request_includes_system_message_and_stream_mode() {
        let request = build_ai_request(AiIntent::Plan, "prepare migration plan");
        assert!(request.stream);
        assert!(request.max_output_tokens.is_some());
        assert!(request.temperature.is_some());
        assert_eq!(
            request.messages.first().map(|message| message.role),
            Some(AiRole::System)
        );
        assert_eq!(
            request.messages.get(1).map(|message| message.role),
            Some(AiRole::User)
        );
    }

    #[test]
    fn is_retriable_ai_error_matches_transient_variants() {
        assert!(is_retriable_ai_error(&AiProviderError::Timeout {
            timeout: Duration::from_secs(1)
        }));
        assert!(is_retriable_ai_error(&AiProviderError::Unavailable(
            "maintenance".to_string()
        )));
        assert!(is_retriable_ai_error(&AiProviderError::Transport(
            "socket".to_string()
        )));
        assert!(!is_retriable_ai_error(&AiProviderError::InvalidRequest(
            "bad".to_string()
        )));
    }

    #[test]
    fn ai_retry_delay_is_exponential_and_bounded() {
        let policy = AiRetryPolicy {
            max_retries: 3,
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_millis(250),
            request_timeout: Duration::from_secs(1),
        };

        assert_eq!(ai_retry_delay(policy, 1), Duration::from_millis(100));
        assert_eq!(ai_retry_delay(policy, 2), Duration::from_millis(200));
        assert_eq!(ai_retry_delay(policy, 3), Duration::from_millis(250));
    }

    #[test]
    fn evaluate_ai_rate_limit_enforces_budget_and_resets_after_window() {
        let start = Instant::now();
        let mut state = AiRateLimitState {
            window_started_at: start,
            used_requests: 0,
        };
        let policy = AiRateLimitPolicy {
            max_requests: 2,
            window: Duration::from_secs(60),
        };

        assert_eq!(evaluate_ai_rate_limit(&mut state, policy, start), Ok(1));
        assert_eq!(evaluate_ai_rate_limit(&mut state, policy, start), Ok(0));
        assert!(evaluate_ai_rate_limit(&mut state, policy, start).is_err());

        let after_window = start + Duration::from_secs(61);
        assert_eq!(
            evaluate_ai_rate_limit(&mut state, policy, after_window),
            Ok(1)
        );
    }

    #[test]
    fn update_ai_approval_metrics_updates_ratio_gauge() {
        let metrics = Metrics::new();
        update_ai_approval_metrics(&metrics, true);
        update_ai_approval_metrics(&metrics, false);

        assert_eq!(metrics.get_counter("runtime_ai_approvals_total"), 2);
        assert_eq!(
            metrics.get_counter("runtime_ai_approvals_approved_total"),
            1
        );
        assert_eq!(metrics.get_counter("runtime_ai_approvals_denied_total"), 1);
        assert_eq!(
            metrics.get_gauge("runtime_ai_approval_ratio_pct"),
            Some(50.0)
        );
    }

    #[test]
    fn record_ai_usage_estimates_updates_token_gauges() {
        let metrics = Metrics::new();
        let request = build_ai_request(AiIntent::Ask, "explain cache warming strategy");
        record_ai_usage_estimates(&metrics, &request, "Use staged rollout and telemetry.");

        assert!(metrics
            .get_gauge("runtime_ai_last_input_tokens_estimate")
            .is_some());
        assert!(metrics
            .get_gauge("runtime_ai_last_output_tokens_estimate")
            .is_some());
        assert!(metrics
            .get_gauge("runtime_ai_tokens_estimated_total")
            .is_some());
    }

    #[test]
    fn ai_error_guidance_message_is_available_for_transient_failures() {
        let timeout = AiProviderError::Timeout {
            timeout: Duration::from_secs(1),
        };
        let unavailable = AiProviderError::Unavailable("outage".to_string());
        assert!(ai_error_guidance_message(&timeout).is_some());
        assert!(ai_error_guidance_message(&unavailable).is_some());
    }

    #[tokio::test]
    async fn request_ai_stream_with_retry_retries_transient_errors() {
        let provider = MockAiProvider::with_scripted(
            mocked_ai_response(AiIntent::Ask, "hello"),
            vec![
                crate::execution::ai::MockAiOutcome::Error(AiProviderError::Timeout {
                    timeout: Duration::from_millis(5),
                }),
                crate::execution::ai::MockAiOutcome::Complete(AiResponse::new("mock", "ok")),
            ],
        );
        let request = build_ai_request(AiIntent::Ask, "hello");
        let policy = AiRetryPolicy {
            max_retries: 1,
            base_delay: Duration::from_millis(1),
            max_delay: Duration::from_millis(1),
            request_timeout: Duration::from_secs(1),
        };
        let metrics = Metrics::new();

        let (chunks, retries) =
            request_ai_stream_with_retry(&provider, &request, policy, &metrics, AiIntent::Ask)
                .await
                .expect("retry should recover");

        assert_eq!(retries, 1);
        assert_eq!(metrics.get_counter("runtime_ai_retries_total"), 1);
        assert_eq!(chunks.last().map(|item| item.content.as_str()), Some("ok"));
    }

    #[tokio::test]
    async fn run_async_alias_with_progress_returns_interrupted_when_flag_is_set() {
        let interrupted = AtomicBool::new(true);
        let status = run_async_alias_with_progress(
            "apply-async",
            Some(&interrupted),
            |_progress_tx| async move { ExitStatus::Success },
        )
        .await;
        assert_eq!(status, ExitStatus::Interrupted);
    }

    #[tokio::test]
    async fn run_async_alias_with_progress_returns_success_when_not_interrupted() {
        let interrupted = AtomicBool::new(false);
        let status = run_async_alias_with_progress(
            "apply-async",
            Some(&interrupted),
            |progress_tx| async move {
                let _ = progress_tx.send(CommandProgress::percent("Completed", 100));
                ExitStatus::Success
            },
        )
        .await;
        assert_eq!(status, ExitStatus::Success);
    }

    #[test]
    fn update_runtime_rate_gauges_computes_percentages() {
        let metrics = Metrics::new();
        metrics.increment_counter("runtime_commands_total");
        metrics.increment_counter("runtime_commands_total");
        metrics.increment_counter("runtime_commands_total");
        metrics.increment_counter("runtime_commands_total");
        metrics.increment_counter("runtime_commands_success_total");
        metrics.increment_counter("runtime_commands_success_total");
        metrics.increment_counter("runtime_commands_error_total");
        metrics.increment_counter("runtime_commands_interrupted_total");

        update_runtime_rate_gauges(&metrics);

        assert_eq!(
            metrics.get_gauge("runtime_command_success_rate_pct"),
            Some(50.0)
        );
        assert_eq!(
            metrics.get_gauge("runtime_command_error_rate_pct"),
            Some(25.0)
        );
        assert_eq!(
            metrics.get_gauge("runtime_command_cancellation_rate_pct"),
            Some(25.0)
        );
    }

    #[test]
    fn record_command_outcome_metrics_updates_counters() {
        let metrics = Metrics::new();
        let label = record_command_outcome_metrics(&metrics, "help", ExitStatus::Error);
        assert_eq!(label, "error");
        assert_eq!(metrics.get_counter("runtime_commands_error_total"), 1);
        assert_eq!(metrics.get_counter("runtime_command_help_error_total"), 1);
    }

    #[test]
    fn set_config_value_updates_known_keys() {
        let mut config = AppConfig::default();
        assert!(set_config_value(&mut config, "verbose", "true").is_ok());
        assert!(config.general.verbose);

        assert!(set_config_value(&mut config, "log_level", "trace").is_ok());
        assert_eq!(config.general.log_level, "trace");

        assert!(set_config_value(&mut config, "footer_output", "false").is_ok());
        assert!(!config.general.footer_output);

        assert!(set_config_value(&mut config, "runtime_mode", "service").is_ok());
        assert_eq!(config.general.runtime_mode, RuntimeMode::Service);

        assert!(set_config_value(&mut config, "attention_bell", "true").is_ok());
        assert!(config.general.attention_bell);

        assert!(set_config_value(&mut config, "attention_desktop_notification", "true").is_ok());
        assert!(config.general.attention_desktop_notification);

        assert!(set_config_value(&mut config, "attention_unfocused_only", "true").is_ok());
        assert!(config.general.attention_unfocused_only);

        assert!(set_config_value(&mut config, "predictive_input", "false").is_ok());
        assert!(!config.general.predictive_input);

        assert!(set_config_value(&mut config, "ai_session_retention", "12").is_ok());
        assert_eq!(config.general.ai_session_retention, 12);

        assert!(set_config_value(&mut config, "color", "never").is_ok());
        assert_eq!(config.display.color, ColorMode::Never);

        assert!(set_config_value(&mut config, "template_dir", "/tmp/x").is_ok());
        assert_eq!(config.templates.directory.as_deref(), Some("/tmp/x"));
    }

    #[test]
    fn set_config_value_rejects_unknown_key() {
        let mut config = AppConfig::default();
        let result = set_config_value(&mut config, "unknown", "x");
        assert!(result.is_err());
    }

    #[test]
    fn unset_config_value_resets_known_keys() {
        let mut config = AppConfig::default();
        config.general.verbose = true;
        config.general.log_level = "debug".to_string();
        config.general.footer_output = false;
        config.general.runtime_mode = RuntimeMode::Service;
        config.general.attention_bell = true;
        config.general.attention_desktop_notification = true;
        config.general.attention_unfocused_only = true;
        config.general.predictive_input = false;
        config.general.ai_session_retention = 3;
        config.display.color = ColorMode::Always;
        config.templates.directory = Some("/tmp/x".to_string());

        assert!(unset_config_value(&mut config, "verbose").is_ok());
        assert!(unset_config_value(&mut config, "log_level").is_ok());
        assert!(unset_config_value(&mut config, "footer_output").is_ok());
        assert!(unset_config_value(&mut config, "runtime_mode").is_ok());
        assert!(unset_config_value(&mut config, "attention_bell").is_ok());
        assert!(unset_config_value(&mut config, "attention_desktop_notification").is_ok());
        assert!(unset_config_value(&mut config, "attention_unfocused_only").is_ok());
        assert!(unset_config_value(&mut config, "predictive_input").is_ok());
        assert!(unset_config_value(&mut config, "ai_session_retention").is_ok());
        assert!(unset_config_value(&mut config, "color").is_ok());
        assert!(unset_config_value(&mut config, "template_dir").is_ok());

        assert!(!config.general.verbose);
        assert_eq!(config.general.log_level, "info");
        assert!(config.general.footer_output);
        assert_eq!(config.general.runtime_mode, RuntimeMode::Cli);
        assert!(!config.general.attention_bell);
        assert!(!config.general.attention_desktop_notification);
        assert!(!config.general.attention_unfocused_only);
        assert!(config.general.predictive_input);
        assert_eq!(
            config.general.ai_session_retention,
            default_ai_session_retention()
        );
        assert_eq!(config.display.color, ColorMode::Auto);
        assert_eq!(config.templates.directory, None);
    }

    #[test]
    fn parse_bool_handles_supported_values() {
        assert_eq!(parse_bool("true"), Some(true));
        assert_eq!(parse_bool("1"), Some(true));
        assert_eq!(parse_bool("on"), Some(true));
        assert_eq!(parse_bool("false"), Some(false));
        assert_eq!(parse_bool("0"), Some(false));
        assert_eq!(parse_bool("off"), Some(false));
        assert_eq!(parse_bool("maybe"), None);
    }

    #[test]
    fn parse_positive_usize_accepts_nonzero_values_only() {
        assert_eq!(parse_positive_usize("1"), Some(1));
        assert_eq!(parse_positive_usize("40"), Some(40));
        assert_eq!(parse_positive_usize("0"), None);
        assert_eq!(parse_positive_usize("-1"), None);
        assert_eq!(parse_positive_usize("x"), None);
    }

    #[test]
    fn parse_log_level_accepts_known_values() {
        assert_eq!(parse_log_level("OFF").as_deref(), Ok("off"));
        assert_eq!(parse_log_level("warn").as_deref(), Ok("warn"));
        assert_eq!(parse_log_level("Trace").as_deref(), Ok("trace"));
    }

    #[test]
    fn parse_log_level_rejects_unknown_values() {
        assert!(parse_log_level("verbose").is_err());
    }

    #[test]
    fn parse_runtime_mode_accepts_known_values() {
        assert_eq!(parse_runtime_mode("cli"), Ok(RuntimeMode::Cli));
        assert_eq!(parse_runtime_mode("service"), Ok(RuntimeMode::Service));
    }

    #[test]
    fn parse_runtime_mode_rejects_unknown_values() {
        assert!(parse_runtime_mode("worker").is_err());
    }

    #[test]
    fn infer_command_from_text_routes_direct_aliases() {
        assert_eq!(infer_command_from_text("help").as_deref(), Some("/help"));
        assert_eq!(infer_command_from_text("ajuda").as_deref(), Some("/help"));
        assert_eq!(
            infer_command_from_text("ai ask explain caching").as_deref(),
            Some("/ai ask explain caching")
        );
        assert_eq!(
            infer_command_from_text("planejar migracao").as_deref(),
            Some("/ai plan migracao")
        );
        assert_eq!(infer_command_from_text("clear").as_deref(), Some("/clear"));
        assert_eq!(infer_command_from_text("limpar").as_deref(), Some("/clear"));
        assert_eq!(
            infer_command_from_text("manifest check sample.yaml").as_deref(),
            Some("/manifest check sample.yaml")
        );
        assert_eq!(
            infer_command_from_text("translate --from dotnet --to rust a.cs.hbs").as_deref(),
            Some("/translate --from dotnet --to rust a.cs.hbs")
        );
    }

    #[test]
    fn infer_command_from_text_routes_keyword_intents() {
        assert_eq!(
            infer_command_from_text("listar manifests").as_deref(),
            Some("/manifest list")
        );
        assert_eq!(
            infer_command_from_text("please validate manifest").as_deref(),
            Some("/manifest check")
        );
        assert_eq!(
            infer_command_from_text("quero preview do manifest").as_deref(),
            Some("/manifest render")
        );
        assert_eq!(
            infer_command_from_text("open settings").as_deref(),
            Some("/config")
        );
        assert_eq!(
            infer_command_from_text("assistant explain retry policy").as_deref(),
            Some("/ai explain retry policy")
        );
        assert_eq!(
            infer_command_from_text("pode limpar a tela?").as_deref(),
            Some("/clear")
        );
    }

    #[test]
    fn infer_command_from_text_returns_none_for_unrelated_text() {
        assert_eq!(infer_command_from_text(""), None);
        assert_eq!(infer_command_from_text("   "), None);
        assert_eq!(infer_command_from_text("hello world"), None);
    }

    #[test]
    fn first_manifest_positional_path_extracts_apply_path() {
        let parts = vec!["/manifest", "apply", "feature.manifest.yaml", "--dry-run"];
        let path = first_manifest_positional_path(&parts);
        assert_eq!(
            path.as_deref(),
            Some(std::path::Path::new("feature.manifest.yaml"))
        );
    }

    #[test]
    fn parse_output_root_extracts_output_value() {
        let parts = vec![
            "/manifest",
            "apply",
            "feature.manifest.yaml",
            "--output",
            "./src",
        ];
        let output = parse_output_root(&parts);
        assert_eq!(output.as_deref(), Some(std::path::Path::new("./src")));
    }

    #[tokio::test]
    async fn process_ai_command_apply_without_dry_run_returns_error() {
        let parts = vec!["/ai", "apply", "update", "service"];
        let status = process_ai_command(&parts, Some("apply")).await;
        assert_eq!(status, ExitStatus::Error);
    }

    #[tokio::test]
    async fn process_ai_command_apply_with_explicit_write_approval_succeeds() {
        let parts = vec!["/ai", "apply", "--approve-write", "update", "service"];
        let status = process_ai_command(&parts, Some("apply")).await;
        assert_eq!(status, ExitStatus::Success);
    }

    #[tokio::test]
    async fn process_ai_command_resume_without_session_id_returns_error() {
        let parts = vec!["/ai", "resume"];
        let status = process_ai_command(&parts, Some("resume")).await;
        assert_eq!(status, ExitStatus::Error);
    }

    #[tokio::test]
    async fn process_ai_command_resume_with_session_id_succeeds() {
        let parts = vec!["/ai", "resume", "session-dev"];
        let status = process_ai_command(&parts, Some("resume")).await;
        assert_eq!(status, ExitStatus::Success);
    }

    #[test]
    fn parse_task_intent_kind_maps_supported_values() {
        assert_eq!(
            parse_task_intent_kind("command"),
            Some(TaskIntentKind::CommandExecution)
        );
        assert_eq!(
            parse_task_intent_kind("ai-plan"),
            Some(TaskIntentKind::AiPlan)
        );
        assert_eq!(
            parse_task_intent_kind("apply-dry-run"),
            Some(TaskIntentKind::AiApplyDryRun)
        );
        assert_eq!(parse_task_intent_kind("unknown"), None);
    }

    #[test]
    fn task_retry_delay_is_exponential_and_bounded() {
        let policy = TaskWorkerPolicy {
            queue_capacity: 16,
            max_concurrency: 2,
            max_retries: 3,
            retry_base_delay: Duration::from_millis(40),
            retry_max_delay: Duration::from_millis(90),
        };

        assert_eq!(task_retry_delay(policy, 1), Duration::from_millis(40));
        assert_eq!(task_retry_delay(policy, 2), Duration::from_millis(80));
        assert_eq!(task_retry_delay(policy, 3), Duration::from_millis(90));
    }

    #[test]
    fn resolve_task_execution_target_service_uses_background_worker_label() {
        let (target, _note) = resolve_task_execution_target(RuntimeMode::Service);
        assert_eq!(target, "background-worker-local");
    }

    #[tokio::test]
    async fn process_task_command_submit_ai_plan_succeeds() {
        let parts = vec!["/task", "submit", "ai-plan", "prepare", "deployment"];
        let status = process_task_command(&parts).await;
        assert_eq!(status, ExitStatus::Success);
    }

    #[tokio::test]
    async fn process_task_command_submit_service_mode_queues_background_task() {
        let _guard = env_test_guard().await;
        std::env::set_var("NTK_RUNTIME_MODE", "service");
        std::env::set_var("NTK_AI_PROVIDER", "mock");

        let payload_marker = format!("service-queue-test-{}", current_unix_timestamp_ms());
        let parts = vec!["/task", "submit", "ai-plan", payload_marker.as_str()];
        let status = process_task_command(&parts).await;

        std::env::remove_var("NTK_AI_PROVIDER");
        std::env::remove_var("NTK_RUNTIME_MODE");

        assert_eq!(status, ExitStatus::Success);

        let record = with_task_registry(|registry| {
            registry
                .values()
                .filter(|task| task.runtime_mode == RuntimeMode::Service)
                .filter(|task| task.intent.payload.contains(payload_marker.as_str()))
                .max_by_key(|task| task.updated_at_unix_ms)
                .cloned()
        })
        .expect("service task should exist in registry");

        assert_eq!(record.runtime_mode, RuntimeMode::Service);
        assert_eq!(record.execution_target, "background-worker-local");
        assert!(record.max_attempts >= 1);
        assert!(matches!(
            record.status,
            TaskExecutionStatus::Queued
                | TaskExecutionStatus::Running
                | TaskExecutionStatus::Succeeded
                | TaskExecutionStatus::Failed
                | TaskExecutionStatus::Cancelled
        ));
    }

    #[tokio::test]
    async fn process_task_command_submit_without_payload_returns_error() {
        let parts = vec!["/task", "submit", "ai-plan"];
        let status = process_task_command(&parts).await;
        assert_eq!(status, ExitStatus::Error);
    }

    #[tokio::test]
    async fn process_task_command_watch_without_id_returns_error() {
        let parts = vec!["/task", "watch"];
        let status = process_task_command(&parts).await;
        assert_eq!(status, ExitStatus::Error);
    }

    #[test]
    fn infer_command_from_text_routes_task_aliases() {
        assert_eq!(
            infer_command_from_text("task list").as_deref(),
            Some("/task list")
        );
        assert_eq!(
            infer_command_from_text("tarefas submit ai-plan objetivo").as_deref(),
            Some("/task submit ai-plan objetivo")
        );
    }
}
