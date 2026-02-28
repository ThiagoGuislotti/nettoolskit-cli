use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use nettoolskit_core::async_utils::with_timeout;
use nettoolskit_ui::{append_footer_log, handle_resize, process_pending_resize};
use owo_colors::OwoColorize;
use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Result of an interactive input read operation.
#[derive(Debug)]
pub enum InputResult {
    /// User submitted a command prefixed with `/`.
    Command(String),
    /// User submitted free-form text (not a command).
    Text(String),
    /// User requested to exit (Ctrl-C / Ctrl-D).
    Exit,
    /// User pressed `F1` or `?` to open the interactive menu.
    ShowMenu,
}

/// Read a single line of interactive user input, handling keyboard events.
///
/// Returns an [`InputResult`] indicating whether the user entered a command,
/// free-form text, requested the menu, or chose to exit. The function also
/// processes terminal resize events while waiting for input.
pub async fn read_line(
    buffer: &mut String,
    interrupted: &Arc<AtomicBool>,
) -> io::Result<InputResult> {
    loop {
        // Check if interrupted before polling
        if interrupted.load(Ordering::SeqCst) {
            return Ok(InputResult::Exit);
        }

        // Use async-utils timeout for consistent timeout handling
        let poll_timeout = std::time::Duration::from_millis(50);

        match with_timeout(poll_timeout, async {
            while !event::poll(std::time::Duration::from_millis(1))? {
                tokio::time::sleep(std::time::Duration::from_millis(1)).await;
            }
            event::read()
        })
        .await
        {
            Ok(Ok(event)) => match event {
                Event::Key(key_event) => match handle_key_event(key_event, buffer, interrupted)? {
                    Some(result) => return Ok(result),
                    None => continue,
                },
                Event::Resize(width, height) => {
                    if let Err(err) = handle_resize(width, height) {
                        let _ =
                            append_footer_log(&format!("Warning: failed to handle resize: {err}"));
                    }
                }
                _ => {}
            },
            Ok(Err(e)) => return Err(e),
            Err(_) => {
                // Timeout — process any pending deferred resize before continuing
                if let Err(err) = process_pending_resize() {
                    let _ = append_footer_log(&format!("Warning: resize processing failed: {err}"));
                }
                continue;
            }
        }
    }
}

