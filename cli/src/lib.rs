use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use owo_colors::OwoColorize;
use std::io;

pub mod input;
pub mod async_executor;

use input::{read_line_with_palette, InputResult};
use nettoolskit_async_utils::with_timeout;
use nettoolskit_commands::processor::{process_command, process_text};
use nettoolskit_commands::ExitStatus;
use nettoolskit_otel::{init_tracing_with_config, Metrics, Timer, TracingConfig};
use nettoolskit_ui::{
    append_footer_log, begin_interactive_logging, clear_terminal, ensure_layout_integrity,
    print_logo, CommandPalette, TerminalLayout, PRIMARY_COLOR,
};
use tracing::{error, info, warn};

#[cfg(feature = "modern-tui")]
use nettoolskit_ui::modern::{handle_events, EventResult, Tui, EventStream, handle_events_stream};

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

    let (layout_failure_notice, _terminal_layout) = match TerminalLayout::initialize() {
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
        with_line_numbers: true,
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
    if let Err(_) = with_timeout(
        std::time::Duration::from_millis(50),
        tokio::time::sleep(std::time::Duration::from_millis(50)),
    )
    .await
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
    let mut palette = CommandPalette::new();

    #[cfg(feature = "modern-tui")]
    {
        if std::env::var("NTK_USE_MODERN_TUI").is_ok() {
            info!("Using modern TUI with 16ms event polling");
            return run_modern_loop(&mut input_buffer, &mut palette).await;
        }
    }

    info!("Using legacy TUI with 50ms event polling");
    run_legacy_loop(&mut input_buffer, &mut palette).await
}

/// Check if a command should use async execution
#[cfg(feature = "modern-tui")]
fn is_async_command(cmd: &str) -> bool {
    std::env::var("NTK_USE_ASYNC_EXECUTOR").is_ok()
        && (cmd.starts_with("/check-async") || cmd.starts_with("/list-async"))
}

#[cfg(feature = "modern-tui")]
async fn run_modern_loop(
    input_buffer: &mut String,
    palette: &mut CommandPalette,
) -> io::Result<ExitStatus> {
    // Check if event stream mode is enabled (Phase 1.3)
    let use_event_stream = std::env::var("NTK_USE_EVENT_STREAM").is_ok();

    if use_event_stream {
        info!("Using event stream (Phase 1.3 - zero CPU idle)");
        run_modern_loop_with_stream(input_buffer, palette).await
    } else {
        info!("Using event polling (Phase 1.2 - 16ms polling)");
        run_modern_loop_with_polling(input_buffer, palette).await
    }
}

/// Modern loop with event stream (Phase 1.3) - zero CPU when idle
#[cfg(feature = "modern-tui")]
async fn run_modern_loop_with_stream(
    input_buffer: &mut String,
    palette: &mut CommandPalette,
) -> io::Result<ExitStatus> {
    use nettoolskit_commands::processor_async::process_async_command;

    let mut tui = Tui::new()?;

    // Print initial prompt before entering raw mode
    print!("> ");
    std::io::Write::flush(&mut std::io::stdout())?;

    tui.enter()?;

    let mut events = EventStream::new();

    let exit_status = loop {
        match handle_events_stream(input_buffer, palette, &mut events).await? {
            EventResult::Command(cmd) => {
                tui.exit()?;
                if cmd == "/quit" {
                    break ExitStatus::Success;
                }

                // Check if async execution is enabled and command supports it
                let status = if is_async_command(&cmd) {
                    // Use async executor with progress
                    match process_async_command(&cmd).await {
                        Ok(output) => {
                            println!("\n{}", output);
                            ExitStatus::Success
                        }
                        Err(e) => {
                            eprintln!("\n{}: {}", "Error".red().bold(), e);
                            ExitStatus::Error
                        }
                    }
                } else {
                    // Use standard synchronous execution
                    process_command(&cmd).await.into()
                };

                if matches!(status, ExitStatus::Success) && cmd == "/quit" {
                    break status;
                }
                if matches!(status, ExitStatus::Interrupted) {
                    break status;
                }

                // Print new prompt BEFORE re-entering raw mode
                print!("\n> ");
                std::io::Write::flush(&mut std::io::stdout())?;
                input_buffer.clear();

                tui.enter()?;
            }
            EventResult::Text(text) => {
                tui.exit()?;
                process_text(&text);

                // Print new prompt BEFORE re-entering raw mode
                print!("\n> ");
                std::io::Write::flush(&mut std::io::stdout())?;
                input_buffer.clear();

                tui.enter()?;
            }
            EventResult::Exit => {
                tui.exit()?;
                println!("{}", "👋 Goodbye!".color(PRIMARY_COLOR));
                break ExitStatus::Interrupted;
            }
            EventResult::Continue => {
                // Keep looping
            }
        }
    };

    tui.exit()?;
    Ok(exit_status)
}

