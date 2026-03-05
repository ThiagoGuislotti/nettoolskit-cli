use crate::core::capabilities::pick_str;
use crate::core::colors::Color;
use crate::rendering::components::{render_box, BoxConfig};
use crossterm::terminal;
use inquire::ui::{Color as InquireColor, RenderConfig, Styled};
use inquire::Select;
use nettoolskit_core::file_search::{search_files, SearchConfig};
use owo_colors::{OwoColorize, Rgb};
use regex::Regex;
use std::fmt::{self, Display, Formatter};
use std::io;
use std::path::{Path, PathBuf};

const DEFAULT_PAGE_SIZE: usize = 10;
const DEFAULT_MAX_DEPTH: usize = 8;
const DEFAULT_HELP_MESSAGE: &str =
    "Type to fuzzy-filter. Use re:<regex> for regex, lit:<text> for literal. Enter selects.";

#[derive(Debug, Clone, PartialEq, Eq)]
struct FilePickerEntry {
    display: String,
    path: PathBuf,
}

impl Display for FilePickerEntry {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display)
    }
}

/// Interactive file picker with fuzzy and regex filtering.
///
/// Filtering modes while typing:
/// - default: fuzzy subsequence matching
/// - `re:<pattern>`: regex matching
/// - `lit:<text>`: case-insensitive literal matching
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilePicker {
    root: PathBuf,
    include_patterns: Vec<String>,
    exclude_patterns: Vec<String>,
    max_depth: Option<usize>,
    include_hidden: bool,
    page_size: usize,
    title: Option<String>,
    subtitle: Option<String>,
    prompt: String,
    help_message: String,
    cursor_color: Rgb,
}

