//! Repository workflow orchestration with explicit policy gating.
//!
//! This module provides a bounded workflow for remote repository automation:
//! `clone -> branch -> execute -> commit -> push -> PR`.
//! All mutating actions are blocked unless policy is explicitly enabled.

use nettoolskit_core::AppConfig;
use serde::Deserialize;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

/// Enable repository workflow automation.
pub const NTK_REPO_WORKFLOW_ENABLED_ENV: &str = "NTK_REPO_WORKFLOW_ENABLED";
/// Comma/semicolon allowlist of repository hosts (`github.com`, `gitlab.com`, `local`).
pub const NTK_REPO_WORKFLOW_ALLOWED_HOSTS_ENV: &str = "NTK_REPO_WORKFLOW_ALLOWED_HOSTS";
/// Comma/semicolon allowlist of command prefixes (`cargo test`, `dotnet test`).
pub const NTK_REPO_WORKFLOW_ALLOWED_COMMANDS_ENV: &str = "NTK_REPO_WORKFLOW_ALLOWED_COMMANDS";
/// Allow `git push` step for repo workflow jobs.
pub const NTK_REPO_WORKFLOW_ALLOW_PUSH_ENV: &str = "NTK_REPO_WORKFLOW_ALLOW_PUSH";
/// Allow pull request creation step (`gh pr create`) for repo workflow jobs.
pub const NTK_REPO_WORKFLOW_ALLOW_PR_ENV: &str = "NTK_REPO_WORKFLOW_ALLOW_PR";
/// Optional base directory for repository workflow workspaces.
pub const NTK_REPO_WORKFLOW_BASE_DIR_ENV: &str = "NTK_REPO_WORKFLOW_BASE_DIR";

/// Policy controls for repository workflow execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepoWorkflowPolicy {
    /// Enables repository workflow execution.
    pub enabled: bool,
    /// Allowlisted repository hosts.
    pub allowed_hosts: Vec<String>,
    /// Allowlisted command prefixes for execution step.
    pub allowed_command_prefixes: Vec<String>,
    /// Allows push operation.
    pub allow_push: bool,
    /// Allows pull request creation operation.
    pub allow_pull_request: bool,
    /// Workspace root path where job clones are created.
    pub workspace_root: PathBuf,
}

impl Default for RepoWorkflowPolicy {
    fn default() -> Self {
        Self {
            enabled: false,
            allowed_hosts: Vec::new(),
            allowed_command_prefixes: Vec::new(),
            allow_push: false,
            allow_pull_request: false,
            workspace_root: default_workspace_root(),
        }
    }
}

impl RepoWorkflowPolicy {
    /// Resolve policy from environment.
    #[must_use]
    pub fn from_env() -> Self {
        let mut policy = Self::default();

        if let Ok(value) = std::env::var(NTK_REPO_WORKFLOW_ENABLED_ENV) {
            if let Some(parsed) = parse_bool(&value) {
                policy.enabled = parsed;
            }
        }
        if let Ok(value) = std::env::var(NTK_REPO_WORKFLOW_ALLOWED_HOSTS_ENV) {
            policy.allowed_hosts = parse_list(&value)
                .into_iter()
                .map(|host| host.to_ascii_lowercase())
                .collect();
        }
        if let Ok(value) = std::env::var(NTK_REPO_WORKFLOW_ALLOWED_COMMANDS_ENV) {
            policy.allowed_command_prefixes = parse_list(&value)
                .into_iter()
                .map(|command| command.to_ascii_lowercase())
                .collect();
        }
        if let Ok(value) = std::env::var(NTK_REPO_WORKFLOW_ALLOW_PUSH_ENV) {
            if let Some(parsed) = parse_bool(&value) {
                policy.allow_push = parsed;
            }
        }
        if let Ok(value) = std::env::var(NTK_REPO_WORKFLOW_ALLOW_PR_ENV) {
            if let Some(parsed) = parse_bool(&value) {
                policy.allow_pull_request = parsed;
            }
        }
        if let Ok(value) = std::env::var(NTK_REPO_WORKFLOW_BASE_DIR_ENV) {
            let trimmed = value.trim();
            if !trimmed.is_empty() {
                policy.workspace_root = PathBuf::from(trimmed);
            }
        }

        policy
    }
}

/// Parsed repository workflow request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepoWorkflowRequest {
    /// Source repository URL/path.
    pub repo_url: String,
    /// Branch to create and execute on.
    pub branch_name: String,
    /// Command to execute in cloned repository.
    pub command: String,
    /// Commit message used for local commit step.
    pub commit_message: String,
    /// Whether push should be attempted.
    pub push: bool,
    /// Whether PR creation should be attempted.
    pub open_pull_request: bool,
    /// Whether the workflow should run in planning mode only.
    pub dry_run: bool,
}

/// Deterministic workflow plan generated for one request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepoWorkflowPlan {
    /// Planned workspace path for the job.
    pub workspace_path: PathBuf,
    /// Ordered shell-level workflow steps.
    pub steps: Vec<String>,
}

