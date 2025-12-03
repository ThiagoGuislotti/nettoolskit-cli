use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use nettoolskit_core::async_utils::with_timeout;
use nettoolskit_ui::{append_footer_log, handle_resize};
use owo_colors::OwoColorize;
use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[derive(Debug)]
pub enum InputResult {
    Command(String),
    Text(String),
    Exit,
    ShowMenu,
}

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
                Event::Key(key_event) => {
                    match handle_key_event(key_event, buffer, interrupted)? {
                        Some(result) => return Ok(result),
                        None => continue,
                    }
                }
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
                // Timeout - continue polling
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
