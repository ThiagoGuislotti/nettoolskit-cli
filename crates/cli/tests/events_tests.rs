//! Events Module Tests
//!
//! Tests for CLI event handling functionality.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use nettoolskit_cli::events::{is_ctrl_c, is_enter, EventSender, CliEvent};
use tokio::sync::mpsc;

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
