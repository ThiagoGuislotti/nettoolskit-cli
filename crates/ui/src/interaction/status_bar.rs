//! Runtime status bar for interactive CLI sessions.
//!
//! This module renders a single-line status bar above the prompt with:
//! - current interaction mode
//! - input backend indicator
//! - notifications queue summary
//! - lightweight runtime/resource metadata

use crate::core::capabilities::capabilities;
use crate::interaction::terminal::prepare_prompt_line;
use crossterm::style::Print;
use crossterm::terminal::{self, Clear, ClearType};
use crossterm::{cursor, queue};
use nettoolskit_core::ExitStatus;
use std::collections::VecDeque;
use std::io::{self, Write};
use std::time::{Duration, Instant};

const DEFAULT_MAX_NOTIFICATIONS: usize = 8;
const DEFAULT_RENDER_WIDTH: usize = 120;

/// Current interaction mode shown in the status bar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusBarMode {
    /// Waiting for input.
    Ready,
    /// Command palette/menu is active.
    Menu,
    /// A command is being processed.
    Command,
    /// Free-form text routing is active.
    Text,
    /// Session is shutting down.
    Shutdown,
}

impl StatusBarMode {
    fn label(self) -> &'static str {
        match self {
            Self::Ready => "READY",
            Self::Menu => "MENU",
            Self::Command => "COMMAND",
            Self::Text => "TEXT",
            Self::Shutdown => "SHUTDOWN",
        }
    }
}

/// Notification severity used by the status bar queue.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusNotificationLevel {
    /// Informational notification.
    Info,
    /// Successful operation notification.
    Success,
    /// Warning notification.
    Warning,
    /// Error notification.
    Error,
}

