//! NetToolsKit CLI-specific display functions
//!
//! This module contains display functions specific to the NetToolsKit CLI application,
//! including branding, logo, and startup messages.

use nettoolskit_core::path_utils::directory::get_current_directory;
use nettoolskit_ui::{BoxConfig, render_box, Color};
use owo_colors::OwoColorize;

const VERSION: &str = env!("CARGO_PKG_VERSION");

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

    let config = BoxConfig::new(format!("NetToolsKit CLI ({})", VERSION))
        .with_title_prefix(">_")
        .with_title_color(Color::WHITE)
        .with_subtitle("A comprehensive toolkit for backend development")
        .add_footer_item("directory", current_dir, Color::WHITE)
        .with_border_color(Color::PURPLE)
        .with_width(89)
        .with_spacing(true);

    render_box(config);
    println!();
}

/// Print the NetToolsKit ASCII art logo.
///
/// Displays a stylized ASCII representation of the NetToolsKit branding
/// using the primary color scheme.
pub fn print_ascii_logo() {
    let logo_color = Color::PURPLE;

    println!(" {}", "███╗   ██╗███████╗████████╗████████╗ ██████╗  ██████╗ ██╗     ███████╗██╗  ██╗██╗████████╗".color(logo_color));
    println!(" {}", "████╗  ██║██╔════╝╚══██╔══╝╚══██╔══╝██╔═══██╗██╔═══██╗██║     ██╔════╝██║ ██╔╝██║╚══██╔══╝".color(logo_color));
    println!(" {}", "██╔██╗ ██║█████╗     ██║      ██║   ██║   ██║██║   ██║██║     ███████╗█████╔╝ ██║   ██║   ".color(logo_color));
    println!(" {}", "██║╚██╗██║██╔══╝     ██║      ██║   ██║   ██║██║   ██║██║     ╚════██║██╔═██╗ ██║   ██║   ".color(logo_color));
    println!(" {}", "██║ ╚████║███████╗   ██║      ██║   ╚██████╔╝╚██████╔╝███████╗███████║██║  ██╗██║   ██║   ".color(logo_color));
    println!(" {}", "╚═╝  ╚═══╝╚══════╝   ╚═╝      ╚═╝    ╚═════╝  ╚═════╝ ╚══════╝╚══════╝╚═╝  ╚═╝╚═╝   ╚═╝   ".color(logo_color));

    println!();
    println!();
    println!(
        "{}",
        "💡 Tip: Type /help to see all commands, or / to open command palette".color(Color::GRAY)
    );
    println!(
        "{}",
        "   Use ↑↓ to navigate, Enter to select, /quit to exit".color(Color::GRAY)
    );
    println!();
    println!();
}