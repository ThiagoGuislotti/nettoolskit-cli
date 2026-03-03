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
use std::collections::VecDeque;
use std::future::Future;
use std::io;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;

pub mod display;
/// User input handling and command parsing.
pub mod input;

use display::print_logo;
use input::{read_line, InputResult, RustylineInput};
use nettoolskit_core::async_utils::with_timeout;
use nettoolskit_orchestrator::{
    get_main_action, process_command, process_text, ExitStatus, MainAction,
};
use nettoolskit_otel::{
    init_tracing_with_config, next_correlation_id, Metrics, Timer, TracingConfig,
};
use nettoolskit_ui::{
    append_footer_log, begin_interactive_logging, clear_terminal, ensure_layout_integrity,
    footer_output_enabled, render_prompt, set_footer_output_enabled, Color, CommandPalette,
    HistoryViewer, StatusBar, StatusBarMode, StatusNotificationLevel, TerminalLayout,
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
const HISTORY_COMMAND: &str = "/history";
const SESSION_HISTORY_CAPACITY: usize = 200;

/// Runtime options for interactive mode.
#[derive(Debug, Clone)]
pub struct InteractiveOptions {
    /// Enable verbose tracing output.
    pub verbose: bool,
    /// Base log level used by tracing filter setup.
    pub log_level: String,
    /// Enable footer stream rendering.
    pub footer_output: bool,
}

/// Launch the interactive CLI mode
pub async fn interactive_mode(options: InteractiveOptions) -> ExitStatus {
    interactive_mode_with_runner(options, run_interactive_loop).await
}

async fn interactive_mode_with_runner<F, Fut>(
    options: InteractiveOptions,
    run_loop: F,
) -> ExitStatus
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = io::Result<ExitStatus>>,
{
    let session_correlation_id = next_correlation_id("session");
    set_footer_output_enabled(options.footer_output);
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
        verbose: options.verbose,
        log_level: options.log_level.clone(),
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
    let interrupted = Arc::new(AtomicBool::new(false));
    let interrupted_fallback = interrupted.clone();

    // Fallback handler for Ctrl+C when raw mode is not active
    ctrlc::set_handler(move || {
        interrupted_fallback.store(true, Ordering::SeqCst);
    })
    .map_err(io::Error::other)?;

    match RustylineInput::new() {
        Ok(mut reader) => {
            let mut status_bar = StatusBar::new()
                .with_input_backend("rustyline")
                .with_max_notifications(10);
            run_input_loop_with_rustyline(&mut reader, interrupted, &mut status_bar).await
        }
        Err(err) => {
            let _ = append_footer_log(&format!(
                "Warning: Rustyline initialization failed, falling back to legacy input: {err}"
            ));
            let mut status_bar = StatusBar::new()
                .with_input_backend("legacy")
                .with_max_notifications(10);
            status_bar.push_notification(
                StatusNotificationLevel::Warning,
                "Rustyline unavailable, using legacy input path",
            );

            let mut raw_mode = RawModeGuard::new()?;
            run_input_loop_with(
                input_buffer,
                &mut raw_mode,
                &mut status_bar,
                interrupted,
                |buffer, interrupted| Box::pin(read_line(buffer, interrupted)),
                show_main_menu,
                render_prompt,
            )
            .await
        }
    }
}

async fn run_input_loop_with_rustyline(
    reader: &mut RustylineInput,
    interrupted: Arc<AtomicBool>,
    status_bar: &mut StatusBar,
) -> io::Result<ExitStatus> {
    let mut session_history = VecDeque::with_capacity(SESSION_HISTORY_CAPACITY);

    loop {
        ensure_layout_guard();
        status_bar.set_mode(StatusBarMode::Ready);
        render_status_bar(status_bar);

        match reader.read_line(&interrupted)? {
            InputResult::ShowMenu => {
                status_bar.set_mode(StatusBarMode::Menu);
                status_bar
                    .push_notification(StatusNotificationLevel::Info, "Command palette opened");
                if let Some(selected_cmd) = show_main_menu() {
                    if selected_cmd == MainAction::Quit {
                        status_bar.set_mode(StatusBarMode::Shutdown);
                        render_status_bar(status_bar);
                        print_goodbye();
                        return Ok(ExitStatus::Success);
                    }

                    let selected_label = selected_cmd.slash_static();
                    record_session_history(&mut session_history, &selected_label);
                    if is_history_command(&selected_label) {
                        status_bar.push_notification(
                            StatusNotificationLevel::Info,
                            "History viewer opened",
                        );
                        show_history_viewer(&session_history, status_bar)?;
                        ensure_layout_guard();
                        continue;
                    }

                    let started = Instant::now();
                    status_bar.set_mode(StatusBarMode::Command);
                    let status: ExitStatus = process_command(&selected_label).await;
                    record_status_outcome(status_bar, status, started.elapsed(), &selected_label);
                    if matches!(status, ExitStatus::Interrupted) {
                        return Ok(status);
                    }
                    ensure_layout_guard();
                }
                ensure_layout_guard();
            }
            InputResult::Command(cmd) => {
                record_session_history(&mut session_history, &cmd);
                if is_history_command(&cmd) {
                    status_bar.set_mode(StatusBarMode::Menu);
                    status_bar
                        .push_notification(StatusNotificationLevel::Info, "History viewer opened");
                    show_history_viewer(&session_history, status_bar)?;
                    ensure_layout_guard();
                    continue;
                }

                if let Some(MainAction::Quit) = get_main_action(&cmd) {
                    status_bar.set_mode(StatusBarMode::Shutdown);
                    render_status_bar(status_bar);
                    print_goodbye();
                    return Ok(ExitStatus::Success);
                }

                let started = Instant::now();
                status_bar.set_mode(StatusBarMode::Command);
                let status: ExitStatus = process_command(&cmd).await;
                record_status_outcome(status_bar, status, started.elapsed(), &cmd);
                if matches!(status, ExitStatus::Interrupted) {
                    return Ok(status);
                }
                ensure_layout_guard();
            }
            InputResult::Text(text) => {
                if is_empty_text_submission(&text) {
                    ensure_layout_guard();
                    continue;
                }
                record_session_history(&mut session_history, &text);
                status_bar.set_mode(StatusBarMode::Text);
                let _ = process_text(&text).await;
                status_bar.push_notification(StatusNotificationLevel::Info, "Text input processed");
                ensure_layout_guard();
            }
            InputResult::Exit => {
                status_bar.set_mode(StatusBarMode::Shutdown);
                render_status_bar(status_bar);
                if interrupted.load(Ordering::SeqCst) {
                    println!("\n⚠️  {}", "Interrupted".yellow());
                } else {
                    print_goodbye();
                }
                return Ok(ExitStatus::Success);
            }
        }
    }
}

async fn run_input_loop_with<R, F, M, P>(
    input_buffer: &mut String,
    raw_mode: &mut R,
    status_bar: &mut StatusBar,
    interrupted: Arc<AtomicBool>,
    mut read_line_fn: F,
    mut show_menu_fn: M,
    mut render_prompt_fn: P,
) -> io::Result<ExitStatus>
where
    R: RawModeControl,
    F: for<'a> FnMut(&'a mut String, &'a Arc<AtomicBool>) -> ReadLineFuture<'a>,
    M: FnMut() -> Option<MainAction>,
    P: FnMut() -> io::Result<()>,
{
    let mut session_history = VecDeque::with_capacity(SESSION_HISTORY_CAPACITY);

    loop {
        raw_mode.enable()?;
        status_bar.set_mode(StatusBarMode::Ready);
        render_status_bar(status_bar);
        render_prompt_fn()?;
        input_buffer.clear();

        match read_line_fn(input_buffer, &interrupted).await? {
            InputResult::ShowMenu => {
                // User typed "/" - show menu immediately
                status_bar.set_mode(StatusBarMode::Menu);
                status_bar
                    .push_notification(StatusNotificationLevel::Info, "Command palette opened");
                if let Some(selected_cmd) = show_menu_fn() {
                    raw_mode.disable()?;

                    // Check if user selected quit command
                    if selected_cmd == MainAction::Quit {
                        status_bar.set_mode(StatusBarMode::Shutdown);
                        render_status_bar(status_bar);
                        print_goodbye();
                        return Ok(ExitStatus::Success);
                    }

                    let selected_label = selected_cmd.slash_static();
                    record_session_history(&mut session_history, &selected_label);
                    if is_history_command(&selected_label) {
                        status_bar.push_notification(
                            StatusNotificationLevel::Info,
                            "History viewer opened",
                        );
                        show_history_viewer(&session_history, status_bar)?;
                        raw_mode.enable()?;
                        ensure_layout_guard();
                        continue;
                    }

                    let started = Instant::now();
                    status_bar.set_mode(StatusBarMode::Command);
                    let status: ExitStatus = process_command(&selected_label).await;
                    record_status_outcome(status_bar, status, started.elapsed(), &selected_label);
                    if matches!(status, ExitStatus::Interrupted) {
                        return Ok(status);
                    }
                    raw_mode.enable()?;
                    ensure_layout_guard();
                }
                ensure_layout_guard();
                // Clear buffer and prompt for next input
                input_buffer.clear();
                continue;
            }
            InputResult::Command(cmd) => {
                raw_mode.disable()?;
                record_session_history(&mut session_history, &cmd);
                if is_history_command(&cmd) {
                    status_bar.set_mode(StatusBarMode::Menu);
                    status_bar
                        .push_notification(StatusNotificationLevel::Info, "History viewer opened");
                    show_history_viewer(&session_history, status_bar)?;
                    raw_mode.enable()?;
                    ensure_layout_guard();
                    continue;
                }

                // Check if user typed quit command
                if let Some(MainAction::Quit) = get_main_action(&cmd) {
                    status_bar.set_mode(StatusBarMode::Shutdown);
                    render_status_bar(status_bar);
                    print_goodbye();
                    return Ok(ExitStatus::Success);
                }

                let started = Instant::now();
                status_bar.set_mode(StatusBarMode::Command);
                let status: ExitStatus = process_command(&cmd).await;
                record_status_outcome(status_bar, status, started.elapsed(), &cmd);
                if matches!(status, ExitStatus::Interrupted) {
                    return Ok(status);
                }
                raw_mode.enable()?;
                // NOTE: Layout guard kept for commands as they may modify terminal state
                ensure_layout_guard();
            }
            InputResult::Text(text) => {
                if is_empty_text_submission(&text) {
                    raw_mode.enable()?;
                    ensure_layout_guard();
                    continue;
                }
                raw_mode.disable()?;
                record_session_history(&mut session_history, &text);
                status_bar.set_mode(StatusBarMode::Text);
                let _ = process_text(&text).await;
                status_bar.push_notification(StatusNotificationLevel::Info, "Text input processed");
                raw_mode.enable()?;
                ensure_layout_guard();
            }
            InputResult::Exit => {
                raw_mode.disable()?;
                status_bar.set_mode(StatusBarMode::Shutdown);
                render_status_bar(status_bar);
                // Check if this was triggered by Ctrl+C
                if interrupted.load(Ordering::SeqCst) {
                    println!("\n⚠️  {}", "Interrupted".yellow());
                } else {
                    print_goodbye();
                }
                return Ok(ExitStatus::Success);
            }
        }
    }
}

/// Show main menu when user types "/" - returns MainAction enum directly
fn show_main_menu() -> Option<MainAction> {
    use nettoolskit_core::MenuProvider;
    let menu_entries = MainAction::all_variants();
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

fn is_history_command(command: &str) -> bool {
    command.trim().eq_ignore_ascii_case(HISTORY_COMMAND)
}

fn is_empty_text_submission(text: &str) -> bool {
    text.trim().is_empty()
}

fn record_session_history(history: &mut VecDeque<String>, entry: &str) {
    let trimmed = entry.trim();
    if trimmed.is_empty() {
        return;
    }

    if history.len() == SESSION_HISTORY_CAPACITY {
        history.pop_front();
    }

    history.push_back(trimmed.to_string());
}

fn show_history_viewer(history: &VecDeque<String>, status_bar: &mut StatusBar) -> io::Result<()> {
    if history.is_empty() {
        status_bar.push_notification(
            StatusNotificationLevel::Warning,
            "History is empty for this session",
        );
        println!(
            "{}",
            "No session history entries available.".color(Color::YELLOW)
        );
        return Ok(());
    }

    let mut viewer = HistoryViewer::new(history.iter().cloned().collect())
        .with_title("Session History")
        .with_subtitle("Use typing to filter entries. Enter selects. Esc cancels.")
        .with_prompt("History entry:")
        .with_help_message("Type to search. Up/Down navigate. Enter selects. Esc cancels.")
        .with_page_size(12);
    viewer.scroll_to_last_page();

    match viewer.show() {
        Some(selected) => {
            status_bar.push_notification(
                StatusNotificationLevel::Info,
                format!("History selection: {selected}"),
            );
        }
        None => {
            status_bar.push_notification(StatusNotificationLevel::Info, "History viewer closed");
        }
    }

    Ok(())
}

fn record_status_outcome(
    status_bar: &mut StatusBar,
    status: ExitStatus,
    duration: std::time::Duration,
    command_label: &str,
) {
    status_bar.record_command_result(status, duration);

    match status {
        ExitStatus::Success => {
            status_bar.push_notification(
                StatusNotificationLevel::Success,
                format!("{command_label} completed"),
            );
        }
        ExitStatus::Error => {
            status_bar.push_notification(
                StatusNotificationLevel::Error,
                format!("{command_label} failed"),
            );
        }
        ExitStatus::Interrupted => {
            status_bar.push_notification(
                StatusNotificationLevel::Warning,
                format!("{command_label} interrupted"),
            );
        }
    }
}

fn render_status_bar(status_bar: &StatusBar) {
    // Inline status bar and footer logger both compete for dynamic area rendering.
    // When footer output is enabled, skip inline status line to avoid visual duplication.
    if footer_output_enabled() {
        return;
    }

    if let Err(err) = status_bar.render() {
        warn!(error = %err, "Failed to render interactive status bar");
        let _ = append_footer_log(&format!("Warning: failed to render status bar: {err}"));
    }
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

    fn default_interactive_options() -> InteractiveOptions {
        InteractiveOptions {
            verbose: false,
            log_level: "info".to_string(),
            footer_output: true,
        }
    }

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
        let status = interactive_mode_with_runner(default_interactive_options(), || async {
            Ok(ExitStatus::Success)
        })
        .await;
        assert_eq!(status, ExitStatus::Success);
    }

    #[tokio::test]
    async fn interactive_mode_with_runner_error_maps_to_error() {
        let status = interactive_mode_with_runner(default_interactive_options(), || async {
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
        let mut status_bar = StatusBar::new().with_input_backend("test");

        let status = run_input_loop_with(
            &mut buffer,
            &mut raw_mode,
            &mut status_bar,
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
        let mut status_bar = StatusBar::new().with_input_backend("test");

        let status = run_input_loop_with(
            &mut buffer,
            &mut raw_mode,
            &mut status_bar,
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
        let mut status_bar = StatusBar::new().with_input_backend("test");

        let status = run_input_loop_with(
            &mut buffer,
            &mut raw_mode,
            &mut status_bar,
            interrupted,
            scripted_reader(vec![InputResult::ShowMenu]),
            || Some(MainAction::Quit),
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
        let mut status_bar = StatusBar::new().with_input_backend("test");
        let mut menu_calls = 0usize;

        let status = run_input_loop_with(
            &mut buffer,
            &mut raw_mode,
            &mut status_bar,
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

    #[tokio::test]
    async fn run_input_loop_with_empty_text_does_not_add_processed_notification() {
        let mut buffer = String::new();
        let interrupted = Arc::new(AtomicBool::new(false));
        let mut raw_mode = FakeRawMode::default();
        let mut status_bar = StatusBar::new().with_input_backend("test");

        let status = run_input_loop_with(
            &mut buffer,
            &mut raw_mode,
            &mut status_bar,
            interrupted,
            scripted_reader(vec![
                InputResult::Text("   ".to_string()),
                InputResult::Exit,
            ]),
            || None,
            || Ok(()),
        )
        .await
        .expect("loop should complete");

        assert_eq!(status, ExitStatus::Success);
        assert_ne!(
            status_bar.latest_notification_message(),
            Some("Text input processed")
        );
    }

    #[test]
    fn record_status_outcome_success_adds_notification() {
        let mut status_bar = StatusBar::new().with_input_backend("test");
        record_status_outcome(
            &mut status_bar,
            ExitStatus::Success,
            std::time::Duration::from_millis(12),
            "/help",
        );

        assert_eq!(
            status_bar.latest_notification_message(),
            Some("/help completed")
        );
    }

    #[test]
    fn record_status_outcome_error_adds_failure_notification() {
        let mut status_bar = StatusBar::new().with_input_backend("test");
        record_status_outcome(
            &mut status_bar,
            ExitStatus::Error,
            std::time::Duration::from_millis(25),
            "/manifest",
        );

        assert_eq!(
            status_bar.latest_notification_message(),
            Some("/manifest failed")
        );
    }

    #[test]
    fn is_history_command_matches_expected_alias() {
        assert!(is_history_command("/history"));
        assert!(is_history_command(" /history "));
        assert!(!is_history_command("/help"));
    }

    #[test]
    fn is_empty_text_submission_detects_blank_inputs() {
        assert!(is_empty_text_submission(""));
        assert!(is_empty_text_submission("   "));
        assert!(!is_empty_text_submission("hello"));
    }

    #[test]
    fn record_session_history_enforces_capacity() {
        let mut history = VecDeque::new();
        for idx in 0..(SESSION_HISTORY_CAPACITY + 5) {
            record_session_history(&mut history, &format!("entry-{idx}"));
        }

        assert_eq!(history.len(), SESSION_HISTORY_CAPACITY);
        assert_eq!(history.front().map(String::as_str), Some("entry-5"));
    }
}