impl StatusNotificationLevel {
    fn symbol(self, unicode: bool) -> &'static str {
        match (self, unicode) {
            (Self::Info, true) => "ℹ",
            (Self::Info, false) => "i",
            (Self::Success, true) => "✓",
            (Self::Success, false) => "+",
            (Self::Warning, true) => "⚠",
            (Self::Warning, false) => "!",
            (Self::Error, true) => "✖",
            (Self::Error, false) => "x",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct StatusNotification {
    level: StatusNotificationLevel,
    message: String,
}

/// Stateful status bar for the interactive loop.
#[derive(Debug, Clone)]
pub struct StatusBar {
    mode: StatusBarMode,
    input_backend: String,
    notifications: VecDeque<StatusNotification>,
    max_notifications: usize,
    started_at: Instant,
    commands_total: u64,
    commands_success: u64,
    commands_error: u64,
    commands_interrupted: u64,
    last_command_duration_ms: Option<u128>,
}

impl Default for StatusBar {
    fn default() -> Self {
        Self::new()
    }
}

impl StatusBar {
    /// Creates a new status bar with defaults.
    #[must_use]
    pub fn new() -> Self {
        Self {
            mode: StatusBarMode::Ready,
            input_backend: "legacy".to_string(),
            notifications: VecDeque::with_capacity(DEFAULT_MAX_NOTIFICATIONS),
            max_notifications: DEFAULT_MAX_NOTIFICATIONS,
            started_at: Instant::now(),
            commands_total: 0,
            commands_success: 0,
            commands_error: 0,
            commands_interrupted: 0,
            last_command_duration_ms: None,
        }
    }

    /// Sets the input backend label (`rustyline`, `legacy`, etc.).
    #[must_use]
    pub fn with_input_backend(mut self, input_backend: impl Into<String>) -> Self {
        let backend = input_backend.into();
        if !backend.trim().is_empty() {
            self.input_backend = backend;
        }
        self
    }

    /// Sets the maximum number of queued notifications.
    #[must_use]
    pub fn with_max_notifications(mut self, max_notifications: usize) -> Self {
        self.max_notifications = max_notifications.max(1);
        while self.notifications.len() > self.max_notifications {
            self.notifications.pop_front();
        }
        self
    }

    /// Updates the current status bar mode.
    pub fn set_mode(&mut self, mode: StatusBarMode) {
        self.mode = mode;
    }

    /// Returns the current mode.
    #[must_use]
    pub fn mode(&self) -> StatusBarMode {
        self.mode
    }

    /// Pushes a notification into the queue (bounded by max capacity).
    pub fn push_notification(
        &mut self,
        level: StatusNotificationLevel,
        message: impl Into<String>,
    ) {
        let message = message.into();
        if message.trim().is_empty() {
            return;
        }

        if self.notifications.len() == self.max_notifications {
            self.notifications.pop_front();
        }
        self.notifications
            .push_back(StatusNotification { level, message });
    }

    /// Returns number of currently queued notifications.
    #[must_use]
    pub fn notifications_queued(&self) -> usize {
        self.notifications.len()
    }

    /// Records a command result and updates counters/duration.
    pub fn record_command_result(&mut self, status: ExitStatus, duration: Duration) {
        self.commands_total = self.commands_total.saturating_add(1);
        self.last_command_duration_ms = Some(duration.as_millis());

        match status {
            ExitStatus::Success => {
                self.commands_success = self.commands_success.saturating_add(1);
            }
            ExitStatus::Error => {
                self.commands_error = self.commands_error.saturating_add(1);
            }
            ExitStatus::Interrupted => {
                self.commands_interrupted = self.commands_interrupted.saturating_add(1);
            }
        }
    }

    /// Returns latest notification message, if any.
    #[must_use]
    pub fn latest_notification_message(&self) -> Option<&str> {
        self.notifications.back().map(|item| item.message.as_str())
    }

    /// Formats the status bar line for a target width.
    #[must_use]
    pub fn format_line_for_width(&self, width: usize) -> String {
        let caps = capabilities();
        let unicode = caps.unicode;
        let separator = if unicode { " │ " } else { " | " };
        let terminal_dims = terminal::size().ok();
        let (term_w, term_h) = terminal_dims.unwrap_or((0, 0));
        let uptime_secs = self.started_at.elapsed().as_secs();
        let last_ms = self.last_command_duration_ms.unwrap_or(0);

        let latest_notification = self
            .notifications
            .back()
            .map(|note| format!("{} {}", note.level.symbol(unicode), note.message))
            .unwrap_or_else(|| "none".to_string());

        let segments = [
            format!("mode:{}", self.mode.label()),
            format!("input:{}", self.input_backend),
            format!(
                "cmd:{} ok:{} err:{} int:{} last:{}ms",
                self.commands_total,
                self.commands_success,
                self.commands_error,
                self.commands_interrupted,
                last_ms
            ),
            format!("notif:{} {}", self.notifications.len(), latest_notification),
            format!("res:{}x{} up:{}s", term_w, term_h, uptime_secs),
        ];

        let raw_line = segments.join(separator);
        let safe_width = width.max(1).saturating_sub(1);
        truncate_with_ellipsis(&raw_line, safe_width, unicode)
    }

    /// Renders the status bar line right above the prompt line.
    ///
    /// The line is truncated to terminal width and rendered in-place
    /// (no extra newline), avoiding duplicate status rows in scroll history.
    pub fn render(&self) -> io::Result<()> {
        prepare_prompt_line()?;

        let width = terminal::size()
            .map(|(w, _)| w as usize)
            .unwrap_or(DEFAULT_RENDER_WIDTH);
        let line = self.format_line_for_width(width);

        let mut stdout = io::stdout();
        let (_, current_row) = cursor::position().unwrap_or((0, 0));

        queue!(stdout, cursor::SavePosition)?;
        if current_row > 0 {
            queue!(stdout, cursor::MoveUp(1))?;
        }
        queue!(
            stdout,
            cursor::MoveToColumn(0),
            Clear(ClearType::CurrentLine),
            Print(&line),
            cursor::RestorePosition
        )?;
        stdout.flush()
    }
}

fn truncate_with_ellipsis(input: &str, width: usize, unicode: bool) -> String {
    if width == 0 {
        return String::new();
    }

    let char_count = input.chars().count();
    if char_count <= width {
        return input.to_string();
    }

    let ellipsis = if unicode { "…" } else { "..." };
    let ellipsis_len = ellipsis.chars().count();
    if width <= ellipsis_len {
        return ellipsis.chars().take(width).collect();
    }

    let keep_chars = width - ellipsis_len;
    let prefix: String = input.chars().take(keep_chars).collect();
    format!("{prefix}{ellipsis}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn notification_queue_is_bounded() {
        let mut status = StatusBar::new().with_max_notifications(2);
        status.push_notification(StatusNotificationLevel::Info, "one");
        status.push_notification(StatusNotificationLevel::Warning, "two");
        status.push_notification(StatusNotificationLevel::Error, "three");

        assert_eq!(status.notifications_queued(), 2);
        assert_eq!(status.latest_notification_message(), Some("three"));
    }

    #[test]
    fn command_result_updates_counters() {
        let mut status = StatusBar::new();
        status.record_command_result(ExitStatus::Success, Duration::from_millis(10));
        status.record_command_result(ExitStatus::Error, Duration::from_millis(25));
        status.record_command_result(ExitStatus::Interrupted, Duration::from_millis(40));

        assert_eq!(status.commands_total, 3);
        assert_eq!(status.commands_success, 1);
        assert_eq!(status.commands_error, 1);
        assert_eq!(status.commands_interrupted, 1);
        assert_eq!(status.last_command_duration_ms, Some(40));
    }

    #[test]
    fn format_line_contains_required_segments() {
        let mut status = StatusBar::new().with_input_backend("rustyline");
        status.set_mode(StatusBarMode::Command);
        status.push_notification(StatusNotificationLevel::Success, "done");
        status.record_command_result(ExitStatus::Success, Duration::from_millis(12));

        let line = status.format_line_for_width(200);
        assert!(line.contains("mode:COMMAND"));
        assert!(line.contains("input:rustyline"));
        assert!(line.contains("cmd:1"));
        assert!(line.contains("notif:1"));
        assert!(line.contains("res:"));
    }

    #[test]
    fn format_line_respects_requested_width() {
        let mut status = StatusBar::new();
        status.push_notification(
            StatusNotificationLevel::Warning,
            "this is a long notification message to force truncation",
        );

        let line = status.format_line_for_width(40);
        assert!(line.chars().count() <= 39);
    }

    #[test]
    fn truncate_with_ellipsis_handles_narrow_width() {
        assert_eq!(truncate_with_ellipsis("abcdef", 1, true), "…");
        assert_eq!(truncate_with_ellipsis("abcdef", 2, false), "..");
        assert_eq!(truncate_with_ellipsis("abcdef", 3, false), "...");
    }
}
