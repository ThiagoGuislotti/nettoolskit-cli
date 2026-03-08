//! Interactive history viewer with pagination and filtering.
//!
//! Provides a reusable widget for rendering and browsing command/text history.

use crate::core::capabilities::pick_str;
use crate::core::colors::Color;
use crate::rendering::components::{render_box, BoxConfig};
use crossterm::terminal;
use inquire::ui::{Color as InquireColor, RenderConfig, Styled};
use inquire::Select;
use owo_colors::OwoColorize;

const DEFAULT_PAGE_SIZE: usize = 12;
const DEFAULT_MAX_ENTRY_WIDTH: usize = 120;

/// Interactive history viewer with filtering, pagination, and entry rendering.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistoryViewer {
    entries: Vec<String>,
    title: String,
    subtitle: Option<String>,
    prompt: String,
    help_message: String,
    page_size: usize,
    query: Option<String>,
    offset: usize,
}

impl HistoryViewer {
    /// Creates a new history viewer.
    #[must_use]
    pub fn new(entries: Vec<String>) -> Self {
        Self {
            entries,
            title: "History Viewer".to_string(),
            subtitle: None,
            prompt: "Select history entry:".to_string(),
            help_message:
                "Type to filter history, Enter to select, Esc to cancel. Use Up/Down to navigate."
                    .to_string(),
            page_size: DEFAULT_PAGE_SIZE,
            query: None,
            offset: 0,
        }
    }

    /// Sets widget title.
    #[must_use]
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Sets optional subtitle.
    #[must_use]
    pub fn with_subtitle(mut self, subtitle: impl Into<String>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }

    /// Sets selection prompt.
    #[must_use]
    pub fn with_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.prompt = prompt.into();
        self
    }

    /// Sets help message.
    #[must_use]
    pub fn with_help_message(mut self, help_message: impl Into<String>) -> Self {
        self.help_message = help_message.into();
        self
    }

    /// Sets page size (minimum 1).
    #[must_use]
    pub fn with_page_size(mut self, page_size: usize) -> Self {
        self.page_size = page_size.max(1);
        self
    }

    /// Sets case-insensitive filter query.
    pub fn set_query(&mut self, query: Option<String>) {
        self.query = query.map(|value| value.trim().to_string());
        self.offset = 0;
    }

    /// Scrolls one page down.
    pub fn scroll_next_page(&mut self) {
        let total = self.filtered_entries().len();
        let max_start = total.saturating_sub(self.page_size);
        self.offset = (self.offset + self.page_size).min(max_start);
    }

    /// Scrolls one page up.
    pub fn scroll_previous_page(&mut self) {
        self.offset = self.offset.saturating_sub(self.page_size);
    }

    /// Scrolls to the last page.
    pub fn scroll_to_last_page(&mut self) {
        let total = self.filtered_entries().len();
        self.offset = total.saturating_sub(self.page_size);
    }

    /// Returns filtered entries (case-insensitive `contains`).
    #[must_use]
    pub fn filtered_entries(&self) -> Vec<String> {
        match self.query.as_ref().map(|value| value.to_ascii_lowercase()) {
            Some(query) if !query.is_empty() => self
                .entries
                .iter()
                .filter(|entry| entry.to_ascii_lowercase().contains(&query))
                .cloned()
                .collect(),
            _ => self.entries.clone(),
        }
    }

    /// Returns rendered entries for the current page window.
    #[must_use]
    pub fn rendered_page_entries(&self) -> Vec<String> {
        let filtered = self.filtered_entries();
        let start = self.offset.min(filtered.len().saturating_sub(1));
        let end = (start + self.page_size).min(filtered.len());
        let max_width = terminal::size()
            .map(|(width, _)| width as usize)
            .unwrap_or(DEFAULT_MAX_ENTRY_WIDTH)
            .saturating_sub(10);

        filtered[start..end]
            .iter()
            .enumerate()
            .map(|(idx, entry)| {
                let entry_number = start + idx + 1;
                format!("{entry_number:04} | {}", truncate_entry(entry, max_width))
            })
            .collect()
    }

    /// Opens the interactive history viewer.
    ///
    /// Returns the selected rendered entry, or `None` when cancelled/empty.
    pub fn show(&self) -> Option<String> {
        let rendered_entries = self.rendered_page_entries();
        if rendered_entries.is_empty() {
            println!("{}", "No history entries available.".color(Color::YELLOW));
            return None;
        }

        self.render_header(rendered_entries.len(), self.filtered_entries().len());

        let render_config = RenderConfig {
            prompt_prefix: Styled::new(pick_str("?", "?")).with_fg(InquireColor::Rgb {
                r: Color::CYAN.0,
                g: Color::CYAN.1,
                b: Color::CYAN.2,
            }),
            highlighted_option_prefix: Styled::new(pick_str("❯", ">")).with_fg(InquireColor::Rgb {
                r: Color::CYAN.0,
                g: Color::CYAN.1,
                b: Color::CYAN.2,
            }),
            ..RenderConfig::default()
        };

        Select::new(&self.prompt, rendered_entries)
            .with_help_message(&self.help_message)
            .with_page_size(self.page_size)
            .with_vim_mode(true)
            .with_render_config(render_config)
            .prompt_skippable()
            .ok()
            .flatten()
    }

    fn render_header(&self, visible_count: usize, filtered_total: usize) {
        let mut box_config = BoxConfig::new(&self.title)
            .with_title_color(Color::WHITE)
            .with_border_color(Color::CYAN)
            .add_footer_item(
                "entries",
                format!("{visible_count}/{filtered_total}"),
                Color::WHITE,
            );

        if let Some(subtitle) = &self.subtitle {
            box_config = box_config.with_subtitle(subtitle);
        }

        if let Some(width) = terminal::size().ok().map(|(w, _)| w as usize) {
            box_config = box_config.with_width(width.saturating_sub(4));
        }

        render_box(box_config);
        println!();
    }
}

