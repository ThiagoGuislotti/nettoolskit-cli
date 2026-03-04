//! Desktop notification helpers for interactive attention signals.
//!
//! This module provides a minimal cross-platform abstraction for emitting
//! desktop notifications when interactive commands fail or are interrupted.

use std::io;

trait DesktopNotificationBackend {
    fn show(&self, title: &str, body: &str) -> Result<(), String>;
}

struct PlatformDesktopNotificationBackend;

impl DesktopNotificationBackend for PlatformDesktopNotificationBackend {
    fn show(&self, title: &str, body: &str) -> Result<(), String> {
        emit_platform_desktop_notification(title, body)
    }
}

/// Emit a desktop notification with title/body content.
///
/// Returns an error when title/body are empty or when the platform backend
/// fails to deliver the notification.
pub fn emit_desktop_attention_notification(title: &str, body: &str) -> io::Result<()> {
    emit_with_backend(&PlatformDesktopNotificationBackend, title, body)
}

fn emit_with_backend<B: DesktopNotificationBackend>(
    backend: &B,
    title: &str,
    body: &str,
) -> io::Result<()> {
    let title = title.trim();
    let body = body.trim();

    if title.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "desktop notification title cannot be empty",
        ));
    }

    if body.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "desktop notification body cannot be empty",
        ));
    }

    backend.show(title, body).map_err(io::Error::other)
}

#[cfg(target_os = "windows")]
fn emit_platform_desktop_notification(title: &str, body: &str) -> Result<(), String> {
    use winrt_notification::{Duration, Sound, Toast};

    Toast::new(Toast::POWERSHELL_APP_ID)
        .title(title)
        .text1(body)
        .duration(Duration::Short)
        .sound(Some(Sound::Default))
        .show()
        .map_err(|err| err.to_string())
}

#[cfg(target_os = "macos")]
fn emit_platform_desktop_notification(title: &str, body: &str) -> Result<(), String> {
    use std::process::Command;

    let body = body.replace('\\', "\\\\").replace('"', "\\\"");
    let title = title.replace('\\', "\\\\").replace('"', "\\\"");
    let script = format!("display notification \"{body}\" with title \"{title}\"");

    let status = Command::new("osascript")
        .arg("-e")
        .arg(script)
        .status()
        .map_err(|err| err.to_string())?;

    if status.success() {
        Ok(())
    } else {
        Err(format!("osascript exited with status: {status}"))
    }
}

#[cfg(all(unix, not(target_os = "macos")))]
fn emit_platform_desktop_notification(title: &str, body: &str) -> Result<(), String> {
    use std::process::Command;

    let status = Command::new("notify-send")
        .arg("--app-name")
        .arg("NetToolsKit CLI")
        .arg(title)
        .arg(body)
        .status()
        .map_err(|err| err.to_string())?;

    if status.success() {
        Ok(())
    } else {
        Err(format!("notify-send exited with status: {status}"))
    }
}

#[cfg(not(any(target_os = "windows", unix)))]
fn emit_platform_desktop_notification(_title: &str, _body: &str) -> Result<(), String> {
    Err("desktop notifications are not supported on this platform".to_string())
}

#[cfg(test)]
mod tests {
    use super::{emit_with_backend, DesktopNotificationBackend};

    #[derive(Clone, Copy)]
    struct FakeBackend {
        fail: bool,
    }

    impl DesktopNotificationBackend for FakeBackend {
        fn show(&self, _title: &str, _body: &str) -> Result<(), String> {
            if self.fail {
                Err("backend failure".to_string())
            } else {
                Ok(())
            }
        }
    }

    #[test]
    fn emit_with_backend_rejects_empty_title() {
        let result = emit_with_backend(&FakeBackend { fail: false }, "  ", "message");
        assert!(result.is_err());
    }

    #[test]
    fn emit_with_backend_rejects_empty_body() {
        let result = emit_with_backend(&FakeBackend { fail: false }, "title", "   ");
        assert!(result.is_err());
    }

    #[test]
    fn emit_with_backend_propagates_backend_error() {
        let result = emit_with_backend(&FakeBackend { fail: true }, "title", "message");
        assert!(result.is_err());
    }

    #[test]
    fn emit_with_backend_accepts_valid_notification() {
        let result = emit_with_backend(&FakeBackend { fail: false }, "title", "message");
        assert!(result.is_ok());
    }
}
