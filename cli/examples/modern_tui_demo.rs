/// Example demonstrating the modern Ratatui-based TUI
///
/// Run with: cargo run --features modern-tui --example modern_tui_demo

use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use nettoolskit_ui::modern::{render_ui, App, Tui};
use std::io;
use std::time::Duration;

fn main() -> io::Result<()> {
    // Create TUI and App
    let mut tui = Tui::new()?;
    let mut app = App::new();

    // Enter TUI mode
    tui.enter()?;

    // Set initial status
    app.set_status("Welcome to Modern TUI! Type something and press Enter");

    // Main event loop
    loop {
        // Draw the UI
        tui.draw(|frame| render_ui(frame, &app))?;

        // Handle events
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        break;
                    }
                    KeyCode::Char(c) => {
                        app.on_char(c);
                    }
                    KeyCode::Backspace => {
                        app.on_backspace();
                    }
                    KeyCode::Enter => {
                        if let Some(command) = app.on_submit() {
                            if command == "/quit" {
                                break;
                            }
                            app.set_status(format!("You typed: {}", command));
                        }
                    }
                    KeyCode::Up => {
                        app.history_up();
                    }
                    KeyCode::Down => {
                        app.history_down();
                    }
                    KeyCode::Esc => {
                        app.quit();
                    }
                    _ => {}
                }

                if app.should_quit {
                    break;
                }
            }
        }
    }

    // Exit TUI mode
    tui.exit()?;

    println!("Thanks for using NetToolsKit Modern TUI!");
    Ok(())
}