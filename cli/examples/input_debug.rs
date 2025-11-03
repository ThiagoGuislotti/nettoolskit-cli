use crossterm::event::{self, Event, KeyCode, KeyEvent};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
/// Quick debug test to check input flow
///
/// Run this to see what's happening with the input buffer
use std::io::{self, Write};

fn main() -> io::Result<()> {
    println!("=== INPUT DEBUG TEST ===");
    println!("Type something and press Enter");
    println!("Press Ctrl+C to exit\n");

    enable_raw_mode()?;

    let mut buffer = String::new();

    print!("> ");
    io::stdout().flush()?;

    loop {
        if event::poll(std::time::Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key_event) => match key_event.code {
                    KeyCode::Char('c')
                        if key_event
                            .modifiers
                            .contains(crossterm::event::KeyModifiers::CONTROL) =>
                    {
                        disable_raw_mode()?;
                        println!("\n\nExiting...");
                        break;
                    }
                    KeyCode::Char(c) => {
                        buffer.push(c);
                        print!("{}", c);
                        io::stdout().flush()?;

                        eprintln!("\r\nDEBUG: Added '{}', buffer now: '{}'", c, buffer);
                    }
                    KeyCode::Backspace => {
                        if !buffer.is_empty() {
                            buffer.pop();
                            print!("\x08 \x08");
                            io::stdout().flush()?;

                            eprintln!("\r\nDEBUG: Backspace, buffer now: '{}'", buffer);
                        }
                    }
                    KeyCode::Enter => {
                        println!();
                        eprintln!("DEBUG: Enter pressed, buffer: '{}'", buffer);

                        if !buffer.is_empty() {
                            disable_raw_mode()?;
                            println!("You typed: '{}'", buffer);
                            enable_raw_mode()?;

                            buffer.clear();
                            print!("> ");
                            io::stdout().flush()?;
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }

    disable_raw_mode()?;
    Ok(())
}
