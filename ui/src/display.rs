use owo_colors::{OwoColorize, Rgb};
use std::env;
use nettoolskit_utils::string::truncate_directory_with_middle;

/// Global UI color constants
pub const PRIMARY_COLOR: Rgb = Rgb(155, 114, 255);
pub const SECONDARY_COLOR: Rgb = Rgb(204, 185, 254);
pub const WHITE_COLOR: Rgb = Rgb(255, 255, 255);
pub const GRAY_COLOR: Rgb = Rgb(128, 128, 128);

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
pub fn print_welcome_box() {
    let current_dir = get_current_directory();
    let box_width = 89; // Total characters in the box line

    println!("{}", "╭─────────────────────────────────────────────────────────────────────────────────────────╮".color(PRIMARY_COLOR));
    println!(
        "{} {} {}                                                              {}",
        "│".color(PRIMARY_COLOR),
        ">_".color(PRIMARY_COLOR).bold(),
        format!("NetToolsKit CLI ({})", VERSION).color(WHITE_COLOR),
        "│".color(PRIMARY_COLOR)
    );
    println!(
        "{}    {}                                      {}",
        "│".color(PRIMARY_COLOR),
        "A comprehensive toolkit for backend development".color(GRAY_COLOR),
        "│".color(PRIMARY_COLOR)
    );
    println!("{}", "│                                                                                         │".color(PRIMARY_COLOR));

    // Calculate available width for directory path (leaving 4 spaces before final │)
    let dir_label = "    directory: ";
    let available_width = box_width - dir_label.len() - 1 - 4 - 4; // -1 for │, -4 for spaces, -4 for safety margin
    let truncated_dir = truncate_directory_with_middle(&current_dir, available_width);

    // Calculate padding for directory line to align properly
    let dir_text_length = dir_label.len() + truncated_dir.len();
    let padding_needed = box_width - dir_text_length;
    let padding = " ".repeat(padding_needed.max(4));

    println!(
        "{}{}{}{}",
        "│".color(PRIMARY_COLOR),
        "    directory: ".color(GRAY_COLOR),
        truncated_dir.color(WHITE_COLOR),
        format!("{}│", padding).color(PRIMARY_COLOR)
    );
    println!("{}", "╰─────────────────────────────────────────────────────────────────────────────────────────╯".color(PRIMARY_COLOR));
    println!();
    println!();
}

/// Print the NetToolsKit ASCII art logo.
///
/// Displays a stylized ASCII representation of the NetToolsKit branding
/// using the primary color scheme.
pub fn print_ascii_logo() {
    let logo_color = PRIMARY_COLOR;

    println!(" {}", "███╗   ██╗███████╗████████╗████████╗ ██████╗  ██████╗ ██╗     ███████╗██╗  ██╗██╗████████╗".color(logo_color));
    println!(" {}", "████╗  ██║██╔════╝╚══██╔══╝╚══██╔══╝██╔═══██╗██╔═══██╗██║     ██╔════╝██║ ██╔╝██║╚══██╔══╝".color(logo_color));
    println!(" {}", "██╔██╗ ██║█████╗     ██║      ██║   ██║   ██║██║   ██║██║     ███████╗█████╔╝ ██║   ██║   ".color(logo_color));
    println!(" {}", "██║╚██╗██║██╔══╝     ██║      ██║   ██║   ██║██║   ██║██║     ╚════██║██╔═██╗ ██║   ██║   ".color(logo_color));
    println!(" {}", "██║ ╚████║███████╗   ██║      ██║   ╚██████╔╝╚██████╔╝███████╗███████║██║  ██╗██║   ██║   ".color(logo_color));
    println!(" {}", "╚═╝  ╚═══╝╚══════╝   ╚═╝      ╚═╝    ╚═════╝  ╚═════╝ ╚══════╝╚══════╝╚═╝  ╚═╝╚═╝   ╚═╝   ".color(logo_color));

    println!();
    println!();
    println!("{}", "💡 Tip: Type / to see available commands".color(GRAY_COLOR));
    println!("{}", "   Use ↑↓ to navigate, Enter to select, /quit to exit".color(GRAY_COLOR));
    println!();
    println!();
}

/// Get the current directory with home directory substitution.
///
/// Returns the current working directory, replacing the user's home directory
/// with "~" for cleaner display. Falls back to "~" if directory cannot be determined.
pub fn get_current_directory() -> String {
    let current = env::current_dir()
        .map(|path| path.display().to_string())
        .unwrap_or_else(|_| "~".to_string());

    if let Ok(home) = env::var("USERPROFILE").or_else(|_| env::var("HOME")) {
        if current.starts_with(&home) {
            let relative = &current[home.len()..];
            return if relative.is_empty() {
                "~".to_string()
            } else {
                format!("~{}", relative)
            };
        }
    }

    current
}