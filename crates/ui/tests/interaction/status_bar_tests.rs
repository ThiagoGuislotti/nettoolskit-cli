use nettoolskit_core::ExitStatus;
use nettoolskit_ui::{StatusBar, StatusBarMode, StatusNotificationLevel};
use std::time::Duration;

#[test]
fn status_bar_mode_updates() {
    let mut status = StatusBar::new();
    status.set_mode(StatusBarMode::Menu);
    assert_eq!(status.mode(), StatusBarMode::Menu);
}

#[test]
fn status_bar_records_command_results() {
    let mut status = StatusBar::new();
    status.record_command_result(ExitStatus::Success, Duration::from_millis(10));
    status.record_command_result(ExitStatus::Error, Duration::from_millis(15));

    let line = status.format_line_for_width(240);
    assert!(line.contains("cmd:2"));
    assert!(line.contains("ok:1"));
    assert!(line.contains("err:1"));
}

#[test]
fn status_bar_formats_notification_queue() {
    let mut status = StatusBar::new().with_max_notifications(2);
    status.push_notification(StatusNotificationLevel::Info, "one");
    status.push_notification(StatusNotificationLevel::Warning, "two");
    status.push_notification(StatusNotificationLevel::Error, "three");

    assert_eq!(status.notifications_queued(), 2);
    assert_eq!(status.latest_notification_message(), Some("three"));
}

#[test]
fn status_bar_format_is_width_bounded() {
    let mut status = StatusBar::new().with_input_backend("rustyline");
    status.set_mode(StatusBarMode::Command);
    status.push_notification(
        StatusNotificationLevel::Warning,
        "very long message to force truncation on narrow widths",
    );

    let line = status.format_line_for_width(30);
    assert!(line.chars().count() <= 29);
}
