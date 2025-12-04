use nettoolskit_ui::core::style::{rgb_to_crossterm, set_fg};
use owo_colors::Rgb;
use crossterm::style::{Color, SetForegroundColor};

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
