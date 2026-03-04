//! NetToolsKit CLI binary entry point.

use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::Shell;
use nettoolskit_cli::{interactive_mode, InteractiveOptions};
use nettoolskit_core::{AppConfig, ColorMode, CommandEntry, UnicodeMode};
use nettoolskit_orchestrator::ExitStatus;
use nettoolskit_otel::{
    init_tracing_with_config, next_correlation_id, shutdown_tracing, TracingConfig,
};
use nettoolskit_ui::{set_color_override, set_unicode_override, ColorLevel};
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tracing::{info, info_span};

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

    /// Translate templates between programming languages
    Translate {
        /// Source language identifier
        #[clap(long)]
        from: String,

        /// Target language identifier
        #[clap(long)]
        to: String,

        /// Template file path to translate
        path: String,
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
        #[clap(long, default_value = "0.0.0.0")]
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

#[derive(Debug, Serialize)]
struct ServiceTaskSubmitResponse {
    accepted: bool,
    exit_status: String,
}

#[derive(Debug, Serialize)]
struct ServiceHealthResponse {
    status: String,
    runtime_mode: String,
    uptime_seconds: u64,
    version: String,
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
            Commands::Translate { from, to, path } => {
                let request = nettoolskit_translate::TranslateRequest { from, to, path };
                nettoolskit_translate::handle_translate(request).await
            }
            Commands::Completions { shell } => {
                clap_complete::generate(shell, &mut Cli::command(), "ntk", &mut std::io::stdout());
                ExitStatus::Success
            }
            Commands::Service { host, port } => run_service_mode(host, port).await,
        }
    }
}

fn parse_http_request_line(request: &str) -> Option<(&str, &str)> {
    let line = request.lines().next()?.trim();
    let mut parts = line.split_whitespace();
    let method = parts.next()?;
    let path = parts.next()?;
    Some((method, path))
}

fn request_body(request: &str) -> &str {
    request
        .split_once("\r\n\r\n")
        .map_or("", |(_, body)| body.trim())
}

fn build_http_response(status: &str, content_type: &str, body: &str) -> String {
    format!(
        "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    )
}

fn exit_status_label(status: ExitStatus) -> &'static str {
    match status {
        ExitStatus::Success => "success",
        ExitStatus::Error => "error",
        ExitStatus::Interrupted => "interrupted",
    }
}

async fn write_json_response<T: Serialize>(
    stream: &mut TcpStream,
    status: &str,
    payload: &T,
) -> std::io::Result<()> {
    let body = serde_json::to_string(payload).unwrap_or_else(|_| {
        "{\"accepted\":false,\"exit_status\":\"error\",\"message\":\"serialization\"}".to_string()
    });
    let response = build_http_response(status, "application/json", &body);
    stream.write_all(response.as_bytes()).await?;
    stream.flush().await
}

async fn write_plain_response(
    stream: &mut TcpStream,
    status: &str,
    body: &str,
) -> std::io::Result<()> {
    let response = build_http_response(status, "text/plain; charset=utf-8", body);
    stream.write_all(response.as_bytes()).await?;
    stream.flush().await
}

async fn handle_service_connection(
    mut stream: TcpStream,
    started_at: std::time::Instant,
) -> std::io::Result<()> {
    let mut buffer = vec![0_u8; 32 * 1024];
    let bytes_read = stream.read(&mut buffer).await?;
    if bytes_read == 0 {
        return Ok(());
    }

    let request = String::from_utf8_lossy(&buffer[..bytes_read]).to_string();
    let Some((method, path)) = parse_http_request_line(&request) else {
        write_plain_response(&mut stream, "400 Bad Request", "invalid request line").await?;
        return Ok(());
    };

    match (method, path) {
        ("GET", "/") => {
            write_plain_response(
                &mut stream,
                "200 OK",
                "NetToolsKit service mode is running.\nUse GET /health.",
            )
            .await?;
        }
        ("GET", "/health") | ("GET", "/ready") => {
            let payload = ServiceHealthResponse {
                status: "ok".to_string(),
                runtime_mode: AppConfig::load().general.runtime_mode.to_string(),
                uptime_seconds: started_at.elapsed().as_secs(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            };
            write_json_response(&mut stream, "200 OK", &payload).await?;
        }
        ("POST", "/task/submit") => {
            let body = request_body(&request);
            let parsed: Result<ServiceTaskSubmitRequest, _> = serde_json::from_str(body);
            let Ok(payload) = parsed else {
                write_plain_response(
                    &mut stream,
                    "400 Bad Request",
                    "invalid JSON payload for /task/submit",
                )
                .await?;
                return Ok(());
            };

            if payload.intent.trim().is_empty() || payload.payload.trim().is_empty() {
                write_plain_response(
                    &mut stream,
                    "400 Bad Request",
                    "intent and payload are required",
                )
                .await?;
                return Ok(());
            }

            let command = format!(
                "/task submit {} {}",
                payload.intent.trim(),
                payload.payload.trim()
            );
            let status = nettoolskit_orchestrator::process_command(&command).await;
            let response_payload = ServiceTaskSubmitResponse {
                accepted: status != ExitStatus::Error,
                exit_status: exit_status_label(status).to_string(),
            };
            let status_line = if response_payload.accepted {
                "202 Accepted"
            } else {
                "400 Bad Request"
            };
            write_json_response(&mut stream, status_line, &response_payload).await?;
        }
        _ => {
            write_plain_response(&mut stream, "404 Not Found", "not found").await?;
        }
    }

    Ok(())
}

async fn run_service_mode(host: String, port: u16) -> ExitStatus {
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
    println!("Task submit endpoint: POST /task/submit");

    let started_at = std::time::Instant::now();
    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                tokio::spawn(async move {
                    if let Err(error) = handle_service_connection(stream, started_at).await {
                        tracing::warn!(%error, "service connection handling failed");
                    }
                });
            }
            Err(error) => {
                tracing::warn!(%error, "service listener accept failed");
            }
        }
    }
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
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

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

    async fn execute_service_request(request: &str) -> String {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("test listener should bind");
        let address = listener
            .local_addr()
            .expect("test listener should provide local address");

        let request_data = request.to_string();
        let client = tokio::spawn(async move {
            let mut stream = TcpStream::connect(address)
                .await
                .expect("client should connect to test listener");
            stream
                .write_all(request_data.as_bytes())
                .await
                .expect("client should write request");
            stream
                .shutdown()
                .await
                .expect("client should shutdown write");

            let mut response = Vec::new();
            stream
                .read_to_end(&mut response)
                .await
                .expect("client should read response");
            String::from_utf8(response).expect("response should be valid UTF-8")
        });

        let (server_stream, _) = listener
            .accept()
            .await
            .expect("server should accept client");
        handle_service_connection(server_stream, std::time::Instant::now())
            .await
            .expect("service handler should process request");

        client
            .await
            .expect("client task should complete without panic")
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
    async fn service_mode_task_submit_accepts_valid_payload() {
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
}