/// Execution result for a repository workflow request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepoWorkflowResult {
    /// Expanded plan used by the execution.
    pub plan: RepoWorkflowPlan,
    /// True when commands were executed (not dry-run).
    pub executed: bool,
    /// Human-readable summary.
    pub summary: String,
}

/// Repository workflow failures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RepoWorkflowError {
    /// Input payload is malformed.
    InvalidPayload(String),
    /// Request is blocked by policy.
    PolicyDenied(String),
    /// Execution command failed.
    ExecutionFailed(String),
}

impl Display for RepoWorkflowError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidPayload(message) => write!(f, "invalid repo workflow payload: {message}"),
            Self::PolicyDenied(message) => write!(f, "repo workflow policy denied: {message}"),
            Self::ExecutionFailed(message) => {
                write!(f, "repo workflow execution failed: {message}")
            }
        }
    }
}

impl std::error::Error for RepoWorkflowError {}

/// Parse workflow payload from JSON or key-value format.
///
/// Supported key-value format:
/// `repo=<url>;branch=<name>;command=<cmd>;commit=<msg>;push=<bool>;open_pr=<bool>;dry_run=<bool>`
///
/// # Errors
///
/// Returns an error when required fields are missing or malformed.
pub fn parse_repo_workflow_payload(
    payload: &str,
) -> Result<RepoWorkflowRequest, RepoWorkflowError> {
    let trimmed = payload.trim();
    if trimmed.is_empty() {
        return Err(RepoWorkflowError::InvalidPayload(
            "payload cannot be empty".to_string(),
        ));
    }

    if trimmed.starts_with('{') {
        let parsed: RepoWorkflowRequestJson = serde_json::from_str(trimmed)
            .map_err(|error| RepoWorkflowError::InvalidPayload(error.to_string()))?;
        return parsed.into_request();
    }

    parse_key_value_payload(trimmed)
}

/// Validate request against configured policy.
///
/// # Errors
///
/// Returns an error if request is denied by policy constraints.
pub fn validate_repo_workflow_request(
    request: &RepoWorkflowRequest,
    policy: &RepoWorkflowPolicy,
) -> Result<(), RepoWorkflowError> {
    if !policy.enabled {
        return Err(RepoWorkflowError::PolicyDenied(
            "workflow is disabled; set NTK_REPO_WORKFLOW_ENABLED=true".to_string(),
        ));
    }

    let host = extract_repo_host(&request.repo_url).ok_or_else(|| {
        RepoWorkflowError::PolicyDenied("unable to resolve repository host".to_string())
    })?;
    let host_allowed = policy
        .allowed_hosts
        .iter()
        .any(|allowed| allowed.eq_ignore_ascii_case(&host));
    if !host_allowed {
        return Err(RepoWorkflowError::PolicyDenied(format!(
            "repository host `{host}` is not allowlisted"
        )));
    }

    if policy.allowed_command_prefixes.is_empty() {
        return Err(RepoWorkflowError::PolicyDenied(
            "no allowed command prefixes configured".to_string(),
        ));
    }

    let normalized_command = request.command.trim().to_ascii_lowercase();
    let command_allowed = policy
        .allowed_command_prefixes
        .iter()
        .any(|prefix| normalized_command.starts_with(prefix));
    if !command_allowed {
        return Err(RepoWorkflowError::PolicyDenied(format!(
            "command `{}` is not allowlisted",
            request.command
        )));
    }

    if request.push && !policy.allow_push {
        return Err(RepoWorkflowError::PolicyDenied(
            "push step requested but NTK_REPO_WORKFLOW_ALLOW_PUSH is disabled".to_string(),
        ));
    }
    if request.open_pull_request && !policy.allow_pull_request {
        return Err(RepoWorkflowError::PolicyDenied(
            "pull request step requested but NTK_REPO_WORKFLOW_ALLOW_PR is disabled".to_string(),
        ));
    }
    if request.open_pull_request && !request.push {
        return Err(RepoWorkflowError::InvalidPayload(
            "open_pr=true requires push=true".to_string(),
        ));
    }

    Ok(())
}

