//! Box component for rendering bordered information boxes
//!
//! Provides a flexible, configurable box component that can display:
//! - Title with customizable color
//! - Optional subtitle
//! - Multiple footer items (label-value pairs)
//! - Custom border color and width

use nettoolskit_core::string_utils::string::truncate_directory_with_middle;
use owo_colors::{OwoColorize, Rgb};

use crate::core::capabilities::{pick_char, pick_str};
use crossterm::terminal;

const DEFAULT_BOX_WIDTH: usize = 89;
const MIN_SAFE_BOX_WIDTH: usize = 10;
const TERMINAL_HORIZONTAL_MARGIN: usize = 2;
const MIN_RENDER_WIDTH: usize = 2;

/// Configuration for rendering a bordered box
#[derive(Debug, Clone)]
pub struct BoxConfig {
    /// Main title text
    pub title: String,

    /// Color for the title text
    pub title_color: Rgb,

    /// Optional subtitle below the title
    pub subtitle: Option<String>,

    /// Optional prefix before title (e.g., ">_")
    pub title_prefix: Option<String>,

    /// Footer items as (label, value, value_color) tuples
    pub footer_items: Vec<(String, String, Rgb)>,

    /// Color for the box border
    pub border_color: Rgb,

    /// Width of the box (must be >= 10)
    pub width: usize,

    /// Whether to add blank lines before/after
    pub add_spacing: bool,
}

impl Default for BoxConfig {
    fn default() -> Self {
        Self {
            title: String::new(),
            title_color: Rgb(255, 255, 255),
            subtitle: None,
            title_prefix: None,
            footer_items: Vec::new(),
            border_color: Rgb(155, 114, 255),
            width: responsive_box_width_from_terminal(current_terminal_width()),
            add_spacing: true,
        }
    }
}

impl BoxConfig {
    /// Create a new BoxConfig with required fields
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            ..Default::default()
        }
    }

    /// Set the title color
    pub fn with_title_color(mut self, color: Rgb) -> Self {
        self.title_color = color;
        self
    }

    /// Set the subtitle
    pub fn with_subtitle(mut self, subtitle: impl Into<String>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }

    /// Set the title prefix
    pub fn with_title_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.title_prefix = Some(prefix.into());
        self
    }

    /// Add a footer item
    pub fn add_footer_item(
        mut self,
        label: impl Into<String>,
        value: impl Into<String>,
        color: Rgb,
    ) -> Self {
        self.footer_items.push((label.into(), value.into(), color));
        self
    }

    /// Set the border color
    pub fn with_border_color(mut self, color: Rgb) -> Self {
        self.border_color = color;
        self
    }

    /// Set the width
    pub fn with_width(mut self, width: usize) -> Self {
        self.width = width.max(MIN_SAFE_BOX_WIDTH);
        self
    }

    /// Set spacing behavior
    pub fn with_spacing(mut self, add_spacing: bool) -> Self {
        self.add_spacing = add_spacing;
        self
    }
}

