use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use nettoolskit_async_utils::with_timeout;
use nettoolskit_ui::CommandPalette;
use std::io::{self, Write};

#[derive(Debug)]
pub enum InputResult {
    Command(String),
    Text(String),
    Exit,
}

pub async fn read_line_with_palette(
    buffer: &mut String,
    palette: &mut CommandPalette,
) -> io::Result<InputResult> {
    loop {
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
                Event::Key(key_event) => match handle_key_event(key_event, buffer, palette)? {
                    Some(result) => return Ok(result),
                    None => continue,
                },
                Event::Resize(_, _) => {
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
) -> io::Result<Option<InputResult>> {
    if key.kind != KeyEventKind::Press {
        return Ok(None);
    }

    match key.code {
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            return Ok(Some(InputResult::Exit));
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
        }
        KeyCode::Backspace => {
            if !buffer.is_empty() {
                buffer.pop();
                print!("\x08 \x08");
                io::stdout().flush()?;

                if palette.is_active() {
                    if buffer.starts_with('/') && buffer.len() > 0 {
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
                print!("\r\x1b[K> {}", cmd);
                io::stdout().flush()?;
                buffer.clear();
                buffer.push_str(&cmd);
            }
        }
        KeyCode::Esc if palette.is_active() => {
            palette.close()?;
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