/// Execute repository workflow request under policy controls.
///
/// # Errors
///
/// Returns an error if policy denies request or any execution step fails.
pub fn execute_repo_workflow(
    request: &RepoWorkflowRequest,
    policy: &RepoWorkflowPolicy,
) -> Result<RepoWorkflowResult, RepoWorkflowError> {
    validate_repo_workflow_request(request, policy)?;

    let workspace_path =
        allocate_workspace_path(policy.workspace_root.as_path(), &request.repo_url);
    let plan = build_plan(request, &workspace_path);

    if request.dry_run {
        return Ok(RepoWorkflowResult {
            plan,
            executed: false,
            summary: "Dry-run: workflow plan generated; no commands executed.".to_string(),
        });
    }

    std::fs::create_dir_all(policy.workspace_root.as_path()).map_err(|error| {
        RepoWorkflowError::ExecutionFailed(format!(
            "failed to create workspace root `{}`: {error}",
            policy.workspace_root.display()
        ))
    })?;

    let workspace_display = workspace_path.display().to_string();
    run_command(
        "git",
        &[
            "clone",
            "--depth",
            "1",
            request.repo_url.as_str(),
            workspace_display.as_str(),
        ],
        None,
    )?;
    run_command(
        "git",
        &[
            "-C",
            workspace_display.as_str(),
            "checkout",
            "-b",
            request.branch_name.as_str(),
        ],
        None,
    )?;
    run_shell_command(request.command.as_str(), workspace_path.as_path())?;

    let has_changes = repository_has_changes(workspace_path.as_path())?;
    if !has_changes {
        return Ok(RepoWorkflowResult {
            plan,
            executed: true,
            summary:
                "Workflow executed: no file changes detected after command; commit/push/PR steps skipped."
                    .to_string(),
        });
    }

    run_command(
        "git",
        &["-C", workspace_display.as_str(), "add", "-A"],
        None,
    )?;
    run_command(
        "git",
        &[
            "-C",
            workspace_display.as_str(),
            "commit",
            "-m",
            request.commit_message.as_str(),
        ],
        None,
    )?;

    if request.push {
        run_command(
            "git",
            &[
                "-C",
                workspace_display.as_str(),
                "push",
                "-u",
                "origin",
                request.branch_name.as_str(),
            ],
            None,
        )?;
    }

    if request.open_pull_request {
        run_command(
            "gh",
            &[
                "pr",
                "create",
                "--fill",
                "--head",
                request.branch_name.as_str(),
            ],
            Some(workspace_path.as_path()),
        )?;
    }

    Ok(RepoWorkflowResult {
        plan,
        executed: true,
        summary:
            "Workflow executed successfully (clone, branch, command, commit, optional push/PR)."
                .to_string(),
    })
}

#[derive(Debug, Deserialize)]
struct RepoWorkflowRequestJson {
    repo: Option<String>,
    repository: Option<String>,
    repo_url: Option<String>,
    branch: Option<String>,
    branch_name: Option<String>,
    command: Option<String>,
    run: Option<String>,
    commit: Option<String>,
    commit_message: Option<String>,
    push: Option<bool>,
    open_pr: Option<bool>,
    pull_request: Option<bool>,
    dry_run: Option<bool>,
}

impl RepoWorkflowRequestJson {
    fn into_request(self) -> Result<RepoWorkflowRequest, RepoWorkflowError> {
        let repo_url = required_field(self.repo.or(self.repository).or(self.repo_url), "repo")?;
        let branch_name = required_field(self.branch.or(self.branch_name), "branch")?;
        let command = required_field(self.command.or(self.run), "command")?;
        let commit_message = self
            .commit
            .or(self.commit_message)
            .unwrap_or_else(|| format!("chore(ntk): automated workflow for `{branch_name}`"));
        let push = self.push.unwrap_or(false);
        let open_pull_request = self.open_pr.or(self.pull_request).unwrap_or(false);
        let dry_run = self.dry_run.unwrap_or(true);

        sanitize_request(RepoWorkflowRequest {
            repo_url,
            branch_name,
            command,
            commit_message,
            push,
            open_pull_request,
            dry_run,
        })
    }
}

fn parse_key_value_payload(payload: &str) -> Result<RepoWorkflowRequest, RepoWorkflowError> {
    let mut fields = HashMap::new();
    for segment in payload.split(';') {
        let trimmed = segment.trim();
        if trimmed.is_empty() {
            continue;
        }
        let (raw_key, raw_value) = trimmed.split_once('=').ok_or_else(|| {
            RepoWorkflowError::InvalidPayload(format!(
                "expected `key=value` segment, got `{trimmed}`"
            ))
        })?;
        let key = raw_key.trim().to_ascii_lowercase();
        let value = strip_quotes(raw_value.trim()).to_string();
        fields.insert(key, value);
    }

    let repo_url = required_field(
        fields
            .remove("repo")
            .or_else(|| fields.remove("repository"))
            .or_else(|| fields.remove("repo_url")),
        "repo",
    )?;
    let branch_name = required_field(
        fields
            .remove("branch")
            .or_else(|| fields.remove("branch_name")),
        "branch",
    )?;
    let command = required_field(
        fields.remove("command").or_else(|| fields.remove("run")),
        "command",
    )?;
    let commit_message = fields
        .remove("commit")
        .or_else(|| fields.remove("commit_message"))
        .unwrap_or_else(|| format!("chore(ntk): automated workflow for `{branch_name}`"));
    let push = fields
        .remove("push")
        .and_then(|value| parse_bool(&value))
        .unwrap_or(false);
    let open_pull_request = fields
        .remove("open_pr")
        .or_else(|| fields.remove("pull_request"))
        .and_then(|value| parse_bool(&value))
        .unwrap_or(false);
    let dry_run = fields
        .remove("dry_run")
        .and_then(|value| parse_bool(&value))
        .unwrap_or(true);

    sanitize_request(RepoWorkflowRequest {
        repo_url,
        branch_name,
        command,
        commit_message,
        push,
        open_pull_request,
        dry_run,
    })
}

