//! NetToolsKit CLI-specific display functions
//!
//! This module contains display functions specific to the NetToolsKit CLI application,
//! including branding, logo, and startup messages.

use crossterm::terminal;
use nettoolskit_core::path_utils::directory::get_current_directory;
use nettoolskit_ui::{capabilities, render_box, BoxConfig, Color};
use owo_colors::OwoColorize;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const DEFAULT_BOX_WIDTH: usize = 89;
const MIN_SAFE_BOX_WIDTH: usize = 10;
const FULL_LOGO_MIN_WIDTH: usize = 110;
const COMPACT_LOGO_MIN_WIDTH: usize = 72;

/// Print the complete CLI startup display including welcome box and ASCII logo.
pub fn print_logo() {
    print_welcome_box();
    print_ascii_logo();
}

/// Print an informative welcome box with CLI information.
///
/// Displays the NetToolsKit CLI version, description, current directory,
/// and helpful usage tips in a formatted box layout.
///
/// This is a convenience wrapper around `render_box` with CLI-specific configuration.
pub fn print_welcome_box() {
    let current_dir = get_current_directory();
    let box_width = responsive_box_width();
    let subtitle = "A comprehensive toolkit for backend development";

    let mut config = BoxConfig::new(format!("NetToolsKit CLI ({})", VERSION))
        .with_title_prefix(">_")
        .with_title_color(Color::WHITE)
        .add_footer_item("directory", current_dir, Color::WHITE)
        .with_border_color(Color::PURPLE)
        .with_width(box_width)
        .with_spacing(true);

    // Keep subtitle only when there's enough horizontal room to avoid wrapping.
    if box_width > subtitle.len() + 12 {
        config = config.with_subtitle(subtitle);
    }

    render_box(config);
    println!();
}

/// Print the NetToolsKit ASCII art logo.
///
/// Displays a stylized ASCII representation of the NetToolsKit branding
/// using the primary color scheme.
pub fn print_ascii_logo() {
    let width = terminal_width().unwrap_or(120);
    let logo_color = Color::PURPLE;
    let has_unicode = capabilities().unicode;

    if has_unicode && width >= FULL_LOGO_MIN_WIDTH {
        println!(" {}", "███╗   ██╗███████╗████████╗████████╗ ██████╗  ██████╗ ██╗     ███████╗██╗  ██╗██╗████████╗".color(logo_color));
        println!(" {}", "████╗  ██║██╔════╝╚══██╔══╝╚══██╔══╝██╔═══██╗██╔═══██╗██║     ██╔════╝██║ ██╔╝██║╚══██╔══╝".color(logo_color));
        println!(" {}", "██╔██╗ ██║█████╗     ██║      ██║   ██║   ██║██║   ██║██║     ███████╗█████╔╝ ██║   ██║   ".color(logo_color));
        println!(" {}", "██║╚██╗██║██╔══╝     ██║      ██║   ██║   ██║██║   ██║██║     ╚════██║██╔═██╗ ██║   ██║   ".color(logo_color));
        println!(" {}", "██║ ╚████║███████╗   ██║      ██║   ╚██████╔╝╚██████╔╝███████╗███████║██║  ██╗██║   ██║   ".color(logo_color));
        println!(" {}", "╚═╝  ╚═══╝╚══════╝   ╚═╝      ╚═╝    ╚═════╝  ╚═════╝ ╚══════╝╚══════╝╚═╝  ╚═╝╚═╝   ╚═╝   ".color(logo_color));
    } else if width >= COMPACT_LOGO_MIN_WIDTH {
        println!("{}", "NETTOOLSKIT CLI".color(logo_color).bold());
        println!("{}", "Backend automation toolkit".color(Color::GRAY));
    } else {
        println!("{}", "NTK".color(logo_color).bold());
    }

    println!();
    if width >= 90 {
        println!(
            "{}",
            "Tip: type /help to see all commands, or / to open command palette".color(Color::GRAY)
        );
        println!(
            "{}",
            "Use Up/Down to navigate, Enter to select, /quit to exit".color(Color::GRAY)
        );
    } else if width >= 60 {
        println!("{}", "Tip: /help, /, /quit".color(Color::GRAY));
    }
    println!();
}

fn responsive_box_width() -> usize {
    responsive_box_width_from_terminal(terminal_width())
}

fn terminal_width() -> Option<usize> {
    terminal::size().ok().map(|(width, _)| width as usize)
}

fn responsive_box_width_from_terminal(width: Option<usize>) -> usize {
    let terminal_based = width
        .map(|value| value.saturating_sub(2))
        .unwrap_or(DEFAULT_BOX_WIDTH);
    terminal_based.clamp(MIN_SAFE_BOX_WIDTH, DEFAULT_BOX_WIDTH)
}

#[cfg(test)]
mod tests {
    use super::{responsive_box_width_from_terminal, DEFAULT_BOX_WIDTH, MIN_SAFE_BOX_WIDTH};

    #[test]
    fn responsive_box_width_uses_workspace_default_when_terminal_is_unknown() {
        assert_eq!(responsive_box_width_from_terminal(None), DEFAULT_BOX_WIDTH);
    }

    #[test]
    fn responsive_box_width_caps_to_default_on_wide_terminals() {
        assert_eq!(
            responsive_box_width_from_terminal(Some(300)),
            DEFAULT_BOX_WIDTH
        );
    }

    #[test]
    fn responsive_box_width_shrinks_for_narrow_terminals() {
        assert_eq!(responsive_box_width_from_terminal(Some(40)), 38);
    }

    #[test]
    fn responsive_box_width_respects_minimum_guard() {
        assert_eq!(
            responsive_box_width_from_terminal(Some(4)),
            MIN_SAFE_BOX_WIDTH
        );
    }
}
