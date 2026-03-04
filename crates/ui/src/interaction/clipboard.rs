//! Clipboard helpers for terminal interactions.
//!
//! This module wraps `arboard` so CLI code can copy/paste text with
//! platform-neutral `io::Result` error handling.

use std::io;

trait ClipboardBackend {
    fn set_text(&mut self, text: String) -> Result<(), String>;
    fn get_text(&mut self) -> Result<String, String>;
}

struct ArboardBackend {
    inner: arboard::Clipboard,
}

impl ArboardBackend {
    fn new() -> io::Result<Self> {
        let inner = arboard::Clipboard::new().map_err(|err| io::Error::other(err.to_string()))?;
        Ok(Self { inner })
    }
}

impl ClipboardBackend for ArboardBackend {
    fn set_text(&mut self, text: String) -> Result<(), String> {
        self.inner.set_text(text).map_err(|err| err.to_string())
    }

    fn get_text(&mut self) -> Result<String, String> {
        self.inner.get_text().map_err(|err| err.to_string())
    }
}

/// Copy text into the system clipboard.
///
/// Returns an error when text is empty/blank or clipboard access fails.
pub fn copy_to_clipboard(text: &str) -> io::Result<()> {
    let mut backend = ArboardBackend::new()?;
    copy_with_backend(&mut backend, text)
}

/// Paste text from the system clipboard.
///
/// Returns an error when clipboard is empty or unavailable.
pub fn paste_from_clipboard() -> io::Result<String> {
    let mut backend = ArboardBackend::new()?;
    paste_with_backend(&mut backend)
}

fn copy_with_backend<B: ClipboardBackend>(backend: &mut B, text: &str) -> io::Result<()> {
    if text.trim().is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "cannot copy empty text to clipboard",
        ));
    }

    backend.set_text(text.to_string()).map_err(io::Error::other)
}

fn paste_with_backend<B: ClipboardBackend>(backend: &mut B) -> io::Result<String> {
    let value = backend.get_text().map_err(io::Error::other)?;
    if value.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "clipboard is empty",
        ));
    }
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::{copy_with_backend, paste_with_backend, ClipboardBackend};

    #[derive(Default)]
    struct FakeBackend {
        stored: Option<String>,
        fail_set: bool,
        fail_get: bool,
    }

    impl FakeBackend {
        fn with_text(value: &str) -> Self {
            Self {
                stored: Some(value.to_string()),
                fail_set: false,
                fail_get: false,
            }
        }
    }

    impl ClipboardBackend for FakeBackend {
        fn set_text(&mut self, text: String) -> Result<(), String> {
            if self.fail_set {
                return Err("set failed".to_string());
            }
            self.stored = Some(text);
            Ok(())
        }

        fn get_text(&mut self) -> Result<String, String> {
            if self.fail_get {
                return Err("get failed".to_string());
            }
            Ok(self.stored.clone().unwrap_or_default())
        }
    }

    #[test]
    fn copy_with_backend_rejects_blank_text() {
        let mut backend = FakeBackend::default();
        let result = copy_with_backend(&mut backend, "   ");
        assert!(result.is_err());
    }

    #[test]
    fn copy_with_backend_writes_text() {
        let mut backend = FakeBackend::default();
        let result = copy_with_backend(&mut backend, "hello");
        assert!(result.is_ok());
        assert_eq!(backend.stored.as_deref(), Some("hello"));
    }

    #[test]
    fn paste_with_backend_returns_stored_value() {
        let mut backend = FakeBackend::with_text("clipboard-value");
        let result = paste_with_backend(&mut backend);
        assert_eq!(
            result.expect("clipboard text should exist"),
            "clipboard-value"
        );
    }

    #[test]
    fn paste_with_backend_rejects_empty_clipboard() {
        let mut backend = FakeBackend::default();
        let result = paste_with_backend(&mut backend);
        assert!(result.is_err());
    }
}