fn sanitize_request(
    request: RepoWorkflowRequest,
) -> Result<RepoWorkflowRequest, RepoWorkflowError> {
    if request.branch_name.contains(char::is_whitespace) {
        return Err(RepoWorkflowError::InvalidPayload(
            "branch must not contain whitespace".to_string(),
        ));
    }
    if request.command.trim().is_empty() {
        return Err(RepoWorkflowError::InvalidPayload(
            "command must not be empty".to_string(),
        ));
    }

    Ok(RepoWorkflowRequest {
        repo_url: request.repo_url.trim().to_string(),
        branch_name: request.branch_name.trim().to_string(),
        command: request.command.trim().to_string(),
        commit_message: request.commit_message.trim().to_string(),
        push: request.push,
        open_pull_request: request.open_pull_request,
        dry_run: request.dry_run,
    })
}

fn build_plan(request: &RepoWorkflowRequest, workspace_path: &Path) -> RepoWorkflowPlan {
    let workspace = workspace_path.display();
    let mut steps = vec![
        format!("git clone --depth 1 {} {}", request.repo_url, workspace),
        format!("git -C {} checkout -b {}", workspace, request.branch_name),
        format!("(cd {} && {})", workspace, request.command),
        format!("git -C {} add -A", workspace),
        format!(
            "git -C {} commit -m \"{}\"",
            workspace, request.commit_message
        ),
    ];

    if request.push {
        steps.push(format!(
            "git -C {} push -u origin {}",
            workspace, request.branch_name
        ));
    }
    if request.open_pull_request {
        steps.push(format!(
            "(cd {} && gh pr create --fill --head {})",
            workspace, request.branch_name
        ));
    }

    RepoWorkflowPlan {
        workspace_path: workspace_path.to_path_buf(),
        steps,
    }
}

fn allocate_workspace_path(root: &Path, repo_url: &str) -> PathBuf {
    let slug = sanitize_repo_slug(repo_url);
    let now = current_unix_timestamp_ms();
    root.join(format!("{now}-{slug}"))
}

fn sanitize_repo_slug(repo_url: &str) -> String {
    let normalized = repo_url.replace('\\', "/");
    let tail = normalized
        .rsplit('/')
        .next()
        .unwrap_or("repo")
        .trim_end_matches(".git");
    let base = tail
        .rsplit(':')
        .next()
        .unwrap_or("repo")
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else {
                '-'
            }
        })
        .collect::<String>();

    if base.is_empty() {
        "repo".to_string()
    } else {
        base
    }
}

fn repository_has_changes(workspace: &Path) -> Result<bool, RepoWorkflowError> {
    let workspace_display = workspace.display().to_string();
    let output = capture_command(
        "git",
        &["-C", workspace_display.as_str(), "status", "--porcelain"],
        None,
    )?;
    Ok(!output.trim().is_empty())
}

fn run_shell_command(command_line: &str, current_dir: &Path) -> Result<(), RepoWorkflowError> {
    let output = {
        #[cfg(target_os = "windows")]
        {
            Command::new("cmd")
                .arg("/C")
                .arg(command_line)
                .current_dir(current_dir)
                .output()
        }
        #[cfg(not(target_os = "windows"))]
        {
            Command::new("sh")
                .arg("-lc")
                .arg(command_line)
                .current_dir(current_dir)
                .output()
        }
    }
    .map_err(|error| RepoWorkflowError::ExecutionFailed(error.to_string()))?;

    if output.status.success() {
        return Ok(());
    }

    let stderr = normalize_output(&output.stderr);
    let stdout = normalize_output(&output.stdout);
    Err(RepoWorkflowError::ExecutionFailed(format!(
        "shell command `{command_line}` failed (code {:?}) stdout=`{stdout}` stderr=`{stderr}`",
        output.status.code()
    )))
}

fn run_command(
    program: &str,
    args: &[&str],
    current_dir: Option<&Path>,
) -> Result<(), RepoWorkflowError> {
    let mut command = Command::new(program);
    command.args(args);
    if let Some(path) = current_dir {
        command.current_dir(path);
    }
    let output = command
        .output()
        .map_err(|error| RepoWorkflowError::ExecutionFailed(error.to_string()))?;

    if output.status.success() {
        return Ok(());
    }

    let joined_args = args.join(" ");
    let stderr = normalize_output(&output.stderr);
    let stdout = normalize_output(&output.stdout);
    Err(RepoWorkflowError::ExecutionFailed(format!(
        "`{program} {joined_args}` failed (code {:?}) stdout=`{stdout}` stderr=`{stderr}`",
        output.status.code()
    )))
}

fn capture_command(
    program: &str,
    args: &[&str],
    current_dir: Option<&Path>,
) -> Result<String, RepoWorkflowError> {
    let mut command = Command::new(program);
    command.args(args);
    if let Some(path) = current_dir {
        command.current_dir(path);
    }
    let output = command
        .output()
        .map_err(|error| RepoWorkflowError::ExecutionFailed(error.to_string()))?;

    if output.status.success() {
        return Ok(String::from_utf8_lossy(&output.stdout).to_string());
    }

    let joined_args = args.join(" ");
    let stderr = normalize_output(&output.stderr);
    let stdout = normalize_output(&output.stdout);
    Err(RepoWorkflowError::ExecutionFailed(format!(
        "`{program} {joined_args}` failed (code {:?}) stdout=`{stdout}` stderr=`{stderr}`",
        output.status.code()
    )))
}

