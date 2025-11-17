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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rgb_to_crossterm_conversion() {
        let rgb = Rgb(155, 114, 255);
        let color = rgb_to_crossterm(rgb);

        match color {
            Color::Rgb { r, g, b } => {
                assert_eq!(r, 155);
                assert_eq!(g, 114);
                assert_eq!(b, 255);
            }
            _ => panic!("Expected Color::Rgb variant"),
        }
    }

    #[test]
    fn test_set_fg_creates_valid_command() {
        let rgb = Rgb(255, 255, 255);
        let cmd = set_fg(rgb);

        // Verify it's the correct type (compile-time check)
        let _: SetForegroundColor = cmd;
    }

    #[test]
    fn test_common_colors() {
        let test_cases = [
            (Rgb(255, 255, 255), (255, 255, 255)), // White
            (Rgb(0, 0, 0), (0, 0, 0)),             // Black
            (Rgb(155, 114, 255), (155, 114, 255)), // Primary
        ];

        for (input_rgb, expected) in test_cases {
            let color = rgb_to_crossterm(input_rgb);
            match color {
                Color::Rgb { r, g, b } => {
                    assert_eq!(
                        (r, g, b),
                        expected,
                        "Color conversion mismatch for {:?}",
                        input_rgb
                    );
                }
                _ => panic!("Expected Color::Rgb variant"),
            }
        }
    }
}
