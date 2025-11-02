/// Event handling for modern TUI
///
/// This module provides event-driven input handling using Ratatui's
/// event system, but maintains the exact same visual output as legacy UI.
///
/// Phase 1.3: Added event stream support for zero-CPU idle state

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use std::io::{self, Write};
use std::time::Duration;

#[cfg(feature = "modern-tui")]
use crossterm::event::EventStream;
#[cfg(feature = "modern-tui")]
use futures::StreamExt;

/// Event handler result
pub enum EventResult {
    /// Continue processing events
    Continue,
    /// Command submitted
    Command(String),
    /// Text submitted
    Text(String),
    /// Exit requested
    Exit,
}

/// Handle keyboard events with event stream (Phase 1.3 - zero CPU when idle)
#[cfg(feature = "modern-tui")]
pub async fn handle_events_stream(
    buffer: &mut String,
    palette: &mut crate::legacy::palette::CommandPalette,
    events: &mut EventStream,
) -> io::Result<EventResult> {
    // Use event stream - zero CPU when idle!
    match events.next().await {
        Some(Ok(Event::Key(key_event))) => handle_key_event(key_event, buffer, palette),
        Some(Ok(Event::Resize(width, height))) => {
            crate::legacy::terminal::handle_resize(width, height)?;
            if palette.is_active() {
                palette.close()?;
            }
            Ok(EventResult::Continue)
        }
        Some(Ok(_)) => Ok(EventResult::Continue),
        Some(Err(e)) => Err(e),
        None => Ok(EventResult::Exit), // Stream ended
    }
}

/// Handle keyboard events with polling (Phase 1.2 - 16ms polling)
pub async fn handle_events(
    buffer: &mut String,
    palette: &mut crate::legacy::palette::CommandPalette,
) -> io::Result<EventResult> {
    // Use Ratatui's event polling with timeout
    if !event::poll(Duration::from_millis(16))? {
        return Ok(EventResult::Continue);
    }

    match event::read()? {
        Event::Key(key_event) => handle_key_event(key_event, buffer, palette),
        Event::Resize(width, height) => {
            crate::legacy::terminal::handle_resize(width, height)?;
            if palette.is_active() {
                palette.close()?;
            }
            Ok(EventResult::Continue)
        }
        _ => Ok(EventResult::Continue),
    }
}

/// Handle key events exactly like legacy, but with better event handling
fn handle_key_event(
    key: KeyEvent,
    buffer: &mut String,
    palette: &mut crate::legacy::palette::CommandPalette,
) -> io::Result<EventResult> {
    if key.kind != KeyEventKind::Press {
        return Ok(EventResult::Continue);
    }

    match key.code {
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Ok(EventResult::Exit)
        }
        KeyCode::Char(c) => {
            buffer.push(c);
            print!("{}", c);
            io::stdout().flush()?;

            if c == '/' && buffer.len() == 1 {
                palette.open("")?;
            } else if palette.is_active() && buffer.starts_with('/') && buffer.len() > 1 {
                palette.update_query(&buffer[1..])?;
            }
            Ok(EventResult::Continue)
        }
        KeyCode::Backspace => {
            if !buffer.is_empty() {
                buffer.pop();
                print!("\x08 \x08");
                io::stdout().flush()?;

                if palette.is_active() {
                    if buffer.starts_with('/') && !buffer.is_empty() {
                        palette.update_query(&buffer[1..])?;
                    } else {
                        palette.close()?;
                    }
                }
            }
            Ok(EventResult::Continue)
        }
        KeyCode::Enter => {
            println!();
            if palette.is_active() {
                let selected_cmd = palette.get_selected_command().map(|s| s.to_string());
                palette.close()?;
                if let Some(cmd) = selected_cmd {
                    return Ok(EventResult::Command(cmd));
                }
            }

            let result = if buffer.starts_with('/') {
                EventResult::Command(buffer.clone())
            } else {
                EventResult::Text(buffer.clone())
            };
            Ok(result)
        }
        KeyCode::Tab if palette.is_active() => {
            let selected_cmd = palette.get_selected_command().map(|s| s.to_string());
            palette.close()?;
            if let Some(cmd) = selected_cmd {
                print!("\r\x1b[K> {}", cmd);
                io::stdout().flush()?;
                buffer.clear();
                buffer.push_str(&cmd);
            }
            Ok(EventResult::Continue)
        }
        KeyCode::Esc if palette.is_active() => {
            palette.close()?;
            Ok(EventResult::Continue)
        }
        KeyCode::Up if palette.is_active() => {
            palette.navigate_up()?;
            Ok(EventResult::Continue)
        }
        KeyCode::Down if palette.is_active() => {
            palette.navigate_down()?;
            Ok(EventResult::Continue)
        }
        _ => Ok(EventResult::Continue),
    }
}