fn normalize_output(bytes: &[u8]) -> String {
    let value = String::from_utf8_lossy(bytes).trim().to_string();
    const MAX_LEN: usize = 280;
    if value.chars().count() <= MAX_LEN {
        return value;
    }
    value.chars().take(MAX_LEN).collect::<String>()
}

fn required_field(value: Option<String>, field_name: &str) -> Result<String, RepoWorkflowError> {
    let value = value
        .map(|entry| entry.trim().to_string())
        .filter(|entry| !entry.is_empty())
        .ok_or_else(|| {
            RepoWorkflowError::InvalidPayload(format!("missing required field `{field_name}`"))
        })?;
    Ok(value)
}

fn strip_quotes(value: &str) -> &str {
    value
        .strip_prefix('"')
        .and_then(|inner| inner.strip_suffix('"'))
        .or_else(|| {
            value
                .strip_prefix('\'')
                .and_then(|inner| inner.strip_suffix('\''))
        })
        .unwrap_or(value)
}

fn parse_list(value: &str) -> Vec<String> {
    value
        .split([',', ';'])
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

fn parse_bool(value: &str) -> Option<bool> {
    match value.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Some(true),
        "0" | "false" | "no" | "off" => Some(false),
        _ => None,
    }
}

fn extract_repo_host(repo_url: &str) -> Option<String> {
    let trimmed = repo_url.trim();
    if trimmed.is_empty() {
        return None;
    }

    if let Some((_, after_scheme)) = trimmed.split_once("://") {
        let authority = after_scheme.split('/').next().unwrap_or(after_scheme);
        let host_port = authority.rsplit('@').next().unwrap_or(authority);
        let host = host_port.split(':').next().unwrap_or(host_port).trim();
        if host.is_empty() {
            return None;
        }
        return Some(host.to_ascii_lowercase());
    }

    if let Some((before_colon, _)) = trimmed.split_once(':') {
        if let Some((_, host)) = before_colon.split_once('@') {
            if !host.trim().is_empty() {
                return Some(host.trim().to_ascii_lowercase());
            }
        }
    }

    if trimmed.starts_with('/')
        || trimmed.starts_with("./")
        || trimmed.starts_with("../")
        || trimmed.contains('\\')
        || trimmed.chars().nth(1) == Some(':')
    {
        return Some("local".to_string());
    }

    None
}

fn default_workspace_root() -> PathBuf {
    if let Some(base) = AppConfig::default_data_dir() {
        return base.join("repo-workflow");
    }
    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(".temp")
        .join("repo-workflow")
}