/// Render a bordered box with the given configuration
pub fn render_box(config: BoxConfig) {
    if config.add_spacing {
        println!();
    }

    let border_color = config.border_color;
    let width = clamp_render_width_to_terminal(config.width, current_terminal_width());

    // Resolve box-drawing characters with ASCII fallback
    let corner_tl = pick_char('╭', '+');
    let corner_tr = pick_char('╮', '+');
    let corner_bl = pick_char('╰', '+');
    let corner_br = pick_char('╯', '+');
    let vert = pick_str("│", "|");
    let horiz = pick_str("─", "-");

    // Top border
    let top_border = format!("{}{}{}", corner_tl, horiz.repeat(width - 2), corner_tr);
    println!("{}", top_border.color(border_color));

    // Title line
    if let Some(prefix) = &config.title_prefix {
        // With prefix: "│ >_ Title │"
        let content = format!(" {} {}", prefix, config.title);
        let content_len = content.len() + 2; // +2 for borders │ │
        let padding = " ".repeat(width.saturating_sub(content_len));

        let line = format!(
            "{} {prefix} {}{}{}",
            vert.color(border_color),
            config.title.color(config.title_color).bold(),
            padding,
            vert.color(border_color)
        );
        println!("{}", line.trim_end());
    } else {
        // Without prefix: "│ Title │"
        let content_len = config.title.len() + 3; // +1 for space after │, +2 for borders
        let padding = " ".repeat(width.saturating_sub(content_len));

        let line = format!(
            "{} {}{}{}",
            vert.color(border_color),
            config.title.color(config.title_color).bold(),
            padding,
            vert.color(border_color)
        );
        println!("{}", line.trim_end());
    }

    // Subtitle line
    if let Some(subtitle) = &config.subtitle {
        // Subtitle: "│    subtitle │"
        let content_len = subtitle.len() + 6; // +4 for indentation "    ", +2 for borders
        let padding = " ".repeat(width.saturating_sub(content_len));
        let line = format!(
            "{}{}{}{}",
            vert.color(border_color),
            format!("    {}", subtitle).color(border_color),
            padding,
            vert.color(border_color)
        );
        println!("{}", line.trim_end());
    }

    // Blank line before footer if we have footer items
    if !config.footer_items.is_empty() {
        let blank_line = format!("{}{}{}", vert, " ".repeat(width - 2), vert);
        println!("{}", blank_line.color(border_color).to_string().trim_end());
    }

    // Footer items
    for (label, value, value_color) in &config.footer_items {
        let label_text = format!("    {}: ", label);

        // Truncate value if it's a directory path
        let available_width = width.saturating_sub(label_text.len() + 9);
        let truncated_value = if label.to_lowercase().contains("directory") {
            truncate_directory_with_middle(value, available_width)
        } else if value.len() > available_width {
            format!("{}...", &value[..available_width.saturating_sub(3)])
        } else {
            value.clone()
        };

        let line_len = label_text.len() + truncated_value.len() + 2; // +2 for borders │ │
        let padding_needed = width.saturating_sub(line_len);
        let padding = " ".repeat(padding_needed);

        let line = format!(
            "{}{}{}{}{}",
            vert.color(border_color),
            label_text.color(Rgb(128, 128, 128)), // Gray for labels
            truncated_value.color(*value_color),
            padding,
            vert.color(border_color)
        );
        println!("{}", line.trim_end());
    }

    // Bottom border
    let bottom_border = format!("{}{}{}", corner_bl, horiz.repeat(width - 2), corner_br);
    println!("{}", bottom_border.color(border_color));

    if config.add_spacing {
        println!();
    }
}

fn current_terminal_width() -> Option<usize> {
    terminal::size().ok().map(|(width, _)| width as usize)
}

pub(crate) fn responsive_box_width_from_terminal(width: Option<usize>) -> usize {
    let terminal_based = width
        .map(|value| value.saturating_sub(TERMINAL_HORIZONTAL_MARGIN))
        .unwrap_or(DEFAULT_BOX_WIDTH);
    terminal_based.clamp(MIN_SAFE_BOX_WIDTH, DEFAULT_BOX_WIDTH)
}

fn clamp_render_width_to_terminal(config_width: usize, terminal_width: Option<usize>) -> usize {
    let requested = config_width.max(MIN_SAFE_BOX_WIDTH);
    let Some(terminal_width) = terminal_width else {
        return requested;
    };

    let available = terminal_width.saturating_sub(TERMINAL_HORIZONTAL_MARGIN);
    available.clamp(MIN_RENDER_WIDTH, requested)
}

#[cfg(test)]
mod tests {
    use super::{
        clamp_render_width_to_terminal, responsive_box_width_from_terminal, DEFAULT_BOX_WIDTH,
        MIN_RENDER_WIDTH, MIN_SAFE_BOX_WIDTH,
    };

    #[test]
    fn responsive_box_width_uses_default_when_terminal_is_unknown() {
        assert_eq!(responsive_box_width_from_terminal(None), DEFAULT_BOX_WIDTH);
    }

    #[test]
    fn responsive_box_width_caps_on_wide_terminal() {
        assert_eq!(
            responsive_box_width_from_terminal(Some(300)),
            DEFAULT_BOX_WIDTH
        );
    }

    #[test]
    fn responsive_box_width_shrinks_for_narrow_terminal() {
        assert_eq!(responsive_box_width_from_terminal(Some(40)), 38);
    }

    #[test]
    fn responsive_box_width_respects_minimum_guard() {
        assert_eq!(
            responsive_box_width_from_terminal(Some(4)),
            MIN_SAFE_BOX_WIDTH
        );
    }

    #[test]
    fn clamp_render_width_keeps_requested_when_terminal_unknown() {
        assert_eq!(clamp_render_width_to_terminal(64, None), 64);
    }

    #[test]
    fn clamp_render_width_caps_to_terminal_capacity() {
        assert_eq!(clamp_render_width_to_terminal(89, Some(70)), 68);
    }

    #[test]
    fn clamp_render_width_honors_requested_when_terminal_is_wide() {
        assert_eq!(clamp_render_width_to_terminal(40, Some(120)), 40);
    }

    #[test]
    fn clamp_render_width_never_returns_less_than_render_minimum() {
        assert_eq!(
            clamp_render_width_to_terminal(10, Some(1)),
            MIN_RENDER_WIDTH
        );
    }
}
