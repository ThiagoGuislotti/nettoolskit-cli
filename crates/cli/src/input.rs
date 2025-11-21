use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use nettoolskit_core::async_utils::with_timeout;
use nettoolskit_ui::{
    append_footer_log, handle_resize, render_prompt_with_command, CommandPalette,
};
use owo_colors::OwoColorize;
use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::MenuContext;

#[derive(Debug)]
pub enum InputResult {
    Command(String),
    Text(String),
    Exit,
}

pub async fn read_line_with_palette(
    buffer: &mut String,
    palette: &mut CommandPalette,
    current_context: &mut MenuContext,
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
                    match handle_key_event(key_event, buffer, palette, current_context, interrupted)? {
                        Some(result) => return Ok(result),
                        None => continue,
                    }
                }
                Event::Resize(width, height) => {
                    if let Err(err) = handle_resize(width, height) {
                        let _ =
                            append_footer_log(&format!("Warning: failed to handle resize: {err}"));
                    }
                    if palette.is_active() {
                        palette.close()?;
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
    palette: &mut CommandPalette,
    current_context: &mut MenuContext,
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

            if c == '/' && buffer.len() == 1 {
                palette.open("")?;
            } else if palette.is_active() && buffer.starts_with('/') && buffer.len() > 1 {
                // Detect new context based on buffer
                let new_context = detect_context(buffer);

                // Switch menu if context changed
                if *current_context != new_context {
                    switch_menu_entries(palette, new_context)?;
                    *current_context = new_context;
                }

                // Extract query based on current context
                let query = extract_query(buffer);
                palette.update_query(query)?;
            }
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
        }
        KeyCode::Enter => {
            println!();
            if palette.is_active() {
                let selected_cmd = palette.get_selected_command().map(|s| s.to_string());
                palette.close()?;
                *current_context = MenuContext::Root;
                if let Some(cmd) = selected_cmd {
                    return Ok(Some(InputResult::Command(cmd)));
                }
            }

            let result = if buffer.starts_with('/') {
                InputResult::Command(buffer.clone())
            } else {
                InputResult::Text(buffer.clone())
            };
            return Ok(Some(result));
        }
        KeyCode::Tab if palette.is_active() => {
            let selected_cmd = palette.get_selected_command().map(|s| s.to_string());
            palette.close()?;
            if let Some(cmd) = selected_cmd {
                render_prompt_with_command(&cmd)?;
                buffer.clear();
                buffer.push_str(&cmd);
            }
        }
        KeyCode::Esc if palette.is_active() => {
            palette.close()?;
            *current_context = MenuContext::Root;
        }
        KeyCode::Up if palette.is_active() => {
            palette.navigate_up()?;
        }
        KeyCode::Down if palette.is_active() => {
            palette.navigate_down()?;
        }
        _ => {}
    }

    Ok(None)
}

/// Detects menu context based on buffer content
fn detect_context(buffer: &str) -> MenuContext {
    if buffer.starts_with("/manifest ") {
        MenuContext::Manifest
    } else {
        MenuContext::Root
    }
}

/// Extracts query string based on context
fn extract_query(buffer: &str) -> &str {
    if let Some(rest) = buffer.strip_prefix("/manifest ") {
        rest
    } else if let Some(rest) = buffer.strip_prefix("/") {
        rest
    } else {
        ""
    }
}

/// Switches menu entries based on context (CLI layer provides entries to UI)
fn switch_menu_entries(
    palette: &mut CommandPalette,
    context: MenuContext,
) -> io::Result<()> {
    match context {
        MenuContext::Root => {
            let entries = nettoolskit_commands::menu_entries();
            palette.reload_entries(entries)?;
        }
        MenuContext::Manifest => {
            let entries = nettoolskit_commands::nettoolskit_manifest::menu_entries();
            palette.reload_entries(entries)?;
        }
    }
    Ok(())
}