fn truncate_entry(input: &str, max_chars: usize) -> String {
    if max_chars == 0 {
        return String::new();
    }

    let char_count = input.chars().count();
    if char_count <= max_chars {
        return input.to_string();
    }

    if max_chars == 1 {
        return "…".to_string();
    }

    let keep = max_chars.saturating_sub(1);
    let prefix: String = input.chars().take(keep).collect();
    format!("{prefix}…")
}

#[cfg(test)]
mod tests {
    use super::HistoryViewer;

    fn sample_entries() -> Vec<String> {
        vec![
            "/help".to_string(),
            "/manifest list".to_string(),
            "/manifest apply".to_string(),
            "generate api module".to_string(),
            "/ai explain task flow".to_string(),
        ]
    }

    #[test]
    fn filtered_entries_applies_query_case_insensitive() {
        let mut viewer = HistoryViewer::new(sample_entries());
        viewer.set_query(Some("MANIFEST".to_string()));
        let filtered = viewer.filtered_entries();
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn scrolling_moves_offset_between_pages() {
        let mut viewer = HistoryViewer::new((0..20).map(|idx| format!("entry-{idx}")).collect())
            .with_page_size(5);

        viewer.scroll_next_page();
        let first_page_after_scroll = viewer.rendered_page_entries();
        assert!(first_page_after_scroll[0].starts_with("0006"));

        viewer.scroll_previous_page();
        let first_page_after_back = viewer.rendered_page_entries();
        assert!(first_page_after_back[0].starts_with("0001"));
    }

    #[test]
    fn rendered_page_entries_include_index_and_separator() {
        let viewer = HistoryViewer::new(sample_entries()).with_page_size(3);
        let rendered = viewer.rendered_page_entries();
        assert_eq!(rendered.len(), 3);
        assert!(rendered[0].contains("0001 | "));
    }

    #[test]
    fn scroll_to_last_page_positions_last_window() {
        let mut viewer = HistoryViewer::new((0..12).map(|idx| format!("entry-{idx}")).collect())
            .with_page_size(5);
        viewer.scroll_to_last_page();
        let rendered = viewer.rendered_page_entries();
        assert!(rendered[0].starts_with("0008"));
    }
}
