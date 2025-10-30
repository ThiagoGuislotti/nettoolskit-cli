
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use owo_colors::OwoColorize;
use std::io;

pub mod input;

use nettoolskit_ui::{clear_terminal, print_logo, PRIMARY_COLOR, CommandPalette};
use nettoolskit_async_utils::with_timeout;
use nettoolskit_otel::{init_tracing_with_config, TracingConfig, Metrics, Timer};
use tracing::{info, warn, error};
use nettoolskit_commands::processor::{process_command, process_text, CliExitStatus};
use input::{read_line_with_palette, InputResult};

/// Exit status for the CLI
#[derive(Debug, Clone, Copy)]
pub enum ExitStatus {
    Success,
    Error,
    Interrupted,
}

impl From<CliExitStatus> for ExitStatus {
    fn from(status: CliExitStatus) -> Self {
        match status {
            CliExitStatus::Success => ExitStatus::Success,
            CliExitStatus::Error => ExitStatus::Error,
            CliExitStatus::Interrupted => ExitStatus::Interrupted,
        }
    }
}

impl From<nettoolskit_commands::ExitStatus> for ExitStatus {
    fn from(status: nettoolskit_commands::ExitStatus) -> Self {
        match status {
            nettoolskit_commands::ExitStatus::Success => ExitStatus::Success,
            nettoolskit_commands::ExitStatus::Error => ExitStatus::Error,
            nettoolskit_commands::ExitStatus::Interrupted => ExitStatus::Interrupted,
        }
    }
}

impl From<ExitStatus> for std::process::ExitCode {
    fn from(status: ExitStatus) -> Self {
        match status {
            ExitStatus::Success => std::process::ExitCode::SUCCESS,
            ExitStatus::Error => std::process::ExitCode::FAILURE,
            ExitStatus::Interrupted => std::process::ExitCode::from(130),
        }
    }
}

impl From<ExitStatus> for i32 {
    fn from(status: ExitStatus) -> Self {
        match status {
            ExitStatus::Success => 0,
            ExitStatus::Error => 1,
            ExitStatus::Interrupted => 130,
        }
    }
}

/// Launch the interactive CLI mode
pub async fn interactive_mode() -> ExitStatus {
    // Initialize telemetry with development configuration
    let tracing_config = TracingConfig {
        verbose: false,
        with_line_numbers: true,
        ..Default::default()
    };

    if let Err(e) = init_tracing_with_config(tracing_config) {
        eprintln!("Warning: Failed to initialize tracing: {}", e);
    }

    info!("Starting NetToolsKit CLI interactive mode");

    let metrics = Metrics::new();
    metrics.increment_counter("cli_sessions_started");

    let _session_timer = Timer::start("cli_session_duration", metrics.clone());

    if let Err(e) = clear_terminal() {
        warn!(error = %e, "Failed to clear terminal");
    }

    // Use async-utils for timeout instead of direct tokio
    if let Err(_) = with_timeout(
        std::time::Duration::from_millis(50),
        tokio::time::sleep(std::time::Duration::from_millis(50))
    ).await {
        // Timeout is unlikely but we handle it gracefully
        info!("Initialization timeout completed (expected)");
    }

    info!("Displaying application logo and UI");
    print_logo();

    let result = match run_interactive_loop().await {
        Ok(status) => {
            metrics.increment_counter("cli_sessions_completed");
            info!(
                status = ?status,
                session_counters = ?metrics.counters_snapshot(),
                "CLI session completed successfully"
            );
            status
        },
        Err(e) => {
            metrics.increment_counter("cli_sessions_errored");
            error!(
                error = %e,
                session_counters = ?metrics.counters_snapshot(),
                "CLI session ended with error"
            );
            eprintln!("{}: {}", "Error".red().bold(), e);
            ExitStatus::Error
        }
    };

    // Log final metrics and shutdown
    metrics.log_summary();
    info!(final_status = ?result, "NetToolsKit CLI session ended");
    result
}

async fn run_interactive_loop() -> io::Result<ExitStatus> {
    let mut input_buffer = String::new();
    let mut palette = CommandPalette::new();

    enable_raw_mode()?;

    ctrlc::set_handler(move || {
        disable_raw_mode().unwrap_or(());
        println!("\n⚠️  {}", "Interrupted".yellow());
        std::process::exit(130);
    }).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    loop {
        print!("> ");
        std::io::Write::flush(&mut std::io::stdout())?;
        input_buffer.clear();

        match read_line_with_palette(&mut input_buffer, &mut palette).await? {
            InputResult::Command(cmd) => {
                disable_raw_mode()?;
                if cmd == "/quit" {
                    return Ok(ExitStatus::Success);
                }
                let status: ExitStatus = process_command(&cmd).await.into();
                if matches!(status, ExitStatus::Success) && cmd == "/quit" {
                    return Ok(status);
                }
                enable_raw_mode()?;
            }
            InputResult::Text(text) => {
                disable_raw_mode()?;
                process_text(&text);
                enable_raw_mode()?;
            }
            InputResult::Exit => {
                disable_raw_mode()?;
                println!("{}", "👋 Goodbye!".color(PRIMARY_COLOR));
                return Ok(ExitStatus::Interrupted);
            }
        }

        println!();
    }
}