impl FilePicker {
    /// Creates a picker with generic defaults (`*` include pattern).
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self {
            root: root.into(),
            include_patterns: vec!["*".to_string()],
            exclude_patterns: vec![
                "**/.git/**".to_string(),
                "**/target/**".to_string(),
                "**/node_modules/**".to_string(),
                "**/.idea/**".to_string(),
                "**/.vscode/**".to_string(),
            ],
            max_depth: Some(DEFAULT_MAX_DEPTH),
            include_hidden: false,
            page_size: DEFAULT_PAGE_SIZE,
            title: None,
            subtitle: None,
            prompt: "Select file:".to_string(),
            help_message: DEFAULT_HELP_MESSAGE.to_string(),
            cursor_color: Color::PURPLE,
        }
    }

    /// Creates a manifest-focused picker with YAML manifest include patterns.
    pub fn for_manifests(root: impl Into<PathBuf>) -> Self {
        Self::new(root).with_include_patterns(vec![
            "ntk-*.yml".to_string(),
            "ntk-*.yaml".to_string(),
            "*manifest*.yml".to_string(),
            "*manifest*.yaml".to_string(),
        ])
    }

    /// Sets include patterns for file discovery.
    pub fn with_include_patterns(mut self, include_patterns: Vec<String>) -> Self {
        if !include_patterns.is_empty() {
            self.include_patterns = include_patterns;
        }
        self
    }

    /// Sets exclude patterns for file discovery.
    pub fn with_exclude_patterns(mut self, exclude_patterns: Vec<String>) -> Self {
        self.exclude_patterns = exclude_patterns;
        self
    }

    /// Sets max depth for file discovery.
    pub fn with_max_depth(mut self, max_depth: Option<usize>) -> Self {
        self.max_depth = max_depth;
        self
    }

    /// Enables or disables hidden files in discovery.
    pub fn with_include_hidden(mut self, include_hidden: bool) -> Self {
        self.include_hidden = include_hidden;
        self
    }

    /// Sets prompt text for selection.
    pub fn with_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.prompt = prompt.into();
        self
    }

    /// Sets title rendered above the picker.
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Sets subtitle rendered above the picker.
    pub fn with_subtitle(mut self, subtitle: impl Into<String>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }

    /// Sets help message shown by the interactive selector.
    pub fn with_help_message(mut self, help_message: impl Into<String>) -> Self {
        self.help_message = help_message.into();
        self
    }

    /// Sets page size used by the interactive selector.
    pub fn with_page_size(mut self, page_size: usize) -> Self {
        self.page_size = page_size.max(1);
        self
    }

    /// Discovers files using current picker search settings.
    ///
    /// # Errors
    ///
    /// Returns an error when the search configuration is invalid or directory scanning fails.
    pub fn discover_files(&self) -> io::Result<Vec<PathBuf>> {
        let config = SearchConfig {
            include_patterns: self.include_patterns.clone(),
            exclude_patterns: self.exclude_patterns.clone(),
            max_depth: self.max_depth,
            follow_links: false,
            include_hidden: self.include_hidden,
        };

        let mut files = search_files(&self.root, &config).map_err(io::Error::other)?;
        files.sort();
        files.dedup();
        Ok(files)
    }

    /// Shows the interactive file picker and returns the selected file path.
    ///
    /// Returns `None` when the picker is cancelled or when no files are found.
    pub fn show(&self) -> Option<PathBuf> {
        let files = match self.discover_files() {
            Ok(paths) => paths,
            Err(err) => {
                println!(
                    "{} {}",
                    "⚠ File discovery failed:".color(Color::YELLOW),
                    format!("{err}").color(Color::GRAY)
                );
                return None;
            }
        };

        if files.is_empty() {
            println!(
                "{}",
                "No files found for picker criteria.".color(Color::YELLOW)
            );
            return None;
        }

        self.render_header();

        let entries = build_entries(&self.root, files);
        let mut render_config = RenderConfig::default();
        render_config.prompt_prefix = Styled::new(pick_str("?", "?")).with_fg(InquireColor::Rgb {
            r: self.cursor_color.0,
            g: self.cursor_color.1,
            b: self.cursor_color.2,
        });
        render_config.highlighted_option_prefix =
            Styled::new(pick_str("❯", ">")).with_fg(InquireColor::Rgb {
                r: self.cursor_color.0,
                g: self.cursor_color.1,
                b: self.cursor_color.2,
            });
        render_config.selected_option = Some(
            render_config
                .selected_option
                .unwrap_or_default()
                .with_fg(InquireColor::Rgb {
                    r: self.cursor_color.0,
                    g: self.cursor_color.1,
                    b: self.cursor_color.2,
                }),
        );

        Select::new(&self.prompt, entries)
            .with_help_message(&self.help_message)
            .with_page_size(self.page_size)
            .with_vim_mode(true)
            .with_scorer(&file_picker_scorer)
            .with_render_config(render_config)
            .prompt_skippable()
            .ok()
            .flatten()
            .map(|entry| entry.path)
    }

    fn render_header(&self) {
        if self.title.is_none() && self.subtitle.is_none() {
            return;
        }

        let title = self.title.as_deref().unwrap_or("File Picker");
        let mut box_config = BoxConfig::new(title)
            .with_title_color(Color::WHITE)
            .with_border_color(Color::PURPLE);

        if let Some(subtitle) = &self.subtitle {
            box_config = box_config.with_subtitle(subtitle);
        }

        if let Some(width) = terminal_width() {
            box_config = box_config.with_width(width.saturating_sub(4));
        }

        render_box(box_config);
        println!();
    }
}

fn terminal_width() -> Option<usize> {
    terminal::size().ok().map(|(w, _)| w as usize)
}

fn build_entries(root: &Path, files: Vec<PathBuf>) -> Vec<FilePickerEntry> {
    files
        .into_iter()
        .map(|path| FilePickerEntry {
            display: relative_display_path(root, &path),
            path,
        })
        .collect()
}

