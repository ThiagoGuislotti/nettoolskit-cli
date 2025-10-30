use owo_colors::{OwoColorize, Rgb};
use std::env;

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
    let max_width = 85;
    let truncated_dir = truncate_directory(&current_dir, max_width - 20);

    println!("╭─────────────────────────────────────────────────────────────────────────────────────────╮");
    println!(
        "│ {} {}                                                              │",
        ">_".color(PRIMARY_COLOR).bold(),
        format!("NetToolsKit CLI ({})", VERSION).color(WHITE_COLOR).bold()
    );
    println!(
        "│    {}                                      │",
        "A comprehensive toolkit for backend development".color(GRAY_COLOR)
    );
    println!("│                                                                                         │");
    println!(
        "│    directory: {}        │",
        truncated_dir.color(SECONDARY_COLOR)
    );
    println!("╰─────────────────────────────────────────────────────────────────────────────────────────╯");
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

/// Truncate directory path intelligently to fit within max_width.
///
/// Preserves the beginning and end of the path while truncating the middle
/// to ensure important context is maintained in limited display space.
///
/// # Arguments
///
/// * `dir` - The directory path to truncate
/// * `max_width` - Maximum width allowed for the path
///
/// # Returns
///
/// Returns a truncated path string that fits within the specified width
/// while preserving contextual information.
///
/// # Examples
///
/// ```
/// use nettoolskit_ui::truncate_directory;
/// let truncated = truncate_directory("/very/long/path/to/project", 20);
/// // Returns something like "/very/.../project"
/// ```
pub fn truncate_directory(dir: &str, max_width: usize) -> String {
    if dir.len() <= max_width {
        return dir.to_string();
    }

    // Detect directory separator (Windows or Unix)
    let separator = if dir.contains('\\') { '\\' } else { '/' };
    let separator_str = separator.to_string();

    // Split path into components
    let parts: Vec<&str> = dir.split(separator).collect();

    if parts.len() <= 2 {
        // Can't truncate much, just take the end
        let ellipsis = "...";
        let available = max_width.saturating_sub(ellipsis.len());
        let start_pos = dir.len().saturating_sub(available);
        return format!("{}{}", ellipsis, &dir[start_pos..]);
    }

    // Try to preserve first and last parts with ellipsis in middle
    let first_part = parts[0];
    let last_part = parts[parts.len() - 1];
    let ellipsis = "...";

    let base_length = first_part.len() + last_part.len() + ellipsis.len() + 2; // +2 for separators

    if base_length <= max_width {
        return format!("{}{}{}{}{}", first_part, separator_str, ellipsis, separator_str, last_part);
    }

    // If still too long, truncate the last part
    let available_for_last = max_width.saturating_sub(first_part.len() + ellipsis.len() + 2);
    let truncated_last = if last_part.len() > available_for_last {
        &last_part[last_part.len().saturating_sub(available_for_last)..]
    } else {
        last_part
    };

    format!("{}{}{}{}{}", first_part, separator_str, ellipsis, separator_str, truncated_last)
}