fn current_unix_timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsStr;
    use std::fs;
    use std::sync::{Mutex, OnceLock};
    use tempfile::TempDir;

    fn test_policy() -> RepoWorkflowPolicy {
        RepoWorkflowPolicy {
            enabled: true,
            allowed_hosts: vec!["github.com".to_string(), "local".to_string()],
            allowed_command_prefixes: vec!["cargo ".to_string(), "git status".to_string()],
            allow_push: false,
            allow_pull_request: false,
            workspace_root: PathBuf::from(".temp/repo-workflow-tests"),
        }
    }

    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    fn with_env_vars<F>(entries: &[(&str, Option<&str>)], f: F)
    where
        F: FnOnce(),
    {
        let _guard = env_lock().lock().unwrap_or_else(|error| error.into_inner());
        let previous: Vec<(&str, Option<String>)> = entries
            .iter()
            .map(|(key, _)| (*key, std::env::var(key).ok()))
            .collect();

        for (key, value) in entries {
            match value {
                Some(raw) => std::env::set_var(key, raw),
                None => std::env::remove_var(key),
            }
        }

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(f));

        for (key, value) in previous {
            if let Some(raw) = value {
                std::env::set_var(key, raw);
            } else {
                std::env::remove_var(key);
            }
        }

        if let Err(payload) = result {
            std::panic::resume_unwind(payload);
        }
    }

    fn git_available() -> bool {
        Command::new("git")
            .arg("--version")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    fn run_git<I, S>(args: I, current_dir: Option<&Path>)
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let mut command = Command::new("git");
        command.args(args);
        if let Some(path) = current_dir {
            command.current_dir(path);
        }
        let output = command.output().expect("git command should spawn");
        assert!(
            output.status.success(),
            "git command failed: stdout=`{}` stderr=`{}`",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    fn seed_non_bare_repo(root: &Path) -> PathBuf {
        let repo_dir = root.join("origin-repo");
        fs::create_dir_all(&repo_dir).expect("repo dir should be created");
        run_git(["init"], Some(repo_dir.as_path()));
        fs::write(repo_dir.join("README.md"), "seed\n").expect("seed file should be written");
        run_git(["add", "-A"], Some(repo_dir.as_path()));
        run_git(
            [
                "-c",
                "user.email=test@example.com",
                "-c",
                "user.name=NTK Test",
                "commit",
                "-m",
                "init",
            ],
            Some(repo_dir.as_path()),
        );
        repo_dir
    }

    fn seed_bare_origin_repo(root: &Path) -> PathBuf {
        let bare_origin = root.join("origin-bare.git");
        run_git(
            ["init", "--bare", bare_origin.to_string_lossy().as_ref()],
            None,
        );

        let seed_clone = root.join("seed-clone");
        run_git(
            [
                "clone",
                bare_origin.to_string_lossy().as_ref(),
                seed_clone.to_string_lossy().as_ref(),
            ],
            None,
        );
        fs::write(seed_clone.join("README.md"), "seed\n").expect("seed file should be written");
        run_git(["add", "-A"], Some(seed_clone.as_path()));
        run_git(
            [
                "-c",
                "user.email=test@example.com",
                "-c",
                "user.name=NTK Test",
                "commit",
                "-m",
                "init",
            ],
            Some(seed_clone.as_path()),
        );
        run_git(["push", "origin", "HEAD"], Some(seed_clone.as_path()));

        bare_origin
    }

    #[test]
    fn repo_workflow_policy_from_env_reads_supported_fields() {
        with_env_vars(
            &[
                (NTK_REPO_WORKFLOW_ENABLED_ENV, Some("true")),
                (
                    NTK_REPO_WORKFLOW_ALLOWED_HOSTS_ENV,
                    Some("github.com;gitlab.com"),
                ),
                (
                    NTK_REPO_WORKFLOW_ALLOWED_COMMANDS_ENV,
                    Some("cargo test,git status"),
                ),
                (NTK_REPO_WORKFLOW_ALLOW_PUSH_ENV, Some("yes")),
                (NTK_REPO_WORKFLOW_ALLOW_PR_ENV, Some("on")),
                (
                    NTK_REPO_WORKFLOW_BASE_DIR_ENV,
                    Some(".temp/custom-repo-workflow-root"),
                ),
            ],
            || {
                let policy = RepoWorkflowPolicy::from_env();
                assert!(policy.enabled);
                assert_eq!(policy.allowed_hosts, vec!["github.com", "gitlab.com"]);
                assert_eq!(
                    policy.allowed_command_prefixes,
                    vec!["cargo test", "git status"]
                );
                assert!(policy.allow_push);
                assert!(policy.allow_pull_request);
                assert!(policy
                    .workspace_root
                    .ends_with(PathBuf::from(".temp/custom-repo-workflow-root")));
            },
        );
    }

    #[test]
    fn parse_repo_workflow_payload_supports_json_format() {
        let payload = r#"{
            "repo":"https://github.com/acme/demo.git",
            "branch":"feature/chatops",
            "command":"cargo test"
        }"#;
        let parsed = parse_repo_workflow_payload(payload).expect("json payload should parse");
        assert_eq!(parsed.repo_url, "https://github.com/acme/demo.git");
        assert_eq!(parsed.branch_name, "feature/chatops");
        assert_eq!(parsed.command, "cargo test");
        assert!(parsed.dry_run);
    }

    #[test]
    fn parse_repo_workflow_payload_supports_json_aliases_and_flags() {
        let payload = r#"{
            "repository":"https://github.com/acme/demo.git",
            "branch_name":"feature/aliases",
            "run":"cargo test --all",
            "commit_message":"feat: aliases",
            "push":true,
            "pull_request":true,
            "dry_run":false
        }"#;
        let parsed = parse_repo_workflow_payload(payload).expect("json payload should parse");
        assert_eq!(parsed.branch_name, "feature/aliases");
        assert_eq!(parsed.command, "cargo test --all");
        assert_eq!(parsed.commit_message, "feat: aliases");
        assert!(parsed.push);
        assert!(parsed.open_pull_request);
        assert!(!parsed.dry_run);
    }

    #[test]
    fn parse_repo_workflow_payload_supports_key_value_format() {
        let payload =
            "repo=./repo-origin.git;branch=feature/one;command=git status --short;dry_run=true";
        let parsed = parse_repo_workflow_payload(payload).expect("kv payload should parse");
        assert_eq!(parsed.repo_url, "./repo-origin.git");
        assert_eq!(parsed.branch_name, "feature/one");
        assert_eq!(parsed.command, "git status --short");
        assert!(parsed.dry_run);
    }

    #[test]
    fn parse_repo_workflow_payload_rejects_empty_payload() {
        let result = parse_repo_workflow_payload("   ");
        assert!(matches!(result, Err(RepoWorkflowError::InvalidPayload(_))));
    }

    #[test]
    fn parse_repo_workflow_payload_rejects_invalid_key_value_segment() {
        let result =
            parse_repo_workflow_payload("repo=https://github.com/acme/demo.git;branch=feature/a");
        assert!(matches!(result, Err(RepoWorkflowError::InvalidPayload(_))));
    }

    #[test]
    fn parse_repo_workflow_payload_trims_quoted_values() {
        let payload = "repo=\"./repo-origin.git\";branch='feature/quoted';command=\"git status --short\";dry_run='true'";
        let parsed = parse_repo_workflow_payload(payload).expect("kv payload should parse");
        assert_eq!(parsed.repo_url, "./repo-origin.git");
        assert_eq!(parsed.branch_name, "feature/quoted");
        assert_eq!(parsed.command, "git status --short");
        assert!(parsed.dry_run);
    }

    #[test]
    fn validate_repo_workflow_request_denies_non_allowlisted_host() {
        let policy = test_policy();
        let request = RepoWorkflowRequest {
            repo_url: "https://example.com/private/repo.git".to_string(),
            branch_name: "feature/blocked".to_string(),
            command: "cargo test".to_string(),
            commit_message: "msg".to_string(),
            push: false,
            open_pull_request: false,
            dry_run: true,
        };
        let result = validate_repo_workflow_request(&request, &policy);
        assert!(matches!(result, Err(RepoWorkflowError::PolicyDenied(_))));
    }

    #[test]
    fn validate_repo_workflow_request_denies_when_policy_is_disabled() {
        let mut policy = test_policy();
        policy.enabled = false;
        let request = RepoWorkflowRequest {
            repo_url: "https://github.com/acme/demo.git".to_string(),
            branch_name: "feature/blocked".to_string(),
            command: "cargo test".to_string(),
            commit_message: "msg".to_string(),
            push: false,
            open_pull_request: false,
            dry_run: true,
        };
        let result = validate_repo_workflow_request(&request, &policy);
        assert!(matches!(result, Err(RepoWorkflowError::PolicyDenied(_))));
    }

    #[test]
    fn validate_repo_workflow_request_denies_without_allowed_prefixes() {
        let mut policy = test_policy();
        policy.allowed_command_prefixes.clear();
        let request = RepoWorkflowRequest {
            repo_url: "https://github.com/acme/demo.git".to_string(),
            branch_name: "feature/blocked".to_string(),
            command: "cargo test".to_string(),
            commit_message: "msg".to_string(),
            push: false,
            open_pull_request: false,
            dry_run: true,
        };
        let result = validate_repo_workflow_request(&request, &policy);
        assert!(matches!(result, Err(RepoWorkflowError::PolicyDenied(_))));
    }

    #[test]
    fn validate_repo_workflow_request_denies_push_when_not_allowed() {
        let policy = test_policy();
        let request = RepoWorkflowRequest {
            repo_url: "https://github.com/acme/demo.git".to_string(),
            branch_name: "feature/push".to_string(),
            command: "cargo test".to_string(),
            commit_message: "msg".to_string(),
            push: true,
            open_pull_request: false,
            dry_run: true,
        };
        let result = validate_repo_workflow_request(&request, &policy);
        assert!(matches!(result, Err(RepoWorkflowError::PolicyDenied(_))));
    }

    #[test]
    fn validate_repo_workflow_request_denies_pull_request_without_push() {
        let mut policy = test_policy();
        policy.allow_pull_request = true;
        let request = RepoWorkflowRequest {
            repo_url: "https://github.com/acme/demo.git".to_string(),
            branch_name: "feature/pr".to_string(),
            command: "cargo test".to_string(),
            commit_message: "msg".to_string(),
            push: false,
            open_pull_request: true,
            dry_run: true,
        };
        let result = validate_repo_workflow_request(&request, &policy);
        assert!(matches!(result, Err(RepoWorkflowError::InvalidPayload(_))));
    }

    #[test]
    fn validate_repo_workflow_request_accepts_local_repo_paths() {
        let policy = test_policy();
        let request = RepoWorkflowRequest {
            repo_url: r"C:\repos\demo.git".to_string(),
            branch_name: "feature/local".to_string(),
            command: "git status --short".to_string(),
            commit_message: "msg".to_string(),
            push: false,
            open_pull_request: false,
            dry_run: true,
        };
        let result = validate_repo_workflow_request(&request, &policy);
        assert!(result.is_ok());
    }

    #[test]
    fn execute_repo_workflow_dry_run_returns_plan_without_execution() {
        let policy = test_policy();
        let request = parse_repo_workflow_payload(
            "repo=https://github.com/acme/demo.git;branch=feature/plan;command=cargo test;dry_run=true",
        )
        .expect("payload should parse");

        let result = execute_repo_workflow(&request, &policy).expect("dry run should succeed");
        assert!(!result.executed);
        assert!(result.summary.contains("Dry-run"));
        assert!(result
            .plan
            .steps
            .iter()
            .any(|step| step.starts_with("git clone --depth 1")));
    }

    #[test]
    fn execute_repo_workflow_executes_without_changes_and_skips_commit_flow() {
        if !git_available() {
            return;
        }

        let temp = TempDir::new().expect("tempdir should be created");
        let origin_repo = seed_non_bare_repo(temp.path());
        let workspace_root = temp.path().join("workspaces");
        let policy = RepoWorkflowPolicy {
            enabled: true,
            allowed_hosts: vec!["local".to_string()],
            allowed_command_prefixes: vec!["git status".to_string()],
            allow_push: false,
            allow_pull_request: false,
            workspace_root,
        };
        let request = RepoWorkflowRequest {
            repo_url: origin_repo.to_string_lossy().to_string(),
            branch_name: "feature/no-change".to_string(),
            command: "git status --short".to_string(),
            commit_message: "chore: no change".to_string(),
            push: false,
            open_pull_request: false,
            dry_run: false,
        };

        let result = execute_repo_workflow(&request, &policy)
            .expect("execution without changes should pass");
        assert!(result.executed);
        assert!(result.summary.contains("no file changes"));
    }

    #[test]
    fn execute_repo_workflow_executes_commit_and_push_to_bare_origin() {
        if !git_available() {
            return;
        }

        let temp = TempDir::new().expect("tempdir should be created");
        let bare_origin = seed_bare_origin_repo(temp.path());
        let workspace_root = temp.path().join("workspaces");
        let policy = RepoWorkflowPolicy {
            enabled: true,
            allowed_hosts: vec!["local".to_string()],
            allowed_command_prefixes: vec!["git config".to_string()],
            allow_push: true,
            allow_pull_request: false,
            workspace_root,
        };
        let request = RepoWorkflowRequest {
            repo_url: bare_origin.to_string_lossy().to_string(),
            branch_name: "feature/push".to_string(),
            command:
                "git config user.email test@example.com && git config user.name ntk-test && echo workflow>workflow.txt"
                    .to_string(),
            commit_message: "chore: add workflow artifact".to_string(),
            push: true,
            open_pull_request: false,
            dry_run: false,
        };

        let result = execute_repo_workflow(&request, &policy)
            .expect("execution with commit and push should pass");
        assert!(result.executed);
        assert!(result.summary.contains("executed successfully"));

        let verification = Command::new("git")
            .args([
                "--git-dir",
                bare_origin.to_string_lossy().as_ref(),
                "show-ref",
                "--verify",
                "refs/heads/feature/push",
            ])
            .output()
            .expect("git should spawn");
        assert!(
            verification.status.success(),
            "expected pushed branch reference to exist"
        );
    }

    #[test]
    fn run_shell_command_reports_non_zero_exit_code() {
        let temp = TempDir::new().expect("tempdir should be created");
        let failure = run_shell_command("definitely_missing_command_ntk", temp.path());
        assert!(matches!(
            failure,
            Err(RepoWorkflowError::ExecutionFailed(_))
        ));
    }

    #[test]
    fn run_command_reports_spawn_failure_for_missing_program() {
        let result = run_command("definitely_missing_binary_ntk", &["--version"], None);
        assert!(matches!(result, Err(RepoWorkflowError::ExecutionFailed(_))));
    }

    #[test]
    fn capture_command_reports_spawn_failure_for_missing_program() {
        let result = capture_command("definitely_missing_binary_ntk", &["--version"], None);
        assert!(matches!(result, Err(RepoWorkflowError::ExecutionFailed(_))));
    }

    #[test]
    fn build_plan_includes_push_and_pull_request_steps_when_requested() {
        let request = RepoWorkflowRequest {
            repo_url: "https://github.com/acme/demo.git".to_string(),
            branch_name: "feature/full".to_string(),
            command: "cargo test".to_string(),
            commit_message: "msg".to_string(),
            push: true,
            open_pull_request: true,
            dry_run: true,
        };
        let plan = build_plan(&request, Path::new("/tmp/workflow"));
        assert!(plan
            .steps
            .iter()
            .any(|step| step.contains("push -u origin feature/full")));
        assert!(plan.steps.iter().any(|step| step.contains("gh pr create")));
    }

    #[test]
    fn sanitize_repo_slug_normalizes_special_characters() {
        assert_eq!(
            sanitize_repo_slug("https://github.com/acme/my.repo.git"),
            "my-repo"
        );
        assert_eq!(sanitize_repo_slug("git@github.com:acme/demo.git"), "demo");
    }

    #[test]
    fn extract_repo_host_supports_https_ssh_and_local_paths() {
        assert_eq!(
            extract_repo_host("https://github.com/acme/demo.git"),
            Some("github.com".to_string())
        );
        assert_eq!(
            extract_repo_host("git@github.com:acme/demo.git"),
            Some("github.com".to_string())
        );
        assert_eq!(
            extract_repo_host(r"C:\repos\demo.git"),
            Some("local".to_string())
        );
    }

    #[test]
    fn parse_bool_and_parse_list_cover_supported_inputs() {
        assert_eq!(parse_bool("true"), Some(true));
        assert_eq!(parse_bool("OFF"), Some(false));
        assert_eq!(parse_bool("maybe"), None);
        assert_eq!(
            parse_list("cargo test, git status ; dotnet test"),
            vec![
                "cargo test".to_string(),
                "git status".to_string(),
                "dotnet test".to_string()
            ]
        );
    }

    #[test]
    fn normalize_output_truncates_long_values() {
        let long = "x".repeat(400);
        let normalized = normalize_output(long.as_bytes());
        assert_eq!(normalized.chars().count(), 280);
    }

    #[test]
    fn allocate_workspace_path_uses_slug_suffix() {
        let root = PathBuf::from(".temp/repo-workflow-tests");
        let path = allocate_workspace_path(root.as_path(), "https://github.com/acme/demo.git");
        let path_string = path.display().to_string();
        assert!(path_string.contains("demo"));
    }
}
