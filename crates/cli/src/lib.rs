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

use crossterm::event::{DisableFocusChange, EnableFocusChange};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use nettoolskit_core::{AppConfig, CommandEntry, MenuEntry};
use owo_colors::OwoColorize;
use std::collections::VecDeque;
use std::future::Future;
use std::io;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

pub mod display;
/// User input handling and command parsing.
pub mod input;
/// Shared rich session state primitives.
pub mod state;

use display::print_logo;
use input::{read_line, InputResult, RustylineInput};
use nettoolskit_core::async_utils::with_timeout;
use nettoolskit_orchestrator::{
    get_main_action, list_local_ai_session_snapshots, load_local_ai_session_from_path,
    process_command_with_interrupt, process_text, set_active_ai_session_id, ExitStatus,
    LocalAiSessionSnapshot, LocalAiSessionState, MainAction, LOCAL_AI_SESSIONS_DIR_NAME,
};
use nettoolskit_otel::{
    init_tracing_with_config, next_correlation_id, Metrics, Timer, TracingConfig,
};
use nettoolskit_ui::{
    append_footer_log, begin_interactive_logging, clear_terminal, consume_scheduled_terminal_frame,
    emit_attention_bell, emit_desktop_attention_notification, ensure_layout_integrity,
    footer_output_enabled, render_prompt, request_terminal_frame, set_focus_detection_enabled,
    set_footer_output_enabled, set_terminal_focused, should_emit_attention_signal, Color,
    CommandPalette, HistoryViewer, StatusBar, StatusBarMode, StatusNotificationLevel,
    TerminalLayout,
};
use state::{CliState, HistoryEntryKind, LocalSessionSnapshot, SharedCliState};
use tracing::{error, info, info_span, warn};

struct RawModeGuard {
    active: bool,
    focus_tracking: bool,
}

