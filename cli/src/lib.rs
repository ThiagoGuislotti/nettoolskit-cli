
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use owo_colors::OwoColorize;
use std::io;

pub mod palette;
pub mod commands;
pub mod input;

use palette::CommandPalette;
use nettoolskit_ui::{clear_terminal, print_logo, PRIMARY_COLOR};
use nettoolskit_async_utils::with_timeout;
use nettoolskit_otel::{init_tracing, Metrics};
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

impl From<ExitStatus> for std::process::ExitCode {
    fn from(status: ExitStatus) -> Self {
        match status {
            ExitStatus::Success => std::process::ExitCode::SUCCESS,
            ExitStatus::Error => std::process::ExitCode::FAILURE,
            ExitStatus::Interrupted => std::process::ExitCode::from(130),
        }
    }
}

/// Launch the interactive CLI mode
pub async fn interactive_mode() -> ExitStatus {
    // Initialize telemetry
    if let Err(e) = init_tracing(false) {
        eprintln!("Warning: Failed to initialize tracing: {}", e);
    }

    let mut metrics = Metrics::new();
    metrics.increment_counter("cli_sessions_started");

    clear_terminal().unwrap_or(());

    // Use async-utils for timeout instead of direct tokio
    if let Err(_) = with_timeout(
        std::time::Duration::from_millis(50),
        tokio::time::sleep(std::time::Duration::from_millis(50))
    ).await {
        // Timeout is unlikely but we handle it gracefully
    }

    print_logo();

    let result = match run_interactive_loop().await {
        Ok(status) => {
            metrics.increment_counter("cli_sessions_completed");
            status
        },
        Err(e) => {
            metrics.increment_counter("cli_sessions_errored");
            eprintln!("{}: {}", "Error".red().bold(), e);
            ExitStatus::Error
        }
    };

    // Log metrics
    tracing::info!("CLI session ended with status: {:?}", result);
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