fn handle_key_event(
    key: KeyEvent,
    buffer: &mut String,
    interrupted: &Arc<AtomicBool>,
) -> io::Result<Option<InputResult>> {
    if key.kind != KeyEventKind::Press {
        return Ok(None);
    }

    match key.code {
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            // Set the interrupted flag and return Exit
            interrupted.store(true, Ordering::SeqCst);
            return Ok(Some(InputResult::Exit));
        }
        KeyCode::Char(c) => {
            buffer.push(c);
            print!("{}", c.to_string().white());
            io::stdout().flush()?;

            // If user types "/" as first character, show menu immediately
            if c == '/' && buffer.len() == 1 {
                return Ok(Some(InputResult::ShowMenu));
            }
        }
        KeyCode::Backspace => {
            if !buffer.is_empty() {
                buffer.pop();
                print!("\x08 \x08");
                io::stdout().flush()?;
            }
        }
        KeyCode::Enter => {
            println!();
            let result = if buffer.starts_with('/') {
                InputResult::Command(buffer.clone())
            } else {
                InputResult::Text(buffer.clone())
            };
            return Ok(Some(result));
        }
        _ => {}
    }

    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
    use std::sync::atomic::AtomicBool;
    use std::sync::Arc;

    /// Creates a `KeyEvent` with `Press` kind.
    fn press(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
        KeyEvent {
            code,
            modifiers,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }
    }

    /// Creates a `KeyEvent` with `Release` kind (should be ignored).
    fn release(code: KeyCode) -> KeyEvent {
        KeyEvent {
            code,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Release,
            state: KeyEventState::NONE,
        }
    }

    #[test]
    fn ctrl_c_returns_exit_and_sets_interrupted() {
        let mut buffer = String::new();
        let interrupted = Arc::new(AtomicBool::new(false));

        let result = handle_key_event(
            press(KeyCode::Char('c'), KeyModifiers::CONTROL),
            &mut buffer,
            &interrupted,
        )
        .expect("should not error");

        assert!(matches!(result, Some(InputResult::Exit)));
        assert!(interrupted.load(Ordering::SeqCst));
    }

    #[test]
    fn enter_with_slash_prefix_returns_command() {
        let mut buffer = String::from("/help");
        let interrupted = Arc::new(AtomicBool::new(false));

        let result = handle_key_event(
            press(KeyCode::Enter, KeyModifiers::NONE),
            &mut buffer,
            &interrupted,
        )
        .expect("should not error");

        assert!(matches!(result, Some(InputResult::Command(cmd)) if cmd == "/help"));
    }

    #[test]
    fn enter_without_slash_prefix_returns_text() {
        let mut buffer = String::from("hello world");
        let interrupted = Arc::new(AtomicBool::new(false));

        let result = handle_key_event(
            press(KeyCode::Enter, KeyModifiers::NONE),
            &mut buffer,
            &interrupted,
        )
        .expect("should not error");

        assert!(matches!(result, Some(InputResult::Text(txt)) if txt == "hello world"));
    }

    #[test]
    fn enter_with_empty_buffer_returns_text() {
        let mut buffer = String::new();
        let interrupted = Arc::new(AtomicBool::new(false));

        let result = handle_key_event(
            press(KeyCode::Enter, KeyModifiers::NONE),
            &mut buffer,
            &interrupted,
        )
        .expect("should not error");

        assert!(matches!(result, Some(InputResult::Text(txt)) if txt.is_empty()));
    }

    #[test]
    fn slash_as_first_char_returns_show_menu() {
        let mut buffer = String::new();
        let interrupted = Arc::new(AtomicBool::new(false));

        let result = handle_key_event(
            press(KeyCode::Char('/'), KeyModifiers::NONE),
            &mut buffer,
            &interrupted,
        )
        .expect("should not error");

        assert_eq!(buffer, "/");
        assert!(matches!(result, Some(InputResult::ShowMenu)));
    }

    #[test]
    fn slash_not_first_char_does_not_show_menu() {
        let mut buffer = String::from("a");
        let interrupted = Arc::new(AtomicBool::new(false));

        let result = handle_key_event(
            press(KeyCode::Char('/'), KeyModifiers::NONE),
            &mut buffer,
            &interrupted,
        )
        .expect("should not error");

        assert_eq!(buffer, "a/");
        assert!(result.is_none());
    }

    #[test]
    fn regular_char_is_appended_to_buffer() {
        let mut buffer = String::new();
        let interrupted = Arc::new(AtomicBool::new(false));

        let result = handle_key_event(
            press(KeyCode::Char('x'), KeyModifiers::NONE),
            &mut buffer,
            &interrupted,
        )
        .expect("should not error");

        assert_eq!(buffer, "x");
        assert!(result.is_none());
    }

    #[test]
    fn backspace_removes_last_char() {
        let mut buffer = String::from("abc");
        let interrupted = Arc::new(AtomicBool::new(false));

        let result = handle_key_event(
            press(KeyCode::Backspace, KeyModifiers::NONE),
            &mut buffer,
            &interrupted,
        )
        .expect("should not error");

        assert_eq!(buffer, "ab");
        assert!(result.is_none());
    }

    #[test]
    fn backspace_on_empty_buffer_is_no_op() {
        let mut buffer = String::new();
        let interrupted = Arc::new(AtomicBool::new(false));

        let result = handle_key_event(
            press(KeyCode::Backspace, KeyModifiers::NONE),
            &mut buffer,
            &interrupted,
        )
        .expect("should not error");

        assert!(buffer.is_empty());
        assert!(result.is_none());
    }

    #[test]
    fn release_event_is_ignored() {
        let mut buffer = String::new();
        let interrupted = Arc::new(AtomicBool::new(false));

        let result = handle_key_event(release(KeyCode::Char('a')), &mut buffer, &interrupted)
            .expect("should not error");

        assert!(buffer.is_empty());
        assert!(result.is_none());
    }

    #[test]
    fn unknown_key_code_returns_none() {
        let mut buffer = String::new();
        let interrupted = Arc::new(AtomicBool::new(false));

        let result = handle_key_event(
            press(KeyCode::F(1), KeyModifiers::NONE),
            &mut buffer,
            &interrupted,
        )
        .expect("should not error");

        assert!(buffer.is_empty());
        assert!(result.is_none());
    }

    #[test]
    fn multiple_chars_build_buffer_correctly() {
        let mut buffer = String::new();
        let interrupted = Arc::new(AtomicBool::new(false));

        for ch in ['h', 'e', 'l', 'l', 'o'] {
            handle_key_event(
                press(KeyCode::Char(ch), KeyModifiers::NONE),
                &mut buffer,
                &interrupted,
            )
            .expect("should not error");
        }

        assert_eq!(buffer, "hello");
    }
}
