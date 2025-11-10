//! Event system for the interactive CLI.
//!
//! This module provides an event-driven architecture for handling terminal
//! input and command processing without blocking the main event loop.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::fmt;
use tokio::sync::mpsc;

/// Events that can occur in the CLI session.
#[derive(Debug, Clone)]
pub enum CliEvent {
    /// A key was pressed in the terminal
    Key(KeyEvent),

    /// A complete command was entered
    Command(String),

    /// Plain text input (not a command)
    Text(String),

    /// Request to exit the application
    Exit,

    /// Ctrl+C was pressed
    Interrupt,
}

impl fmt::Display for CliEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CliEvent::Key(key) => write!(f, "Key({:?})", key.code),
            CliEvent::Command(cmd) => write!(f, "Command({})", cmd),
            CliEvent::Text(text) => write!(f, "Text({})", text),
            CliEvent::Exit => write!(f, "Exit"),
            CliEvent::Interrupt => write!(f, "Interrupt"),
        }
    }
}

/// Sender for CLI events.
///
/// This is a thin wrapper around `mpsc::UnboundedSender` that provides
/// a more ergonomic API for sending events.
#[derive(Clone)]
pub struct EventSender {
    tx: mpsc::UnboundedSender<CliEvent>,
}

impl EventSender {
    /// Create a new event sender from an unbounded sender.
    pub fn new(tx: mpsc::UnboundedSender<CliEvent>) -> Self {
        Self { tx }
    }

    /// Send an event, returning true if successful.
    pub fn send(&self, event: CliEvent) -> bool {
        self.tx.send(event).is_ok()
    }

    /// Send a key event.
    pub fn send_key(&self, key: KeyEvent) -> bool {
        self.send(CliEvent::Key(key))
    }

    /// Send a command event.
    pub fn send_command(&self, cmd: impl Into<String>) -> bool {
        self.send(CliEvent::Command(cmd.into()))
    }

    /// Send a text event.
    pub fn send_text(&self, text: impl Into<String>) -> bool {
        self.send(CliEvent::Text(text.into()))
    }

    /// Send an exit event.
    pub fn send_exit(&self) -> bool {
        self.send(CliEvent::Exit)
    }

    /// Send an interrupt event.
    pub fn send_interrupt(&self) -> bool {
        self.send(CliEvent::Interrupt)
    }
}

/// Check if a key event is Ctrl+C.
pub fn is_ctrl_c(key: &KeyEvent) -> bool {
    matches!(
        key,
        KeyEvent {
            code: KeyCode::Char('c'),
            modifiers: KeyModifiers::CONTROL,
            ..
        }
    )
}

/// Check if a key event is Ctrl+D (EOF).
pub fn is_ctrl_d(key: &KeyEvent) -> bool {
    matches!(
        key,
        KeyEvent {
            code: KeyCode::Char('d'),
            modifiers: KeyModifiers::CONTROL,
            ..
        }
    )
}

/// Check if a key event is Enter.
pub fn is_enter(key: &KeyEvent) -> bool {
    matches!(key.code, KeyCode::Enter)
}

/// Check if a key event is Escape.
pub fn is_escape(key: &KeyEvent) -> bool {
    matches!(key.code, KeyCode::Esc)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_sender() {
        let (tx, mut rx) = mpsc::unbounded_channel();
        let sender = EventSender::new(tx);

        sender.send_command("test");

        let event = rx.blocking_recv().unwrap();
        match event {
            CliEvent::Command(cmd) => assert_eq!(cmd, "test"),
            _ => panic!("Expected Command event"),
        }
    }

    #[test]
    fn test_is_ctrl_c() {
        let key = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        assert!(is_ctrl_c(&key));

        let key = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::NONE);
        assert!(!is_ctrl_c(&key));
    }

    #[test]
    fn test_is_enter() {
        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        assert!(is_enter(&key));

        let key = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        assert!(!is_enter(&key));
    }
}