trait RawModeControl {
    fn enable(&mut self) -> io::Result<()>;
    fn disable(&mut self) -> io::Result<()>;
    fn enable_focus_tracking(&mut self) -> io::Result<()> {
        Ok(())
    }
    fn disable_focus_tracking(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl RawModeGuard {
    fn new() -> io::Result<Self> {
        enable_raw_mode()?;
        Ok(Self {
            active: true,
            focus_tracking: false,
        })
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

    fn enable_focus_tracking(&mut self) -> io::Result<()> {
        if !self.focus_tracking {
            crossterm::execute!(io::stdout(), EnableFocusChange)?;
            self.focus_tracking = true;
        }
        Ok(())
    }

    fn disable_focus_tracking(&mut self) -> io::Result<()> {
        if self.focus_tracking {
            crossterm::execute!(io::stdout(), DisableFocusChange)?;
            self.focus_tracking = false;
        }
        Ok(())
    }
}

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        if self.focus_tracking {
            let _ = crossterm::execute!(io::stdout(), DisableFocusChange);
            self.focus_tracking = false;
        }
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

    fn enable_focus_tracking(&mut self) -> io::Result<()> {
        Self::enable_focus_tracking(self)
    }

    fn disable_focus_tracking(&mut self) -> io::Result<()> {
        Self::disable_focus_tracking(self)
    }
}

type ReadLineFuture<'a> = Pin<Box<dyn Future<Output = io::Result<InputResult>> + 'a>>;
const HISTORY_COMMAND: &str = "/history";
const SESSION_HISTORY_CAPACITY: usize = 200;
const LOCAL_SESSION_RETENTION: usize = 25;
const LOCAL_SESSION_PICKER_LIMIT: usize = 20;
const LOCAL_AI_SESSION_PICKER_LIMIT: usize = 20;
const MAX_CONSECUTIVE_INPUT_ERRORS: usize = 3;
const INPUT_ERROR_RECOVERY_BACKOFF_MS: u64 = 75;

#[derive(Debug, Clone, Copy, Default)]
struct AttentionConfig {
    enabled: bool,
    desktop_notification: bool,
    unfocused_only: bool,
}

#[derive(Clone)]
struct InputLoopRuntimeContext {
    interrupted: Arc<AtomicBool>,
    state: SharedCliState,
    attention: AttentionConfig,
}

#[derive(Clone)]
struct SessionResumeOption {
    label: String,
    description: String,
    path: PathBuf,
}

impl MenuEntry for SessionResumeOption {
    fn label(&self) -> &str {
        &self.label
    }

    fn description(&self) -> &str {
        &self.description
    }
}

#[derive(Clone)]
struct AiSessionResumeOption {
    label: String,
    description: String,
    path: PathBuf,
}

impl MenuEntry for AiSessionResumeOption {
    fn label(&self) -> &str {
        &self.label
    }

    fn description(&self) -> &str {
        &self.description
    }
}

/// Runtime options for interactive mode.
#[derive(Debug, Clone)]
pub struct InteractiveOptions {
    /// Enable verbose tracing output.
    pub verbose: bool,
    /// Base log level used by tracing filter setup.
    pub log_level: String,
    /// Enable footer stream rendering.
    pub footer_output: bool,
    /// Emit terminal attention bell on failed/interrupted command outcomes.
    pub attention_bell: bool,
    /// Emit desktop notification on failed/interrupted command outcomes.
    pub attention_desktop_notification: bool,
    /// Only emit attention bell when terminal focus is lost.
    pub attention_unfocused_only: bool,

    /// Enable predictive slash-command hints in interactive input.
    pub predictive_input: bool,

    /// Number of local AI session snapshots retained on disk.
    pub ai_session_retention: usize,
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
    F: FnOnce(InteractiveOptions) -> Fut,
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

    let result = match run_loop(options.clone()).await {
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

async fn run_interactive_loop(options: InteractiveOptions) -> io::Result<ExitStatus> {
    info!("Starting interactive loop");
    let attention = AttentionConfig {
        enabled: options.attention_bell,
        desktop_notification: options.attention_desktop_notification,
        unfocused_only: options.attention_unfocused_only,
    };
    let runtime_state = initialize_runtime_state(&options);
    let interactive_result =
        run_input_loop(attention, options.predictive_input, runtime_state.clone()).await;
    persist_runtime_state_snapshot(&runtime_state);
    interactive_result
}

async fn run_input_loop(
    attention: AttentionConfig,
    predictive_input: bool,
    state: SharedCliState,
) -> io::Result<ExitStatus> {
    let interrupted = Arc::new(AtomicBool::new(false));
    let interrupted_fallback = interrupted.clone();

    // Fallback handler for Ctrl+C when raw mode is not active
    ctrlc::set_handler(move || {
        interrupted_fallback.store(true, Ordering::SeqCst);
    })
    .map_err(io::Error::other)?;

    match RustylineInput::new_with_predictive_input(predictive_input) {
        Ok(mut reader) => {
            set_focus_detection_enabled(false);
            set_terminal_focused(true);
            let mut status_bar = StatusBar::new()
                .with_input_backend("rustyline")
                .with_max_notifications(10);
            run_input_loop_with_rustyline(
                &mut reader,
                interrupted,
                &mut status_bar,
                state,
                attention,
            )
            .await
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
            set_focus_detection_enabled(attention.unfocused_only);
            set_terminal_focused(true);
            let runtime_context = InputLoopRuntimeContext {
                interrupted,
                state,
                attention,
            };
            let result = run_input_loop_with(
                &mut raw_mode,
                &mut status_bar,
                runtime_context,
                |buffer, interrupted| Box::pin(read_line(buffer, interrupted)),
                show_main_menu,
                render_prompt,
            )
            .await;
            set_focus_detection_enabled(false);
            result
        }
    }
}

async fn run_input_loop_with_rustyline(
    reader: &mut RustylineInput,
    interrupted: Arc<AtomicBool>,
    status_bar: &mut StatusBar,
    state: SharedCliState,
    attention: AttentionConfig,
) -> io::Result<ExitStatus> {
    let mut session_history = seed_session_history_from_state(&state);
    let mut consecutive_input_errors = 0usize;

    loop {
        ensure_layout_guard();
        status_bar.set_mode(StatusBarMode::Ready);
        render_status_bar(status_bar);

        let input_result = match reader.read_line(&interrupted) {
            Ok(result) => {
                consecutive_input_errors = 0;
                result
            }
            Err(err) => {
                if register_recoverable_input_error(
                    status_bar,
                    "rustyline",
                    &err,
                    &mut consecutive_input_errors,
                ) {
                    tokio::time::sleep(Duration::from_millis(INPUT_ERROR_RECOVERY_BACKOFF_MS))
                        .await;
                    ensure_layout_guard();
                    continue;
                }

                return Err(err);
            }
        };

        match input_result {
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
                    record_state_history_entry(&state, HistoryEntryKind::Command, &selected_label);
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
                    let status = execute_command_with_recovery(&selected_label, &interrupted).await;
                    record_status_outcome(
                        status_bar,
                        status,
                        started.elapsed(),
                        &selected_label,
                        attention,
                    );
                    if matches!(status, ExitStatus::Interrupted) {
                        return Ok(status);
                    }
                    ensure_layout_guard();
                }
                ensure_layout_guard();
            }
            InputResult::Command(cmd) => {
                record_session_history(&mut session_history, &cmd);
                record_state_history_entry(&state, HistoryEntryKind::Command, &cmd);
                if is_history_command(&cmd) {
                    status_bar.set_mode(StatusBarMode::Menu);
                    status_bar
                        .push_notification(StatusNotificationLevel::Info, "History viewer opened");
                    show_history_viewer(&session_history, status_bar)?;
                    ensure_layout_guard();
                    continue;
                }

                if is_ai_resume_command(&cmd) {
                    status_bar.set_mode(StatusBarMode::Menu);
                    status_bar.push_notification(
                        StatusNotificationLevel::Info,
                        "AI session resume picker opened",
                    );
                    handle_ai_resume_with_picker(status_bar);
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
                let status = execute_command_with_recovery(&cmd, &interrupted).await;
                record_status_outcome(status_bar, status, started.elapsed(), &cmd, attention);
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
                record_state_history_entry(&state, HistoryEntryKind::Text, &text);
                status_bar.set_mode(StatusBarMode::Text);
                let text_status = execute_text_with_recovery(&text).await;
                match text_status {
                    ExitStatus::Success => {
                        status_bar.push_notification(
                            StatusNotificationLevel::Info,
                            "Text input processed",
                        );
                    }
                    ExitStatus::Error => {
                        status_bar.push_notification(
                            StatusNotificationLevel::Warning,
                            "Text input failed but session recovered",
                        );
                    }
                    ExitStatus::Interrupted => {
                        status_bar.push_notification(
                            StatusNotificationLevel::Warning,
                            "Text input interrupted",
                        );
                    }
                }
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
    raw_mode: &mut R,
    status_bar: &mut StatusBar,
    runtime_context: InputLoopRuntimeContext,
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
    let interrupted = &runtime_context.interrupted;
    let state = &runtime_context.state;
    let attention = runtime_context.attention;
    let mut session_history = seed_session_history_from_state(state);
    let mut input_buffer = String::new();
    let mut consecutive_input_errors = 0usize;
    if attention.unfocused_only {
        raw_mode.enable_focus_tracking()?;
    }

    loop {
        raw_mode.enable()?;
        status_bar.set_mode(StatusBarMode::Ready);
        render_status_bar(status_bar);
        render_prompt_fn()?;
        input_buffer.clear();

        let input_result = match read_line_fn(&mut input_buffer, interrupted).await {
            Ok(result) => {
                consecutive_input_errors = 0;
                result
            }
            Err(err) => {
                if register_recoverable_input_error(
                    status_bar,
                    "legacy",
                    &err,
                    &mut consecutive_input_errors,
                ) {
                    tokio::time::sleep(Duration::from_millis(INPUT_ERROR_RECOVERY_BACKOFF_MS))
                        .await;
                    ensure_layout_guard();
                    continue;
                }

                return Err(err);
            }
        };

        match input_result {
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
                        if attention.unfocused_only {
                            let _ = raw_mode.disable_focus_tracking();
                        }
                        return Ok(ExitStatus::Success);
                    }

                    let selected_label = selected_cmd.slash_static();
                    record_session_history(&mut session_history, &selected_label);
                    record_state_history_entry(state, HistoryEntryKind::Command, &selected_label);
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
                    let status = execute_command_with_recovery(&selected_label, interrupted).await;
                    record_status_outcome(
                        status_bar,
                        status,
                        started.elapsed(),
                        &selected_label,
                        attention,
                    );
                    if matches!(status, ExitStatus::Interrupted) {
                        if attention.unfocused_only {
                            let _ = raw_mode.disable_focus_tracking();
                        }
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
                record_state_history_entry(state, HistoryEntryKind::Command, &cmd);
                if is_history_command(&cmd) {
                    status_bar.set_mode(StatusBarMode::Menu);
                    status_bar
                        .push_notification(StatusNotificationLevel::Info, "History viewer opened");
                    show_history_viewer(&session_history, status_bar)?;
                    raw_mode.enable()?;
                    ensure_layout_guard();
                    continue;
                }

                if is_ai_resume_command(&cmd) {
                    status_bar.set_mode(StatusBarMode::Menu);
                    status_bar.push_notification(
                        StatusNotificationLevel::Info,
                        "AI session resume picker opened",
                    );
                    handle_ai_resume_with_picker(status_bar);
                    raw_mode.enable()?;
                    ensure_layout_guard();
                    continue;
                }

                // Check if user typed quit command
                if let Some(MainAction::Quit) = get_main_action(&cmd) {
                    status_bar.set_mode(StatusBarMode::Shutdown);
                    render_status_bar(status_bar);
                    print_goodbye();
                    if attention.unfocused_only {
                        let _ = raw_mode.disable_focus_tracking();
                    }
                    return Ok(ExitStatus::Success);
                }

                let started = Instant::now();
                status_bar.set_mode(StatusBarMode::Command);
                let status = execute_command_with_recovery(&cmd, interrupted).await;
                record_status_outcome(status_bar, status, started.elapsed(), &cmd, attention);
                if matches!(status, ExitStatus::Interrupted) {
                    if attention.unfocused_only {
                        let _ = raw_mode.disable_focus_tracking();
                    }
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
                record_state_history_entry(state, HistoryEntryKind::Text, &text);
                status_bar.set_mode(StatusBarMode::Text);
                let text_status = execute_text_with_recovery(&text).await;
                match text_status {
                    ExitStatus::Success => {
                        status_bar.push_notification(
                            StatusNotificationLevel::Info,
                            "Text input processed",
                        );
                    }
                    ExitStatus::Error => {
                        status_bar.push_notification(
                            StatusNotificationLevel::Warning,
                            "Text input failed but session recovered",
                        );
                    }
                    ExitStatus::Interrupted => {
                        status_bar.push_notification(
                            StatusNotificationLevel::Warning,
                            "Text input interrupted",
                        );
                    }
                }
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
                if attention.unfocused_only {
                    let _ = raw_mode.disable_focus_tracking();
                }
                return Ok(ExitStatus::Success);
            }
        }
    }
}

fn build_runtime_state_config(options: &InteractiveOptions) -> AppConfig {
    let mut config = AppConfig::default();
    config.general.verbose = options.verbose;
    config.general.log_level = options.log_level.clone();
    config.general.footer_output = options.footer_output;
    config.general.attention_bell = options.attention_bell;
    config.general.attention_desktop_notification = options.attention_desktop_notification;
    config.general.attention_unfocused_only = options.attention_unfocused_only;
    config.general.predictive_input = options.predictive_input;
    config.general.ai_session_retention = options.ai_session_retention;
    config
}

fn initialize_runtime_state(options: &InteractiveOptions) -> SharedCliState {
    if let Some(selected_state) = select_local_session_with_picker() {
        let runtime_config = build_runtime_state_config(options);
        return restore_loaded_runtime_state(selected_state, runtime_config);
    }

    initialize_runtime_state_with_loader(options, CliState::load_latest_local_snapshot)
}

fn initialize_runtime_state_with_loader<F>(
    options: &InteractiveOptions,
    load_latest_snapshot: F,
) -> SharedCliState
where
    F: FnOnce() -> io::Result<Option<CliState>>,
{
    let runtime_config = build_runtime_state_config(options);

    match load_latest_snapshot() {
        Ok(Some(loaded_state)) => restore_loaded_runtime_state(loaded_state, runtime_config),
        Ok(None) => {
            info!("No local session snapshot found, starting fresh interactive state");
            CliState::shared(
                next_correlation_id("state"),
                runtime_config,
                SESSION_HISTORY_CAPACITY,
            )
        }
        Err(err) => {
            warn!(
                error = %err,
                "Failed to load local session snapshot; starting fresh interactive state"
            );
            CliState::shared(
                next_correlation_id("state"),
                runtime_config,
                SESSION_HISTORY_CAPACITY,
            )
        }
    }
}

fn restore_loaded_runtime_state(
    mut loaded_state: CliState,
    runtime_config: AppConfig,
) -> SharedCliState {
    loaded_state.config = runtime_config;
    loaded_state.session.history_capacity = SESSION_HISTORY_CAPACITY;
    while loaded_state.history.len() > SESSION_HISTORY_CAPACITY {
        loaded_state.history.pop_front();
    }

    let session_id = loaded_state.session.id.clone();
    let history_entries = loaded_state.history.len();
    info!(
        session_id = %session_id,
        history_entries,
        "Resumed local session snapshot"
    );
    loaded_state.into_shared()
}

fn select_local_session_with_picker() -> Option<CliState> {
    select_local_session_with_picker_using(
        CliState::list_local_snapshots,
        show_session_resume_picker,
        CliState::load_local_snapshot_from_path,
    )
}

fn select_local_session_with_picker_using<L, P, R>(
    mut list_snapshots: L,
    mut pick_snapshot: P,
    mut read_snapshot: R,
) -> Option<CliState>
where
    L: FnMut(usize) -> io::Result<Option<Vec<LocalSessionSnapshot>>>,
    P: FnMut(&[LocalSessionSnapshot]) -> Option<PathBuf>,
    R: FnMut(&Path) -> io::Result<CliState>,
{
    let snapshots = match list_snapshots(LOCAL_SESSION_PICKER_LIMIT) {
        Ok(Some(snapshots)) => snapshots,
        Ok(None) => return None,
        Err(err) => {
            warn!(
                error = %err,
                "Failed to list local snapshots for startup picker"
            );
            return None;
        }
    };

    if snapshots.len() <= 1 {
        return None;
    }

    let selected_path = pick_snapshot(&snapshots)?;
    match read_snapshot(&selected_path) {
        Ok(state) => Some(state),
        Err(err) => {
            warn!(
                error = %err,
                path = %selected_path.display(),
                "Failed to load selected local snapshot"
            );
            None
        }
    }
}

fn show_session_resume_picker(snapshots: &[LocalSessionSnapshot]) -> Option<PathBuf> {
    let options = build_session_resume_options(snapshots);
    if options.is_empty() {
        return None;
    }

    let mut palette = CommandPalette::new(options.clone())
        .with_prompt("resume >")
        .with_title("Resume Local Session")
        .with_subtitle("Select a local snapshot to restore");

    if let Some(directory) = local_sessions_dir_label() {
        palette = palette.with_directory(directory);
    }

    let selected_label = palette.show()?;
    options
        .into_iter()
        .find(|option| option.label == selected_label.trim())
        .map(|option| option.path)
}

fn build_session_resume_options(snapshots: &[LocalSessionSnapshot]) -> Vec<SessionResumeOption> {
    snapshots
        .iter()
        .enumerate()
        .map(|(index, snapshot)| SessionResumeOption {
            label: format!("#{:02} {}", index + 1, snapshot.id),
            description: format!(
                "entries:{} start:{} last:{}",
                snapshot.history_entries, snapshot.started_at_ms, snapshot.last_activity_ms
            ),
            path: snapshot.path.clone(),
        })
        .collect()
}

fn local_sessions_dir_label() -> Option<String> {
    AppConfig::default_data_dir()
        .map(|dir| dir.join("sessions"))
        .and_then(|path| path.to_str().map(ToOwned::to_owned))
}

fn select_local_ai_session_with_picker() -> Option<String> {
    select_local_ai_session_with_picker_using(
        list_local_ai_session_snapshots,
        show_ai_session_resume_picker,
        load_local_ai_session_from_path,
        set_active_ai_session_id,
    )
}

fn select_local_ai_session_with_picker_using<L, P, R, S>(
    mut list_snapshots: L,
    mut pick_snapshot: P,
    mut read_snapshot: R,
    mut activate_session: S,
) -> Option<String>
where
    L: FnMut(usize) -> io::Result<Option<Vec<LocalAiSessionSnapshot>>>,
    P: FnMut(&[LocalAiSessionSnapshot]) -> Option<PathBuf>,
    R: FnMut(&Path) -> io::Result<LocalAiSessionState>,
    S: FnMut(&str) -> String,
{
    let snapshots = match list_snapshots(LOCAL_AI_SESSION_PICKER_LIMIT) {
        Ok(Some(snapshots)) => snapshots,
        Ok(None) => return None,
        Err(err) => {
            warn!(
                error = %err,
                "Failed to list local AI snapshots for resume picker"
            );
            return None;
        }
    };

    if snapshots.is_empty() {
        return None;
    }

    let selected_path = if snapshots.len() == 1 {
        snapshots[0].path.clone()
    } else {
        pick_snapshot(&snapshots)?
    };

    match read_snapshot(&selected_path) {
        Ok(session) => Some(activate_session(&session.id)),
        Err(err) => {
            warn!(
                error = %err,
                path = %selected_path.display(),
                "Failed to load selected local AI snapshot"
            );
            None
        }
    }
}

fn show_ai_session_resume_picker(snapshots: &[LocalAiSessionSnapshot]) -> Option<PathBuf> {
    let options = build_ai_session_resume_options(snapshots);
    if options.is_empty() {
        return None;
    }

    let mut palette = CommandPalette::new(options.clone())
        .with_prompt("ai-resume >")
        .with_title("Resume AI Session")
        .with_subtitle("Select a local AI session to continue context");

    if let Some(directory) = local_ai_sessions_dir_label() {
        palette = palette.with_directory(directory);
    }

    let selected_label = palette.show()?;
    options
        .into_iter()
        .find(|option| option.label == selected_label.trim())
        .map(|option| option.path)
}

fn build_ai_session_resume_options(
    snapshots: &[LocalAiSessionSnapshot],
) -> Vec<AiSessionResumeOption> {
    snapshots
        .iter()
        .enumerate()
        .map(|(index, snapshot)| AiSessionResumeOption {
            label: format!("#{:02} {}", index + 1, snapshot.id),
            description: format!(
                "exchanges:{} start:{} last:{}",
                snapshot.exchange_count, snapshot.started_at_ms, snapshot.last_activity_ms
            ),
            path: snapshot.path.clone(),
        })
        .collect()
}

fn local_ai_sessions_dir_label() -> Option<String> {
    AppConfig::default_data_dir()
        .map(|dir| dir.join(LOCAL_AI_SESSIONS_DIR_NAME))
        .and_then(|path| path.to_str().map(ToOwned::to_owned))
}

fn handle_ai_resume_with_picker(status_bar: &mut StatusBar) {
    if let Some(session_id) = select_local_ai_session_with_picker() {
        status_bar.push_notification(
            StatusNotificationLevel::Info,
            format!("AI session resumed: {session_id}"),
        );
        let _ = append_footer_log(&format!(
            "ai: active session switched via picker id={session_id}"
        ));
    } else {
        status_bar.push_notification(
            StatusNotificationLevel::Warning,
            "No local AI sessions available to resume",
        );
    }
}

fn seed_session_history_from_state(state: &SharedCliState) -> VecDeque<String> {
    let mut history = {
        let guard = state
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        guard.history_lines()
    };

    while history.len() > SESSION_HISTORY_CAPACITY {
        history.pop_front();
    }

    history
}

fn persist_runtime_state_snapshot(state: &SharedCliState) {
    persist_runtime_state_snapshot_with_io(
        state,
        CliState::save_local_snapshot,
        CliState::prune_local_snapshots,
    );
}

fn persist_runtime_state_snapshot_with_io<S, P>(
    state: &SharedCliState,
    mut save_snapshot: S,
    mut prune_snapshots: P,
) where
    S: FnMut(&CliState) -> io::Result<Option<std::path::PathBuf>>,
    P: FnMut(usize) -> io::Result<Option<usize>>,
{
    let state_snapshot = {
        let guard = state
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        guard.clone()
    };

    match save_snapshot(&state_snapshot) {
        Ok(Some(path)) => {
            info!(
                session_id = %state_snapshot.session.id,
                history_entries = state_snapshot.history.len(),
                path = %path.display(),
                "Persisted local session snapshot"
            );
        }
        Ok(None) => {
            warn!("Local data directory unavailable; session snapshot persistence skipped");
        }
        Err(err) => {
            warn!(error = %err, "Failed to persist local session snapshot");
            let _ = append_footer_log(&format!(
                "Warning: failed to persist local session state: {err}"
            ));
        }
    }

    match prune_snapshots(LOCAL_SESSION_RETENTION) {
        Ok(Some(removed)) if removed > 0 => {
            info!(
                removed,
                keep_latest = LOCAL_SESSION_RETENTION,
                "Pruned old local session snapshots"
            );
        }
        Ok(Some(_)) | Ok(None) => {}
        Err(err) => {
            warn!(error = %err, "Failed to prune local session snapshots");
            let _ = append_footer_log(&format!(
                "Warning: failed to prune local session snapshots: {err}"
            ));
        }
    }
}

fn record_state_history_entry(state: &SharedCliState, kind: HistoryEntryKind, entry: &str) {
    let mut guard = state
        .write()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    match kind {
        HistoryEntryKind::Command => {
            let _ = guard.push_command(entry);
        }
        HistoryEntryKind::Text => {
            let _ = guard.push_text(entry);
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

fn is_ai_resume_command(command: &str) -> bool {
    let parts = command.split_whitespace().collect::<Vec<_>>();
    parts.len() == 2
        && parts[0].eq_ignore_ascii_case("/ai")
        && parts[1].eq_ignore_ascii_case("resume")
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
    attention: AttentionConfig,
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
            if should_emit_attention_signal(attention.unfocused_only) {
                if attention.enabled {
                    let _ = emit_attention_bell();
                }
                if attention.desktop_notification {
                    let _ = emit_desktop_attention_notification(
                        "NetToolsKit CLI",
                        &format!("{command_label} failed"),
                    );
                }
            }
            status_bar.push_notification(
                StatusNotificationLevel::Error,
                format!("{command_label} failed"),
            );
        }
        ExitStatus::Interrupted => {
            if should_emit_attention_signal(attention.unfocused_only) {
                if attention.enabled {
                    let _ = emit_attention_bell();
                }
                if attention.desktop_notification {
                    let _ = emit_desktop_attention_notification(
                        "NetToolsKit CLI",
                        &format!("{command_label} interrupted"),
                    );
                }
            }
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

    request_terminal_frame();
    if !consume_scheduled_terminal_frame() {
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

fn register_recoverable_input_error(
    status_bar: &mut StatusBar,
    input_backend: &str,
    error: &io::Error,
    consecutive_errors: &mut usize,
) -> bool {
    *consecutive_errors += 1;

    warn!(
        backend = input_backend,
        attempt = *consecutive_errors,
        max_attempts = MAX_CONSECUTIVE_INPUT_ERRORS,
        error = %error,
        "Recoverable input failure"
    );
    let _ = append_footer_log(&format!(
        "Recoverable {input_backend} input failure ({}/{MAX_CONSECUTIVE_INPUT_ERRORS}): {error}",
        *consecutive_errors
    ));
    status_bar.push_notification(
        StatusNotificationLevel::Warning,
        format!(
            "{input_backend} input recovery {}/{}",
            *consecutive_errors, MAX_CONSECUTIVE_INPUT_ERRORS
        ),
    );

    if *consecutive_errors >= MAX_CONSECUTIVE_INPUT_ERRORS {
        status_bar.push_notification(
            StatusNotificationLevel::Error,
            format!("{input_backend} input recovery exhausted"),
        );
        let _ = append_footer_log(&format!(
            "{input_backend} input recovery exhausted after {} consecutive failures",
            *consecutive_errors
        ));
        return false;
    }

    true
}

async fn await_recoverable_task<F>(task_label: &str, task: F) -> ExitStatus
where
    F: Future<Output = ExitStatus> + Send + 'static,
{
    match tokio::spawn(task).await {
        Ok(status) => status,
        Err(join_error) => {
            let failure_kind = if join_error.is_panic() {
                "panic"
            } else {
                "cancellation"
            };

            warn!(
                task = task_label,
                error = %join_error,
                "Recovered from background task {failure_kind}"
            );
            let _ = append_footer_log(&format!(
                "Recovered from {failure_kind} while executing '{task_label}'. Session remains active."
            ));
            ExitStatus::Error
        }
    }
}

async fn execute_command_with_recovery(
    command_label: &str,
    interrupted: &Arc<AtomicBool>,
) -> ExitStatus {
    let command = command_label.to_string();
    let interrupted_flag = Arc::clone(interrupted);

    await_recoverable_task(command_label, async move {
        process_command_with_interrupt(&command, Some(interrupted_flag.as_ref())).await
    })
    .await
}

async fn execute_text_with_recovery(text: &str) -> ExitStatus {
    let text_owned = text.to_string();
    await_recoverable_task("text input", async move { process_text(&text_owned).await }).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::SessionHistoryEntry;
    use std::collections::VecDeque;
    use std::sync::Mutex;

    fn default_interactive_options() -> InteractiveOptions {
        InteractiveOptions {
            verbose: false,
            log_level: "info".to_string(),
            footer_output: true,
            attention_bell: false,
            attention_desktop_notification: false,
            attention_unfocused_only: false,
            predictive_input: true,
            ai_session_retention: 20,
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

    fn test_shared_state() -> SharedCliState {
        CliState::shared(
            "test-session",
            AppConfig::default(),
            SESSION_HISTORY_CAPACITY,
        )
    }

    fn test_runtime_context(interrupted: Arc<AtomicBool>) -> InputLoopRuntimeContext {
        InputLoopRuntimeContext {
            interrupted,
            state: test_shared_state(),
            attention: AttentionConfig::default(),
        }
    }

    fn snapshot_for_test(
        id: &str,
        path: &str,
        started_at_ms: u64,
        last_activity_ms: u64,
        history_entries: usize,
    ) -> LocalSessionSnapshot {
        LocalSessionSnapshot {
            id: id.to_string(),
            started_at_ms,
            last_activity_ms,
            history_entries,
            path: PathBuf::from(path),
        }
    }

    fn ai_snapshot_for_test(
        id: &str,
        path: &str,
        started_at_ms: u64,
        last_activity_ms: u64,
        exchange_count: usize,
    ) -> LocalAiSessionSnapshot {
        LocalAiSessionSnapshot {
            id: id.to_string(),
            started_at_ms,
            last_activity_ms,
            exchange_count,
            path: PathBuf::from(path),
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
        let status = interactive_mode_with_runner(default_interactive_options(), |_opts| async {
            Ok(ExitStatus::Success)
        })
        .await;
        assert_eq!(status, ExitStatus::Success);
    }

    #[tokio::test]
    async fn interactive_mode_with_runner_error_maps_to_error() {
        let status = interactive_mode_with_runner(default_interactive_options(), |_opts| async {
            Err(io::Error::other("synthetic interactive loop failure"))
        })
        .await;
        assert_eq!(status, ExitStatus::Error);
    }

    #[tokio::test]
    async fn run_input_loop_with_exit_result_returns_success() {
        let interrupted = Arc::new(AtomicBool::new(false));
        let mut raw_mode = FakeRawMode::default();
        let mut status_bar = StatusBar::new().with_input_backend("test");

        let status = run_input_loop_with(
            &mut raw_mode,
            &mut status_bar,
            test_runtime_context(interrupted),
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
        let interrupted = Arc::new(AtomicBool::new(false));
        let mut raw_mode = FakeRawMode::default();
        let mut status_bar = StatusBar::new().with_input_backend("test");

        let status = run_input_loop_with(
            &mut raw_mode,
            &mut status_bar,
            test_runtime_context(interrupted),
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
        let interrupted = Arc::new(AtomicBool::new(false));
        let mut raw_mode = FakeRawMode::default();
        let mut status_bar = StatusBar::new().with_input_backend("test");

        let status = run_input_loop_with(
            &mut raw_mode,
            &mut status_bar,
            test_runtime_context(interrupted),
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
        let interrupted = Arc::new(AtomicBool::new(false));
        let mut raw_mode = FakeRawMode::default();
        let mut status_bar = StatusBar::new().with_input_backend("test");
        let mut menu_calls = 0usize;

        let status = run_input_loop_with(
            &mut raw_mode,
            &mut status_bar,
            test_runtime_context(interrupted),
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
        let interrupted = Arc::new(AtomicBool::new(false));
        let mut raw_mode = FakeRawMode::default();
        let mut status_bar = StatusBar::new().with_input_backend("test");

        let status = run_input_loop_with(
            &mut raw_mode,
            &mut status_bar,
            test_runtime_context(interrupted),
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
            AttentionConfig::default(),
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
            AttentionConfig::default(),
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
    fn is_ai_resume_command_matches_expected_alias() {
        assert!(is_ai_resume_command("/ai resume"));
        assert!(is_ai_resume_command(" /AI   RESUME "));
        assert!(!is_ai_resume_command("/ai ask"));
        assert!(!is_ai_resume_command("/ai resume session-a"));
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

    #[test]
    fn initialize_runtime_state_resumes_latest_local_snapshot() {
        let mut local_state = CliState::new("persisted-session", AppConfig::default(), 8);
        local_state.push_command("/help");

        let options = default_interactive_options();
        let resumed =
            initialize_runtime_state_with_loader(&options, || Ok(Some(local_state.clone())));
        let guard = resumed.read().expect("read lock");

        assert_eq!(guard.session.id, "persisted-session");
        assert_eq!(guard.history.len(), 1);
        assert_eq!(
            guard.config.general.predictive_input,
            options.predictive_input
        );
    }

    #[test]
    fn build_session_resume_options_formats_label_and_description() {
        let snapshots = vec![
            snapshot_for_test("session-a", "a.json", 10, 15, 3),
            snapshot_for_test("session-b", "b.json", 20, 25, 5),
        ];

        let options = build_session_resume_options(&snapshots);
        assert_eq!(options.len(), 2);
        assert_eq!(options[0].label, "#01 session-a");
        assert_eq!(options[1].label, "#02 session-b");
        assert!(options[0].description.contains("entries:3"));
        assert!(options[1].description.contains("last:25"));
    }

    #[test]
    fn build_ai_session_resume_options_formats_label_and_description() {
        let snapshots = vec![
            ai_snapshot_for_test("ai-a", "a.json", 10, 15, 3),
            ai_snapshot_for_test("ai-b", "b.json", 20, 25, 5),
        ];

        let options = build_ai_session_resume_options(&snapshots);
        assert_eq!(options.len(), 2);
        assert_eq!(options[0].label, "#01 ai-a");
        assert_eq!(options[1].label, "#02 ai-b");
        assert!(options[0].description.contains("exchanges:3"));
        assert!(options[1].description.contains("last:25"));
    }

    #[test]
    fn select_local_session_with_picker_uses_user_selection() {
        let snapshots = vec![
            snapshot_for_test("session-a", "a.json", 10, 15, 3),
            snapshot_for_test("session-b", "b.json", 20, 25, 5),
        ];
        let expected_path = snapshots[1].path.clone();

        let selected_state = select_local_session_with_picker_using(
            |_| Ok(Some(snapshots.clone())),
            |_items| Some(expected_path.clone()),
            |path| {
                assert_eq!(path, expected_path.as_path());
                Ok(CliState::new("picked-session", AppConfig::default(), 8))
            },
        )
        .expect("selected state");

        assert_eq!(selected_state.session.id, "picked-session");
    }

    #[test]
    fn select_local_session_with_picker_skips_single_snapshot() {
        let snapshots = vec![snapshot_for_test("session-a", "a.json", 10, 15, 3)];
        let mut picker_called = false;

        let selected_state = select_local_session_with_picker_using(
            |_| Ok(Some(snapshots.clone())),
            |_items| {
                picker_called = true;
                None
            },
            |_path| Err(io::Error::other("snapshot read must not be called")),
        );

        assert!(selected_state.is_none());
        assert!(!picker_called);
    }

    #[test]
    fn select_local_ai_session_with_picker_uses_user_selection() {
        let snapshots = vec![
            ai_snapshot_for_test("ai-a", "a.json", 10, 15, 3),
            ai_snapshot_for_test("ai-b", "b.json", 20, 25, 5),
        ];
        let expected_path = snapshots[1].path.clone();

        let selected_id = select_local_ai_session_with_picker_using(
            |_| Ok(Some(snapshots.clone())),
            |_items| Some(expected_path.clone()),
            |path| {
                assert_eq!(path, expected_path.as_path());
                let mut state = LocalAiSessionState::new("picked-ai");
                state.started_at_ms = 50;
                state.last_activity_ms = 100;
                Ok(state)
            },
            |id| id.to_string(),
        )
        .expect("selected ai session id");

        assert_eq!(selected_id, "picked-ai");
    }

    #[test]
    fn select_local_ai_session_with_picker_skips_empty_list() {
        let selected_id = select_local_ai_session_with_picker_using(
            |_| Ok(Some(Vec::new())),
            |_items| None,
            |_path| Err(io::Error::other("read should not be called")),
            |id| id.to_string(),
        );

        assert!(selected_id.is_none());
    }

    #[test]
    fn seed_session_history_from_state_uses_existing_entries() {
        let mut state = CliState::new("seed-session", AppConfig::default(), 8);
        state.history.push_back(SessionHistoryEntry::new(
            HistoryEntryKind::Command,
            "/manifest list",
        ));
        state.history.push_back(SessionHistoryEntry::new(
            HistoryEntryKind::Text,
            "hello world",
        ));
        let shared = state.into_shared();

        let seeded = seed_session_history_from_state(&shared);
        assert_eq!(seeded.len(), 2);
        assert_eq!(seeded[0], "/manifest list");
        assert_eq!(seeded[1], "hello world");
    }

    #[test]
    fn persist_runtime_state_snapshot_with_io_runs_save_and_prune_paths() {
        let shared = CliState::shared("persist-file", AppConfig::default(), 8);
        let mut save_called = false;
        let mut prune_called = false;

        persist_runtime_state_snapshot_with_io(
            &shared,
            |state| {
                save_called = true;
                assert_eq!(state.session.id, "persist-file");
                Ok(Some(std::path::PathBuf::from("session.json")))
            },
            |keep_latest| {
                prune_called = true;
                assert_eq!(keep_latest, LOCAL_SESSION_RETENTION);
                Ok(Some(0))
            },
        );

        assert!(save_called);
        assert!(prune_called);
    }

    #[test]
    fn recoverable_input_error_retries_before_exhaustion() {
        let mut status_bar = StatusBar::new().with_input_backend("test");
        let mut consecutive_errors = 0usize;

        let can_recover = register_recoverable_input_error(
            &mut status_bar,
            "rustyline",
            &io::Error::other("synthetic input failure"),
            &mut consecutive_errors,
        );

        assert!(can_recover);
        assert_eq!(consecutive_errors, 1);
        assert_eq!(
            status_bar.latest_notification_message(),
            Some("rustyline input recovery 1/3")
        );
    }

    #[test]
    fn recoverable_input_error_stops_after_threshold() {
        let mut status_bar = StatusBar::new().with_input_backend("test");
        let mut consecutive_errors = 0usize;

        for attempt in 1..=MAX_CONSECUTIVE_INPUT_ERRORS {
            let can_recover = register_recoverable_input_error(
                &mut status_bar,
                "legacy",
                &io::Error::other(format!("synthetic input failure #{attempt}")),
                &mut consecutive_errors,
            );

            if attempt < MAX_CONSECUTIVE_INPUT_ERRORS {
                assert!(can_recover);
            } else {
                assert!(!can_recover);
            }
        }

        assert_eq!(consecutive_errors, MAX_CONSECUTIVE_INPUT_ERRORS);
        assert_eq!(
            status_bar.latest_notification_message(),
            Some("legacy input recovery exhausted")
        );
    }

    #[tokio::test]
    async fn await_recoverable_task_returns_success_for_successful_task() {
        let status = await_recoverable_task("ok-task", async { ExitStatus::Success }).await;
        assert_eq!(status, ExitStatus::Success);
    }

    #[tokio::test]
    async fn await_recoverable_task_converts_panic_to_error() {
        let status = await_recoverable_task("panic-task", async {
            panic!("synthetic panic for recovery test");
        })
        .await;
        assert_eq!(status, ExitStatus::Error);
    }
}
