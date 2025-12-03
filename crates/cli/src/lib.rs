//! NetToolsKit CLI application entry point and orchestration
//!
//! This crate coordinates the main CLI application logic, including:
//! - Interactive command input and execution
//! - Terminal event handling and rendering
//! - Integration between UI, commands, and telemetry layers
//!
//! # Features
//!
//! - **modern-tui**: Enable modern ratatui-based terminal interface
//!
//! # Architecture
//!
//! The CLI follows a layered architecture:
//! - Input layer: Handles user input and command palette
//! - Execution layer: Async command processing with progress tracking
//! - Rendering layer: Terminal output and layout management

use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use owo_colors::OwoColorize;
use std::io;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub mod display;
pub mod events;
pub mod input;

use display::print_logo;
use input::{read_line, InputResult};
use nettoolskit_core::async_utils::with_timeout;
use nettoolskit_commands::{process_command, process_text, ExitStatus};
use nettoolskit_otel::{init_tracing_with_config, Metrics, Timer, TracingConfig};
use nettoolskit_ui::{
    append_footer_log, begin_interactive_logging, clear_terminal, ensure_layout_integrity,
    render_prompt, CommandPalette, TerminalLayout, PRIMARY_COLOR,
};
use tracing::{error, info, warn};

struct RawModeGuard {
    active: bool,
}

impl RawModeGuard {
    fn new() -> io::Result<Self> {
        enable_raw_mode()?;
        Ok(Self { active: true })
    }

    fn enable(&mut self) -> io::Result<()> {
        if !self.active {
            enable_raw_mode()?;
            self.active = true;
        }
        Ok(())
    }

    fn disable(&mut self) -> io::Result<()> {
        if self.active {
            disable_raw_mode()?;
            self.active = false;
        }
        Ok(())
    }
}

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        if self.active {
            let _ = disable_raw_mode();
            self.active = false;
        }
    }
}

/// Launch the interactive CLI mode
pub async fn interactive_mode(verbose: bool) -> ExitStatus {
    let mut log_guard = begin_interactive_logging();

    let (layout_failure_notice, _terminal_layout) = match TerminalLayout::initialize(Some(print_logo)) {
        Ok(layout) => (None, Some(layout)),
        Err(e) => {
            let failure_message = format!("Failed to initialize terminal layout: {e}");
            let _ = append_footer_log(&format!("Warning: {failure_message}"));
            log_guard.deactivate();
            if let Err(clear_error) = clear_terminal() {
                let clear_failure = format!("Failed to clear terminal: {clear_error}");
                let _ = append_footer_log(&format!("Warning: {clear_failure}"));
            }
            print_logo();
            (Some(failure_message), None)
        }
    };

    // Initialize telemetry with development configuration
    let tracing_config = TracingConfig {
        verbose,
        with_line_numbers: false,
        interactive_mode: true, // Suppress tracing fmt output in interactive mode
        ..Default::default()
    };

    if let Err(e) = init_tracing_with_config(tracing_config) {
        let message = format!("Failed to initialize tracing: {}", e);
        let _ = append_footer_log(&format!("Warning: {message}"));
    }

    if let Some(failure_message) = layout_failure_notice {
        warn!("{failure_message}");
    }

    let metrics = Metrics::new();
    metrics.increment_counter("cli_sessions_started");

    let _session_timer = Timer::start("cli_session_duration", metrics.clone());

    // Use async-utils for timeout instead of direct tokio
    if (with_timeout(
        std::time::Duration::from_millis(50),
        tokio::time::sleep(std::time::Duration::from_millis(50)),
    )
    .await)
        .is_err()
    {
        // Timeout is unlikely but we handle it gracefully
        info!("Initialization timeout completed (expected)");
    }

    info!("Starting NetToolsKit CLI interactive mode");
    info!("Displaying application logo and UI");

    let result = match run_interactive_loop().await {
        Ok(status) => {
            metrics.increment_counter("cli_sessions_completed");
            info!(
                status = ?status,
                session_counters = ?metrics.counters_snapshot(),
                "CLI session completed successfully"
            );
            status
        }
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

    info!("Starting interactive loop");
    run_input_loop(&mut input_buffer).await
}

async fn run_input_loop(input_buffer: &mut String) -> io::Result<ExitStatus> {
    let mut raw_mode = RawModeGuard::new()?;
    let interrupted = Arc::new(AtomicBool::new(false));
    let interrupted_fallback = interrupted.clone();

    // Fallback handler for Ctrl+C when raw mode is not active
    ctrlc::set_handler(move || {
        interrupted_fallback.store(true, Ordering::SeqCst);
    })
    .map_err(io::Error::other)?;

    loop {
        raw_mode.enable()?;
        render_prompt()?;
        input_buffer.clear();

        match read_line(input_buffer, &interrupted).await? {
            InputResult::ShowMenu => {
                // User typed "/" - show menu immediately
                if let Some(selected) = show_main_menu() {
                    raw_mode.disable()?;

                    // Check if user selected quit command
                    if is_quit_command(&selected) {
                        print_goodbye();
                        return Ok(ExitStatus::Success);
                    }

                    let status: ExitStatus = process_command(&selected).await;
                    if matches!(status, ExitStatus::Interrupted) {
                        return Ok(status);
                    }
                    raw_mode.enable()?;
                    ensure_layout_guard();
                }
                // Clear buffer and prompt for next input
                input_buffer.clear();
                println!();
                continue;
            }
            InputResult::Command(cmd) => {
                raw_mode.disable()?;

                // Check if user typed quit command
                if is_quit_command(&cmd) {
                    print_goodbye();
                    return Ok(ExitStatus::Success);
                }

                let status: ExitStatus = process_command(&cmd).await;
                if matches!(status, ExitStatus::Interrupted) {
                    return Ok(status);
                }
                raw_mode.enable()?;
                // NOTE: Layout guard kept for commands as they may modify terminal state
                ensure_layout_guard();
            }
            InputResult::Text(text) => {
                raw_mode.disable()?;
                std::mem::drop(process_text(&text));
                raw_mode.enable()?;
                // NOTE: Commented out to prevent screen clearing after text input
                // This was causing input to disappear after Enter
                // ensure_layout_guard();
            }
            InputResult::Exit => {
                raw_mode.disable()?;
                // Check if this was triggered by Ctrl+C
                if interrupted.load(Ordering::SeqCst) {
                    println!("\n⚠️  {}", "Interrupted".yellow());
                } else {
                    print_goodbye();
                }
                return Ok(ExitStatus::Success);
            }
        }

        println!();
    }
}

/// Show main menu when user types "/"
fn show_main_menu() -> Option<String> {
    let menu_entries = nettoolskit_commands::menu_entries();
    let current_dir = std::env::current_dir()
        .ok()
        .and_then(|p| p.to_str().map(String::from))
        .unwrap_or_else(|| String::from("."));

    let palette = CommandPalette::new(menu_entries)
        .with_title("NetToolsKit Commands")
        .with_subtitle("Select a command to execute")
        .with_directory(current_dir);

    palette.show()
}

/// Check if a command is a quit/exit command
fn is_quit_command(cmd: &str) -> bool {
    cmd == "/quit" || cmd == "quit"
}

/// Print goodbye message to user
fn print_goodbye() {
    println!("{}", "👋 Goodbye!".color(PRIMARY_COLOR));
}

fn ensure_layout_guard() {
    if let Err(err) = ensure_layout_integrity() {
        warn!(error = %err, "Failed to enforce terminal layout integrity");
        let _ = append_footer_log(&format!(
            "Warning: failed to ensure layout integrity: {err}"
        ));
    }
}
