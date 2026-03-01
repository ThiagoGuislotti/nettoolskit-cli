//! NetToolsKit CLI application - UI layer only
//!
//! This crate provides the terminal user interface for NetToolsKit CLI:
//! - Interactive command input and display
//! - Terminal event handling
//! - Layout and rendering
//!
//! Command orchestration is handled by the `nettoolskit-orchestrator` crate.
//!
//! # Features
//!
//! - **modern-tui**: Enable modern ratatui-based terminal interface
//!
//! # Architecture
//!
//! - Input layer: User input and command palette
//! - Display layer: Terminal output and layout
//! - Event layer: Terminal events (Ctrl+C, Enter, etc.)

use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use nettoolskit_core::CommandEntry;
use owo_colors::OwoColorize;
use std::future::Future;
use std::io;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub mod display;
/// User input handling and command parsing.
pub mod input;

use display::print_logo;
use input::{read_line, InputResult};
use nettoolskit_core::async_utils::with_timeout;
use nettoolskit_orchestrator::{process_command, process_text, Command, ExitStatus};
use nettoolskit_otel::{
    init_tracing_with_config, next_correlation_id, Metrics, Timer, TracingConfig,
};
use nettoolskit_ui::{
    append_footer_log, begin_interactive_logging, clear_terminal, ensure_layout_integrity,
    render_prompt, Color, CommandPalette, TerminalLayout,
};
use tracing::{error, info, info_span, warn};

struct RawModeGuard {
    active: bool,
}

trait RawModeControl {
    fn enable(&mut self) -> io::Result<()>;
    fn disable(&mut self) -> io::Result<()>;
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

impl RawModeControl for RawModeGuard {
    fn enable(&mut self) -> io::Result<()> {
        Self::enable(self)
    }