/// Modern loop with polling (Phase 1.2) - 16ms polling
#[cfg(feature = "modern-tui")]
async fn run_modern_loop_with_polling(
    input_buffer: &mut String,
    palette: &mut CommandPalette,
) -> io::Result<ExitStatus> {
    use nettoolskit_commands::processor_async::process_async_command;

    let mut tui = Tui::new()?;

    // Print initial prompt before entering raw mode
    print!("> ");
    std::io::Write::flush(&mut std::io::stdout())?;

    tui.enter()?;

    let exit_status = loop {
        match handle_events(input_buffer, palette).await? {
            EventResult::Command(cmd) => {
                tui.exit()?;
                if cmd == "/quit" {
                    break ExitStatus::Success;
                }

                // Check if async execution is enabled and command supports it
                let status = if is_async_command(&cmd) {
                    // Use async executor with progress
                    match process_async_command(&cmd).await {
                        Ok(output) => {
                            println!("\n{}", output);
                            ExitStatus::Success
                        }
                        Err(e) => {
                            eprintln!("\n{}: {}", "Error".red().bold(), e);
                            ExitStatus::Error
                        }
                    }
                } else {
                    // Use standard synchronous execution
                    process_command(&cmd).await.into()
                };

                if matches!(status, ExitStatus::Success) && cmd == "/quit" {
                    break status;
                }
                if matches!(status, ExitStatus::Interrupted) {
                    break status;
                }

                // Print new prompt BEFORE re-entering raw mode
                print!("\n> ");
                std::io::Write::flush(&mut std::io::stdout())?;
                input_buffer.clear();

                tui.enter()?;
            }
            EventResult::Text(text) => {
                tui.exit()?;
                process_text(&text);

                // Print new prompt BEFORE re-entering raw mode
                print!("\n> ");
                std::io::Write::flush(&mut std::io::stdout())?;
                input_buffer.clear();

                tui.enter()?;
            }
            EventResult::Exit => {
                tui.exit()?;
                println!("{}", "👋 Goodbye!".color(PRIMARY_COLOR));
                break ExitStatus::Interrupted;
            }
            EventResult::Continue => {
                // Keep looping
            }
        }
    };

    tui.exit()?;
    Ok(exit_status)
}

async fn run_legacy_loop(
    input_buffer: &mut String,
    palette: &mut CommandPalette,
) -> io::Result<ExitStatus> {
    let mut raw_mode = RawModeGuard::new()?;

    ctrlc::set_handler(move || {
        disable_raw_mode().unwrap_or(());
        println!("\n⚠️  {}", "Interrupted".yellow());
        std::process::exit(130);
    })
    .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    loop {
        raw_mode.enable()?;
        print!("> ");
        std::io::Write::flush(&mut std::io::stdout())?;
        input_buffer.clear();

        match read_line_with_palette(input_buffer, palette).await? {
            InputResult::Command(cmd) => {
                raw_mode.disable()?;
                if cmd == "/quit" {
                    return Ok(ExitStatus::Success);
                }
                let status: ExitStatus = process_command(&cmd).await.into();
                if matches!(status, ExitStatus::Success) && cmd == "/quit" {
                    return Ok(status);
                }
                if matches!(status, ExitStatus::Interrupted) {
                    return Ok(status);
                }
                raw_mode.enable()?;
                // NOTE: Layout guard kept for commands as they may modify terminal state
                ensure_layout_guard();
            }
            InputResult::Text(text) => {
                raw_mode.disable()?;
                process_text(&text);
                raw_mode.enable()?;
                // NOTE: Commented out to prevent screen clearing after text input
                // This was causing input to disappear after Enter
                // ensure_layout_guard();
            }
            InputResult::Exit => {
                raw_mode.disable()?;
                println!("{}", "👋 Goodbye!".color(PRIMARY_COLOR));
                return Ok(ExitStatus::Interrupted);
            }
        }

        println!();
    }
}

fn ensure_layout_guard() {
    if let Err(err) = ensure_layout_integrity() {
        warn!(error = %err, "Failed to enforce terminal layout integrity");
        let _ = append_footer_log(&format!(
            "Warning: failed to ensure layout integrity: {err}"
        ));
    }
}
