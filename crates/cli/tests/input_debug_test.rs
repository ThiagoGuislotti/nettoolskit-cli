//! Input debugging and validation tests
//!
//! This module provides manual testing utilities for validating
//! the terminal input system behavior.

use crossterm::event::{self, Event, KeyCode};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use nettoolskit_ui::render_prompt;
use std::io::{self, Write};

/// Manual test for debugging input flow
///
/// Run with: `cargo test --test input_debug_test -- --ignored --nocapture`
///
/// This test is marked as `#[ignore]` because it requires manual interaction
/// and doesn't run in CI/CD pipelines.
#[test]
#[ignore]
fn test_input_flow_manual() -> io::Result<()> {
    println!("=== INPUT DEBUG TEST ===");
    println!("Type something and press Enter");
    println!("Press Ctrl+C to exit\n");

    enable_raw_mode()?;

    let mut buffer = String::new();

    render_prompt()?;

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
                            render_prompt()?;
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