    fn disable(&mut self) -> io::Result<()> {
        Self::disable(self)
    }
}

type ReadLineFuture<'a> = Pin<Box<dyn Future<Output = io::Result<InputResult>> + 'a>>;

/// Launch the interactive CLI mode
pub async fn interactive_mode(verbose: bool) -> ExitStatus {
    interactive_mode_with_runner(verbose, run_interactive_loop).await
}

async fn interactive_mode_with_runner<F, Fut>(verbose: bool, run_loop: F) -> ExitStatus
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = io::Result<ExitStatus>>,
{
    let session_correlation_id = next_correlation_id("session");
    let mut log_guard = begin_interactive_logging();

    let (layout_failure_notice, _terminal_layout) =
        match TerminalLayout::initialize(Some(print_logo)) {
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

    let session_span = info_span!(
        "cli.interactive_session",
        correlation_id = %session_correlation_id
    );
    let _session_scope = session_span.enter();

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

    info!(
        correlation_id = %session_correlation_id,
        "Starting NetToolsKit CLI interactive mode"
    );
    info!("Displaying application logo and UI");

    let result = match run_loop().await {
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
            use nettoolskit_ui::Color;
            eprintln!("{}: {}", "Error".color(Color::RED).bold(), e);
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

    run_input_loop_with(
        input_buffer,
        &mut raw_mode,
        interrupted,
        |buffer, interrupted| Box::pin(read_line(buffer, interrupted)),
        show_main_menu,
        render_prompt,
    )
    .await
}

async fn run_input_loop_with<R, F, M, P>(
    input_buffer: &mut String,
    raw_mode: &mut R,
    interrupted: Arc<AtomicBool>,
    mut read_line_fn: F,
    mut show_menu_fn: M,
    mut render_prompt_fn: P,
) -> io::Result<ExitStatus>
where
    R: RawModeControl,
    F: for<'a> FnMut(&'a mut String, &'a Arc<AtomicBool>) -> ReadLineFuture<'a>,
    M: FnMut() -> Option<Command>,
    P: FnMut() -> io::Result<()>,
{
    loop {
        raw_mode.enable()?;
        render_prompt_fn()?;
        input_buffer.clear();

        match read_line_fn(input_buffer, &interrupted).await? {
            InputResult::ShowMenu => {
                // User typed "/" - show menu immediately
                if let Some(selected_cmd) = show_menu_fn() {
                    raw_mode.disable()?;

                    // Check if user selected quit command
                    if selected_cmd == Command::Quit {
                        print_goodbye();
                        return Ok(ExitStatus::Success);
                    }

                    let status: ExitStatus = process_command(&selected_cmd.slash_static()).await;
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
                if let Some(Command::Quit) = nettoolskit_orchestrator::get_command(&cmd) {
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
                let _ = process_text(&text).await;
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

/// Show main menu when user types "/" - returns Command enum directly
fn show_main_menu() -> Option<Command> {
    use nettoolskit_core::MenuProvider;
    let menu_entries = Command::all_variants();
    let current_dir = std::env::current_dir()
        .ok()
        .and_then(|p| p.to_str().map(String::from))
        .unwrap_or_else(|| String::from("."));

    let palette = CommandPalette::new(menu_entries.clone())
        .with_prompt("   /")
        .with_title("NetToolsKit Commands")
        .with_subtitle("Select a command to execute")
        .with_directory(current_dir);

    palette.show().and_then(|label| {
        menu_entries
            .into_iter()
            .find(|cmd| cmd.slash_static() == label.trim())
    })
}

/// Print goodbye message to user
fn print_goodbye() {
    println!("{}", "👋 Goodbye!".color(Color::PURPLE));
}

fn ensure_layout_guard() {
    if let Err(err) = ensure_layout_integrity() {
        warn!(error = %err, "Failed to enforce terminal layout integrity");
        let _ = append_footer_log(&format!(
            "Warning: failed to ensure layout integrity: {err}"
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;
    use std::sync::Mutex;

    /// Try to create a `RawModeGuard`. Returns `None` if a terminal is not
    /// available (e.g., headless CI), allowing tests to be silently skipped
    /// rather than failing.
    fn try_guard() -> Option<RawModeGuard> {
        RawModeGuard::new().ok()
    }

    #[derive(Default)]
    struct FakeRawMode {
        active: bool,
        enable_calls: usize,
        disable_calls: usize,
    }

    impl RawModeControl for FakeRawMode {
        fn enable(&mut self) -> io::Result<()> {
            self.enable_calls += 1;
            self.active = true;
            Ok(())
        }

        fn disable(&mut self) -> io::Result<()> {
            self.disable_calls += 1;
            self.active = false;
            Ok(())
        }
    }

    fn scripted_reader(
        script: Vec<InputResult>,
    ) -> impl for<'a> FnMut(&'a mut String, &'a Arc<AtomicBool>) -> ReadLineFuture<'a> {
        let queue = Arc::new(Mutex::new(VecDeque::from(script)));
        move |_buffer, _interrupted| {
            let queue = Arc::clone(&queue);
            Box::pin(async move {
                let next = queue
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .pop_front()
                    .unwrap_or(InputResult::Exit);
                Ok(next)
            })
        }
    }

    #[test]
    fn new_guard_is_active() {
        let Some(guard) = try_guard() else { return };
        assert!(guard.active);
        drop(guard); // cleanup
    }

    #[test]
    fn disable_sets_active_false() {
        let Some(mut guard) = try_guard() else { return };
        guard.disable().expect("disable should succeed");
        assert!(!guard.active);
    }

    #[test]
    fn enable_after_disable_restores_active() {
        let Some(mut guard) = try_guard() else { return };
        guard.disable().expect("disable should succeed");
        assert!(!guard.active);
        guard.enable().expect("enable should succeed");
        assert!(guard.active);
    }

    #[test]
    fn double_disable_is_idempotent() {
        let Some(mut guard) = try_guard() else { return };
        guard.disable().expect("first disable");
        guard.disable().expect("second disable");
        assert!(!guard.active);
    }

    #[test]
    fn double_enable_is_idempotent() {
        let Some(mut guard) = try_guard() else { return };
        guard.enable().expect("first enable");
        guard.enable().expect("second enable");
        assert!(guard.active);
    }

    #[test]
    fn drop_does_not_panic() {
        let guard = try_guard();
        drop(guard); // should not panic
    }

    #[test]
    fn drop_after_disable_does_not_panic() {
        if let Some(mut guard) = try_guard() {
            let _ = guard.disable();
            drop(guard);
        }
    }

    #[tokio::test]
    async fn interactive_mode_with_runner_success_maps_to_success() {
        let status =
            interactive_mode_with_runner(false, || async { Ok(ExitStatus::Success) }).await;
        assert_eq!(status, ExitStatus::Success);
    }

    #[tokio::test]
    async fn interactive_mode_with_runner_error_maps_to_error() {
        let status = interactive_mode_with_runner(false, || async {
            Err(io::Error::other("synthetic interactive loop failure"))
        })
        .await;
        assert_eq!(status, ExitStatus::Error);
    }

    #[tokio::test]
    async fn run_input_loop_with_exit_result_returns_success() {
        let mut buffer = String::new();
        let interrupted = Arc::new(AtomicBool::new(false));
        let mut raw_mode = FakeRawMode::default();

        let status = run_input_loop_with(
            &mut buffer,
            &mut raw_mode,
            interrupted,
            scripted_reader(vec![InputResult::Exit]),
            || None,
            || Ok(()),
        )
        .await
        .expect("loop should complete");

        assert_eq!(status, ExitStatus::Success);
        assert!(raw_mode.enable_calls >= 1);
        assert!(raw_mode.disable_calls >= 1);
    }

    #[tokio::test]
    async fn run_input_loop_with_quit_command_returns_success() {
        let mut buffer = String::new();
        let interrupted = Arc::new(AtomicBool::new(false));
        let mut raw_mode = FakeRawMode::default();

        let status = run_input_loop_with(
            &mut buffer,
            &mut raw_mode,
            interrupted,
            scripted_reader(vec![InputResult::Command("/quit".to_string())]),
            || None,
            || Ok(()),
        )
        .await
        .expect("loop should complete");

        assert_eq!(status, ExitStatus::Success);
        assert!(raw_mode.disable_calls >= 1);
    }

    #[tokio::test]
    async fn run_input_loop_with_menu_quit_returns_success() {
        let mut buffer = String::new();
        let interrupted = Arc::new(AtomicBool::new(false));
        let mut raw_mode = FakeRawMode::default();

        let status = run_input_loop_with(
            &mut buffer,
            &mut raw_mode,
            interrupted,
            scripted_reader(vec![InputResult::ShowMenu]),
            || Some(Command::Quit),
            || Ok(()),
        )
        .await
        .expect("loop should complete");

        assert_eq!(status, ExitStatus::Success);
        assert!(raw_mode.disable_calls >= 1);
    }

    #[tokio::test]
    async fn run_input_loop_with_menu_none_continues_until_exit() {
        let mut buffer = String::new();
        let interrupted = Arc::new(AtomicBool::new(false));
        let mut raw_mode = FakeRawMode::default();
        let mut menu_calls = 0usize;

        let status = run_input_loop_with(
            &mut buffer,
            &mut raw_mode,
            interrupted,
            scripted_reader(vec![InputResult::ShowMenu, InputResult::Exit]),
            || {
                menu_calls += 1;
                None
            },
            || Ok(()),
        )
        .await
        .expect("loop should complete");

        assert_eq!(status, ExitStatus::Success);
        assert_eq!(menu_calls, 1);
        assert!(raw_mode.enable_calls >= 2);
    }
}