fn relative_display_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .display()
        .to_string()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FilterMode<'a> {
    Fuzzy(&'a str),
    Regex(&'a str),
    Literal(&'a str),
}

fn parse_filter_mode(input: &str) -> FilterMode<'_> {
    let trimmed = input.trim();
    if let Some(pattern) = trimmed.strip_prefix("re:") {
        FilterMode::Regex(pattern.trim())
    } else if let Some(pattern) = trimmed.strip_prefix("lit:") {
        FilterMode::Literal(pattern.trim())
    } else {
        FilterMode::Fuzzy(trimmed)
    }
}

fn file_picker_scorer(
    input: &str,
    _option: &FilePickerEntry,
    string_value: &str,
    idx: usize,
) -> Option<i64> {
    let idx_score = idx.min(i64::MAX as usize) as i64;
    match parse_filter_mode(input) {
        FilterMode::Regex(pattern) => regex_score(pattern, string_value, idx_score),
        FilterMode::Literal(pattern) => literal_score(pattern, string_value, idx_score),
        FilterMode::Fuzzy(pattern) => fuzzy_score(pattern, string_value, idx_score),
    }
}

fn regex_score(pattern: &str, candidate: &str, idx_score: i64) -> Option<i64> {
    if pattern.is_empty() {
        return Some(i64::MAX.saturating_sub(idx_score));
    }

    let regex = Regex::new(pattern).ok()?;
    let first_match = regex.find(candidate)?;
    Some(20_000 - first_match.start() as i64 - idx_score)
}

fn literal_score(pattern: &str, candidate: &str, idx_score: i64) -> Option<i64> {
    if pattern.is_empty() {
        return Some(i64::MAX.saturating_sub(idx_score));
    }

    let normalized_pattern = pattern.to_ascii_lowercase();
    let normalized_candidate = candidate.to_ascii_lowercase();
    let position = normalized_candidate.find(&normalized_pattern)?;
    Some(10_000 - position as i64 - idx_score)
}

fn fuzzy_score(pattern: &str, candidate: &str, idx_score: i64) -> Option<i64> {
    if pattern.is_empty() {
        return Some(i64::MAX.saturating_sub(idx_score));
    }

    let query = pattern.to_ascii_lowercase();
    let haystack = candidate.to_ascii_lowercase();

    if haystack.contains(&query) {
        return Some(9_000 - idx_score);
    }

    let mut score = 0i64;
    let mut q_index = 0usize;
    let query_chars: Vec<char> = query.chars().collect();
    let mut previous_match: Option<usize> = None;

    for (h_index, ch) in haystack.chars().enumerate() {
        if q_index >= query_chars.len() {
            break;
        }

        if ch == query_chars[q_index] {
            score += 10;
            if let Some(prev) = previous_match {
                if h_index == prev + 1 {
                    score += 5;
                }
            }
            previous_match = Some(h_index);
            q_index += 1;
        }
    }

    if q_index == query_chars.len() {
        Some(score.saturating_sub(idx_score))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    struct TempDirGuard {
        path: PathBuf,
    }

    impl TempDirGuard {
        fn new(prefix: &str) -> Self {
            let nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|duration| duration.as_nanos())
                .unwrap_or(0);
            let path = std::env::temp_dir().join(format!(
                "ntk-file-picker-tests-{prefix}-{}-{nanos}",
                std::process::id()
            ));
            fs::create_dir_all(&path).expect("temp directory should be created");
            Self { path }
        }
    }

    impl Drop for TempDirGuard {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    #[test]
    fn parse_filter_mode_detects_prefixes() {
        assert!(matches!(
            parse_filter_mode("re:foo.*"),
            FilterMode::Regex("foo.*")
        ));
        assert!(matches!(
            parse_filter_mode("lit:manifest"),
            FilterMode::Literal("manifest")
        ));
        assert!(matches!(
            parse_filter_mode("manifest"),
            FilterMode::Fuzzy("manifest")
        ));
    }

    #[test]
    fn relative_display_path_prefers_relative() {
        let root = PathBuf::from("repo");
        let file = root.join("src").join("main.rs");
        let expected = PathBuf::from("src").join("main.rs").display().to_string();
        assert_eq!(relative_display_path(&root, &file), expected);
    }

    #[test]
    fn regex_score_matches_and_rejects_invalid_regex() {
        assert!(regex_score("^src/.*\\.rs$", "src/main.rs", 0).is_some());
        assert!(regex_score("[invalid", "src/main.rs", 0).is_none());
    }

    #[test]
    fn literal_score_is_case_insensitive() {
        assert!(literal_score("MANIFEST", "ntk-manifest.yaml", 0).is_some());
        assert!(literal_score("missing", "ntk-manifest.yaml", 0).is_none());
    }

    #[test]
    fn fuzzy_score_matches_subsequence() {
        assert!(fuzzy_score("mnfst", "ntk-manifest.yaml", 0).is_some());
        assert!(fuzzy_score("zzz", "ntk-manifest.yaml", 0).is_none());
    }

    #[test]
    fn picker_builder_enforces_minimum_page_size() {
        let picker = FilePicker::new(".").with_page_size(0);
        assert_eq!(picker.page_size, 1);
    }

    #[test]
    fn with_include_patterns_keeps_default_when_empty() {
        let picker = FilePicker::new(".").with_include_patterns(Vec::new());
        assert_eq!(picker.include_patterns, vec!["*".to_string()]);
    }

    #[test]
    fn discover_files_for_manifest_picker_respects_patterns() {
        let temp = TempDirGuard::new("discover-manifests");
        let manifest_path = temp.path.join("ntk-demo.yml");
        let other_path = temp.path.join("readme.txt");
        fs::write(&manifest_path, "manifest: true\n").expect("manifest file should be written");
        fs::write(&other_path, "plain text\n").expect("non-manifest file should be written");

        let picker = FilePicker::for_manifests(&temp.path);
        let files = picker.discover_files().expect("discovery should succeed");

        assert!(files.contains(&manifest_path));
        assert!(!files.contains(&other_path));
    }

    #[test]
    fn discover_files_can_include_hidden_files_when_enabled() {
        let temp = TempDirGuard::new("discover-hidden");
        let hidden_path = temp.path.join(".secret.yml");
        fs::write(&hidden_path, "x: 1\n").expect("hidden file should be written");

        let picker = FilePicker::new(&temp.path)
            .with_include_patterns(vec!["*.yml".to_string()])
            .with_include_hidden(true);
        let files = picker.discover_files().expect("discovery should succeed");

        assert!(files.contains(&hidden_path));
    }

    #[test]
    fn build_entries_keeps_full_path_when_outside_root() {
        let root = PathBuf::from("repo");
        let outside = PathBuf::from("external").join("file.txt");
        let entries = build_entries(&root, vec![outside.clone()]);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].display, outside.display().to_string());
        assert_eq!(entries[0].path, outside);
    }

    #[test]
    fn regex_score_empty_pattern_prefers_earlier_index() {
        let score_a = regex_score("", "src/main.rs", 0).expect("score should exist");
        let score_b = regex_score("", "src/main.rs", 10).expect("score should exist");
        assert!(score_a > score_b);
    }

    #[test]
    fn literal_score_empty_pattern_prefers_earlier_index() {
        let score_a = literal_score("", "src/main.rs", 1).expect("score should exist");
        let score_b = literal_score("", "src/main.rs", 2).expect("score should exist");
        assert!(score_a > score_b);
    }

    #[test]
    fn file_picker_scorer_supports_filter_modes() {
        let option = FilePickerEntry {
            display: "src/main.rs".to_string(),
            path: PathBuf::from("src/main.rs"),
        };
        assert!(file_picker_scorer("re:.*main.*", &option, "src/main.rs", 0).is_some());
        assert!(file_picker_scorer("lit:MAIN", &option, "src/main.rs", 0).is_some());
        assert!(file_picker_scorer("smn", &option, "src/main.rs", 0).is_some());
    }
}
