use crossterm::style::{Color, SetForegroundColor};
use owo_colors::Rgb;

/// Convert `owo_colors::Rgb` to `crossterm::style::Color`.
///
/// Provides a bridge between the two color representation systems:
/// - `owo_colors::Rgb(r, g, b)` used for trait-based styling
/// - `crossterm::style::Color::Rgb { r, g, b }` used for terminal commands
///
/// # Examples
///
/// ```
/// use nettoolskit_ui::style::rgb_to_crossterm;
/// use owo_colors::Rgb;
///
/// let color = Rgb(155, 114, 255);
/// let crossterm_color = rgb_to_crossterm(color);
/// ```
#[inline]
pub fn rgb_to_crossterm(rgb: Rgb) -> Color {
    Color::Rgb {
        r: rgb.0,
        g: rgb.1,
        b: rgb.2,
    }
}

/// Create a `SetForegroundColor` command from an `Rgb` color.
///
/// This is a convenience function that combines RGB conversion with
/// the creation of a crossterm styling command. Useful when building
/// command sequences with `queue!` or `execute!` macros.
///
/// # Examples
///
/// ```
/// use nettoolskit_ui::style::set_fg;
/// use nettoolskit_ui::PRIMARY_COLOR;
/// use crossterm::queue;
/// use std::io::stdout;
///
/// queue!(stdout(), set_fg(PRIMARY_COLOR)).unwrap();
/// ```
#[inline]
pub fn set_fg(rgb: Rgb) -> SetForegroundColor {
    SetForegroundColor(rgb_to_crossterm(rgb))
}
