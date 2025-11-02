/// UI widgets for the modern TUI
///
/// This module provides rendering that maintains the exact same visual layout
/// as the legacy UI, but uses Ratatui for better performance and event handling.

use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph},
    Frame,
};

use crate::modern::App;

/// Render the complete UI - maintains legacy visual appearance
pub fn render_ui(frame: &mut Frame, app: &App) {
    // Render just the input line at the bottom (like legacy UI)
    // The rest of the screen shows the scrollback buffer naturally
    let input_line = render_input_line(app);

    // Calculate position for input line (last line of terminal)
    let area = frame.area();
    let input_area = Rect {
        x: 0,
        y: area.height.saturating_sub(1),
        width: area.width,
        height: 1,
    };

    frame.render_widget(input_line, input_area);
}

/// Render the input line exactly like legacy: "> {input}█"
fn render_input_line(app: &App) -> Paragraph<'static> {
    let input_text = format!("> {}", app.input);

    let line = Line::from(vec![
        Span::styled(
            input_text,
            Style::default().fg(Color::Rgb(155, 114, 255))
        ),
    ]);

    Paragraph::new(line).block(Block::default())
}