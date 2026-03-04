use crossterm::event::{
    self, Event, KeyCode, KeyEvent as CrosstermKeyEvent, KeyEventKind, KeyModifiers,
};
use nettoolskit_core::async_utils::with_timeout;
use nettoolskit_core::AppConfig;
use nettoolskit_ui::{
    append_footer_log, copy_to_clipboard, handle_resize, paste_from_clipboard, prepare_prompt_line,
    process_pending_resize, request_terminal_frame, scheduled_frame_poll_timeout,
    set_terminal_focused,
};
use owo_colors::OwoColorize;
use rustyline::completion::{Completer, Pair};
use rustyline::error::ReadlineError;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::history::DefaultHistory;
use rustyline::validate::{ValidationContext, ValidationResult, Validator};
use rustyline::{
    Cmd, CompletionType, ConditionalEventHandler, Config as RustylineConfig, Context, Editor,
    Event as RustylineEvent, EventContext, EventHandler, Helper, KeyEvent as RustylineKeyEvent,
};
use std::borrow::Cow;
use std::cell::RefCell;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use supports_color::Stream;
use tree_sitter::{Node, Parser};

/// Result of an interactive input read operation.
#[derive(Debug)]
pub enum InputResult {
    /// User submitted a command prefixed with `/`.
    Command(String),
    /// User submitted free-form text (not a command).
    Text(String),
    /// User requested to exit (Ctrl-C / Ctrl-D).
    Exit,
    /// User pressed `F1` or `?` to open the interactive menu.
    ShowMenu,
}

const COMPLETION_COMMANDS: &[&str] = &[
    "/help",
    "/manifest",
    "/manifest list",
    "/manifest check",
    "/manifest render",
    "/manifest render-async",
    "/manifest apply",
    "/manifest apply-async",
    "/render-async",
    "/apply-async",
    "/new-async",
    "/translate",
    "/ai",
    "/ai ask",
    "/ai plan",
    "/ai explain",
    "/ai resume",
    "/ai apply --dry-run",
    "/ai apply --approve-write",
    "/task",
    "/task submit",
    "/task list",
    "/task watch",
    "/task cancel",
    "/history",
    "/config",
    "/clear",
    "/quit",
];
const PRIMARY_PROMPT: &str = "> ";
const MULTILINE_CONTINUATION_MARKER: char = '\\';
const MAX_HIGHLIGHT_LINE_BYTES: usize = 2048;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SyntaxLanguage {
    Plain,
    Command,
    Rust,
    CSharp,
    JavaScript,
    TypeScript,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SyntaxTheme {
    command: &'static str,
    flag: &'static str,
    keyword: &'static str,
    string: &'static str,
    comment: &'static str,
    reset: &'static str,
}

impl Default for SyntaxTheme {
    fn default() -> Self {
        Self {
            command: "\u{1b}[1;32m",
            flag: "\u{1b}[36m",
            keyword: "\u{1b}[94m",
            string: "\u{1b}[33m",
            comment: "\u{1b}[90m",
            reset: "\u{1b}[0m",
        }
    }
}

const RUST_KEYWORDS: &[&str] = &[
    "as", "break", "const", "continue", "crate", "else", "enum", "extern", "fn", "for", "if",
    "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref", "return", "self",
    "Self", "static", "struct", "trait", "type", "unsafe", "use", "where", "while",
];

const CSHARP_KEYWORDS: &[&str] = &[
    "abstract",
    "async",
    "await",
    "bool",
    "class",
    "enum",
    "false",
    "interface",
    "namespace",
    "new",
    "null",
    "private",
    "protected",
    "public",
    "record",
    "return",
    "sealed",
    "static",
    "string",
    "struct",
    "this",
    "true",
    "using",
    "var",
    "void",
];

const JAVASCRIPT_KEYWORDS: &[&str] = &[
    "async",
    "await",
    "class",
    "const",
    "constructor",
    "default",
    "export",
    "extends",
    "false",
    "for",
    "function",
    "if",
    "import",
    "let",
    "new",
    "null",
    "return",
    "static",
    "this",
    "true",
    "var",
    "while",
];

const TYPESCRIPT_KEYWORDS: &[&str] = &[
    "as",
    "async",
    "await",
    "class",
    "const",
    "enum",
    "export",
    "extends",
    "false",
    "function",
    "implements",
    "import",
    "interface",
    "let",
    "module",
    "namespace",
    "new",
    "null",
    "private",
    "protected",
    "public",
    "readonly",
    "return",
    "static",
    "this",
    "true",
    "type",
    "var",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TokenClass {
    Keyword,
    String,
    Comment,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct HighlightSpan {
    start: usize,
    end: usize,
    class: TokenClass,
}

struct TreeSitterRuntime {
    rust: Option<Parser>,
    csharp: Option<Parser>,
    javascript: Option<Parser>,
    typescript: Option<Parser>,
}

impl TreeSitterRuntime {
    fn new() -> Self {
        Self {
            rust: build_parser(tree_sitter_rust::language()),
            csharp: build_parser(tree_sitter_c_sharp::language()),
            javascript: build_parser(tree_sitter_javascript::language()),
            typescript: build_parser(tree_sitter_typescript::language_typescript()),
        }
    }

    fn parser_for(&mut self, language: SyntaxLanguage) -> Option<&mut Parser> {
        match language {
            SyntaxLanguage::Rust => self.rust.as_mut(),
            SyntaxLanguage::CSharp => self.csharp.as_mut(),
            SyntaxLanguage::JavaScript => self.javascript.as_mut(),
            SyntaxLanguage::TypeScript => self.typescript.as_mut(),
            SyntaxLanguage::Plain | SyntaxLanguage::Command => None,
        }
    }
}

impl Default for TreeSitterRuntime {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Default)]
struct SyntaxHighlightCache {
    line: String,
    language: Option<SyntaxLanguage>,
    theme: Option<SyntaxTheme>,
    highlighted: Option<String>,
}

thread_local! {
    static TREE_SITTER_RUNTIME: RefCell<TreeSitterRuntime> = RefCell::new(TreeSitterRuntime::new());
    static SYNTAX_HIGHLIGHT_CACHE: RefCell<SyntaxHighlightCache> = RefCell::new(SyntaxHighlightCache::default());
}

fn build_parser(language: tree_sitter::Language) -> Option<Parser> {
    let mut parser = Parser::new();
    parser.set_language(&language).ok()?;
    Some(parser)
}

fn trailing_backslash_count(input: &str) -> usize {
    input
        .chars()
        .rev()
        .take_while(|ch| *ch == MULTILINE_CONTINUATION_MARKER)
        .count()
}

fn has_multiline_continuation_marker(line: &str) -> bool {
    let trimmed = line.trim_end();
    !trimmed.is_empty() && trailing_backslash_count(trimmed) % 2 == 1
}

fn strip_multiline_continuation_marker(line: &str) -> String {
    if !has_multiline_continuation_marker(line) {
        return line.to_string();
    }

    let trimmed = line.trim_end();
    let marker_index = trimmed.len().saturating_sub(1);
    trimmed[..marker_index].to_string()
}

fn normalize_multiline_submission(input: &str) -> String {
    let mut normalized = String::new();
    let mut parts = input.split('\n').peekable();

    while let Some(part) = parts.next() {
        normalized.push_str(&strip_multiline_continuation_marker(part));
        if parts.peek().is_some() {
            normalized.push('\n');
        }
    }

    normalized
}

fn normalize_clipboard_text_for_input(input: &str) -> String {
    let unix_newlines = input.replace("\r\n", "\n").replace('\r', "\n");
    unix_newlines
        .split('\n')
        .map(str::trim_end)
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

#[derive(Debug, Clone)]
struct CliCompleter {
    colors_enabled: bool,
    theme: SyntaxTheme,
    show_menu_requested: Arc<AtomicBool>,
    clipboard_status: Arc<Mutex<Option<String>>>,
    predictive_input_enabled: bool,
}

#[derive(Debug, Clone)]
struct SlashCommandPaletteHandler {
    show_menu_requested: Arc<AtomicBool>,
}

#[derive(Debug, Clone)]
struct ClipboardPasteHandler {
    clipboard_status: Arc<Mutex<Option<String>>>,
}

#[derive(Debug, Clone)]
struct ClipboardCopyHandler {
    clipboard_status: Arc<Mutex<Option<String>>>,
}

impl ConditionalEventHandler for SlashCommandPaletteHandler {
    fn handle(
        &self,
        _evt: &RustylineEvent,
        _n: rustyline::RepeatCount,
        _positive: bool,
        ctx: &EventContext,
    ) -> Option<Cmd> {
        if ctx.line().is_empty() && ctx.pos() == 0 {
            self.show_menu_requested.store(true, Ordering::SeqCst);
            return Some(Cmd::Interrupt);
        }

        None
    }
}

impl ConditionalEventHandler for ClipboardPasteHandler {
    fn handle(
        &self,
        _evt: &RustylineEvent,
        _n: rustyline::RepeatCount,
        _positive: bool,
        _ctx: &EventContext,
    ) -> Option<Cmd> {
        match paste_from_clipboard() {
            Ok(text) => {
                let normalized = normalize_clipboard_text_for_input(&text);
                if normalized.is_empty() {
                    set_clipboard_status(
                        &self.clipboard_status,
                        "Clipboard paste skipped: clipboard is empty".to_string(),
                    );
                    Some(Cmd::Noop)
                } else {
                    set_clipboard_status(
                        &self.clipboard_status,
                        format!(
                            "Clipboard paste inserted {} chars",
                            normalized.chars().count()
                        ),
                    );
                    Some(Cmd::Insert(1, normalized))
                }
            }
            Err(err) => {
                set_clipboard_status(
                    &self.clipboard_status,
                    format!("Clipboard paste failed: {err}"),
                );
                Some(Cmd::Noop)
            }
        }
    }
}

impl ConditionalEventHandler for ClipboardCopyHandler {
    fn handle(
        &self,
        _evt: &RustylineEvent,
        _n: rustyline::RepeatCount,
        _positive: bool,
        ctx: &EventContext,
    ) -> Option<Cmd> {
        let line = ctx.line();
        if line.trim().is_empty() {
            set_clipboard_status(
                &self.clipboard_status,
                "Clipboard copy skipped: current input is empty".to_string(),
            );
            return Some(Cmd::Noop);
        }

        match copy_to_clipboard(line) {
            Ok(()) => {
                set_clipboard_status(
                    &self.clipboard_status,
                    format!("Clipboard copy captured {} chars", line.chars().count()),
                );
            }
            Err(err) => {
                set_clipboard_status(
                    &self.clipboard_status,
                    format!("Clipboard copy failed: {err}"),
                );
            }
        }

        Some(Cmd::Noop)
    }
}

fn set_clipboard_status(status: &Arc<Mutex<Option<String>>>, message: String) {
    let mut slot = status.lock().unwrap_or_else(|error| error.into_inner());
    *slot = Some(message);
}

impl CliCompleter {
    fn new() -> Self {
        Self::with_predictive_input(true)
    }

    fn with_predictive_input(predictive_input_enabled: bool) -> Self {
        Self {
            colors_enabled: supports_color::on(Stream::Stdout).is_some(),
            theme: SyntaxTheme::default(),
            show_menu_requested: Arc::new(AtomicBool::new(false)),
            clipboard_status: Arc::new(Mutex::new(None)),
            predictive_input_enabled,
        }
    }

    fn slash_menu_handler(&self) -> SlashCommandPaletteHandler {
        SlashCommandPaletteHandler {
            show_menu_requested: Arc::clone(&self.show_menu_requested),
        }
    }

    fn clipboard_paste_handler(&self) -> ClipboardPasteHandler {
        ClipboardPasteHandler {
            clipboard_status: Arc::clone(&self.clipboard_status),
        }
    }

    fn clipboard_copy_handler(&self) -> ClipboardCopyHandler {
        ClipboardCopyHandler {
            clipboard_status: Arc::clone(&self.clipboard_status),
        }
    }

    fn take_show_menu_requested(&self) -> bool {
        self.show_menu_requested.swap(false, Ordering::SeqCst)
    }

    fn take_clipboard_status(&self) -> Option<String> {
        let mut slot = self
            .clipboard_status
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        slot.take()
    }
}

impl Default for CliCompleter {
    fn default() -> Self {
        Self::new()
    }
}

impl Helper for CliCompleter {}
impl Hinter for CliCompleter {
    type Hint = String;

    fn hint(&self, line: &str, pos: usize, _ctx: &Context<'_>) -> Option<Self::Hint> {
        if pos != line.len() {
            return None;
        }

        predict_command_hint(line, self.predictive_input_enabled)
    }
}
impl Highlighter for CliCompleter {
    fn highlight<'l>(&self, line: &'l str, _pos: usize) -> Cow<'l, str> {
        if !self.colors_enabled {
            return Cow::Borrowed(line);
        }

        let language = detect_syntax_language(line);
        if language == SyntaxLanguage::Plain {
            return Cow::Borrowed(line);
        }

        match highlight_line_for_language(line, language, self.theme) {
            Some(highlighted) => Cow::Owned(highlighted),
            None => Cow::Borrowed(line),
        }
    }
}
impl Validator for CliCompleter {
    fn validate(&self, ctx: &mut ValidationContext) -> rustyline::Result<ValidationResult> {
        if has_multiline_continuation_marker(ctx.input()) {
            Ok(ValidationResult::Incomplete)
        } else {
            Ok(ValidationResult::Valid(None))
        }
    }
}

fn detect_syntax_language(line: &str) -> SyntaxLanguage {
    let trimmed = line.trim_start();
    if trimmed.is_empty() {
        return SyntaxLanguage::Plain;
    }

    if trimmed.starts_with('/') {
        return SyntaxLanguage::Command;
    }

    if trimmed.contains("namespace ")
        || trimmed.contains("using ")
        || trimmed.contains("public class")
    {
        return SyntaxLanguage::CSharp;
    }

    if trimmed.contains("interface ")
        || trimmed.contains("type ")
        || trimmed.contains(": string")
        || trimmed.contains(": number")
    {
        return SyntaxLanguage::TypeScript;
    }

    if trimmed.contains("function ")
        || trimmed.contains("const ")
        || trimmed.contains("let ")
        || trimmed.contains("=>")
    {
        return SyntaxLanguage::JavaScript;
    }

    if trimmed.contains("fn ")
        || trimmed.contains("let ")
        || trimmed.contains("impl ")
        || trimmed.contains("match ")
        || trimmed.contains("pub ")
    {
        return SyntaxLanguage::Rust;
    }

    SyntaxLanguage::Plain
}

fn comment_prefix(language: SyntaxLanguage) -> Option<&'static str> {
    match language {
        SyntaxLanguage::Rust
        | SyntaxLanguage::CSharp
        | SyntaxLanguage::JavaScript
        | SyntaxLanguage::TypeScript => Some("//"),
        SyntaxLanguage::Command => Some("#"),
        SyntaxLanguage::Plain => None,
    }
}

fn sanitize_keyword_token(token: &str) -> &str {
    token.trim_matches(|ch: char| !ch.is_alphanumeric() && ch != '_')
}

fn is_language_keyword(token: &str, language: SyntaxLanguage) -> bool {
    let token = sanitize_keyword_token(token);
    match language {
        SyntaxLanguage::Rust => RUST_KEYWORDS.contains(&token),
        SyntaxLanguage::CSharp => CSHARP_KEYWORDS.contains(&token),
        SyntaxLanguage::JavaScript => JAVASCRIPT_KEYWORDS.contains(&token),
        SyntaxLanguage::TypeScript => TYPESCRIPT_KEYWORDS.contains(&token),
        SyntaxLanguage::Plain | SyntaxLanguage::Command => false,
    }
}

fn keyword_color_for_token(
    token: &str,
    token_index: usize,
    language: SyntaxLanguage,
    theme: SyntaxTheme,
) -> Option<&'static str> {
    match language {
        SyntaxLanguage::Command => {
            if token_index == 0 && token.starts_with('/') {
                Some(theme.command)
            } else if token.starts_with("--") || token.starts_with('-') {
                Some(theme.flag)
            } else {
                None
            }
        }
        SyntaxLanguage::Rust
        | SyntaxLanguage::CSharp
        | SyntaxLanguage::JavaScript
        | SyntaxLanguage::TypeScript => {
            is_language_keyword(token, language).then_some(theme.keyword)
        }
        SyntaxLanguage::Plain => None,
    }
}

fn highlight_line_for_language(
    line: &str,
    language: SyntaxLanguage,
    theme: SyntaxTheme,
) -> Option<String> {
    if line.is_empty() {
        return None;
    }

    if line.len() > MAX_HIGHLIGHT_LINE_BYTES {
        return highlight_line_lexical(line, language, theme);
    }

    SYNTAX_HIGHLIGHT_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        if cache.language == Some(language) && cache.theme == Some(theme) && cache.line == line {
            return cache.highlighted.clone();
        }

        let highlighted = highlight_line_with_backends(line, language, theme);
        cache.line.clear();
        cache.line.push_str(line);
        cache.language = Some(language);
        cache.theme = Some(theme);
        cache.highlighted = highlighted.clone();
        highlighted
    })
}

fn highlight_line_with_backends(
    line: &str,
    language: SyntaxLanguage,
    theme: SyntaxTheme,
) -> Option<String> {
    if matches!(language, SyntaxLanguage::Plain | SyntaxLanguage::Command) {
        return highlight_line_lexical(line, language, theme);
    }

    tree_sitter_highlight_line(line, language, theme)
        .or_else(|| highlight_line_lexical(line, language, theme))
}

fn highlight_line_lexical(
    line: &str,
    language: SyntaxLanguage,
    theme: SyntaxTheme,
) -> Option<String> {
    if line.is_empty() {
        return None;
    }

    let comment = comment_prefix(language);
    let mut output = String::with_capacity(line.len() + 32);
    let mut cursor = 0usize;
    let mut token_index = 0usize;

    while cursor < line.len() {
        let Some(ch) = line[cursor..].chars().next() else {
            break;
        };

        if ch.is_whitespace() {
            output.push(ch);
            cursor += ch.len_utf8();
            continue;
        }

        if let Some(prefix) = comment {
            if line[cursor..].starts_with(prefix) {
                output.push_str(theme.comment);
                output.push_str(&line[cursor..]);
                output.push_str(theme.reset);
                return Some(output);
            }
        }

        if ch == '"' || ch == '\'' {
            let quote = ch;
            let start = cursor;
            cursor += ch.len_utf8();

            while cursor < line.len() {
                let Some(current) = line[cursor..].chars().next() else {
                    break;
                };

                cursor += current.len_utf8();

                if current == quote {
                    break;
                }

                if current == '\\' && cursor < line.len() {
                    if let Some(escaped) = line[cursor..].chars().next() {
                        cursor += escaped.len_utf8();
                    }
                }
            }

            output.push_str(theme.string);
            output.push_str(&line[start..cursor]);
            output.push_str(theme.reset);
            token_index += 1;
            continue;
        }

        let start = cursor;
        while cursor < line.len() {
            let Some(current) = line[cursor..].chars().next() else {
                break;
            };
            if current.is_whitespace() || current == '"' || current == '\'' {
                break;
            }

            if let Some(prefix) = comment {
                if line[cursor..].starts_with(prefix) {
                    break;
                }
            }

            cursor += current.len_utf8();
        }

        let token = &line[start..cursor];
        if let Some(color) = keyword_color_for_token(token, token_index, language, theme) {
            output.push_str(color);
            output.push_str(token);
            output.push_str(theme.reset);
        } else {
            output.push_str(token);
        }
        token_index += 1;
    }

    (output != line).then_some(output)
}

fn tree_sitter_highlight_line(
    line: &str,
    language: SyntaxLanguage,
    theme: SyntaxTheme,
) -> Option<String> {
    if matches!(language, SyntaxLanguage::Plain | SyntaxLanguage::Command) {
        return None;
    }

    let spans = TREE_SITTER_RUNTIME.with(|runtime| {
        let mut runtime = runtime.borrow_mut();
        let Some(parser) = runtime.parser_for(language) else {
            return Vec::new();
        };

        let Some(tree) = parser.parse(line, None) else {
            return Vec::new();
        };

        collect_tree_sitter_spans(tree.root_node(), line, language)
    });

    if spans.is_empty() {
        return None;
    }

    let mut output = String::with_capacity(line.len() + 32);
    let mut cursor = 0usize;

    for span in spans {
        if span.start < cursor || span.end > line.len() || span.start >= span.end {
            continue;
        }
        output.push_str(&line[cursor..span.start]);
        output.push_str(color_for_class(span.class, theme));
        output.push_str(&line[span.start..span.end]);
        output.push_str(theme.reset);
        cursor = span.end;
    }

    output.push_str(&line[cursor..]);

    if line_contains_comment_prefix(line, language) && !output.contains(theme.comment) {
        return highlight_line_lexical(line, language, theme);
    }

    (output != line).then_some(output)
}

fn line_contains_comment_prefix(line: &str, language: SyntaxLanguage) -> bool {
    comment_prefix(language).is_some_and(|prefix| line.contains(prefix))
}

fn collect_tree_sitter_spans(
    root: Node<'_>,
    source: &str,
    language: SyntaxLanguage,
) -> Vec<HighlightSpan> {
    let mut spans = Vec::new();
    collect_spans_from_node(root, source.as_bytes(), language, &mut spans);

    spans.sort_by(|left, right| {
        left.start
            .cmp(&right.start)
            .then_with(|| token_priority(right.class).cmp(&token_priority(left.class)))
            .then_with(|| left.end.cmp(&right.end))
    });

    let mut deduplicated = Vec::with_capacity(spans.len());
    for span in spans {
        if deduplicated
            .last()
            .is_some_and(|last: &HighlightSpan| span.start < last.end)
        {
            continue;
        }
        deduplicated.push(span);
    }

    deduplicated
}

fn collect_spans_from_node(
    node: Node<'_>,
    source: &[u8],
    language: SyntaxLanguage,
    spans: &mut Vec<HighlightSpan>,
) {
    if node.child_count() == 0 {
        let start = node.start_byte();
        let end = node.end_byte();
        if start >= end || end > source.len() {
            return;
        }

        let Ok(token) = std::str::from_utf8(&source[start..end]) else {
            return;
        };

        if let Some(class) = token_class_from_node(node.kind(), token, language) {
            spans.push(HighlightSpan { start, end, class });
        }
        return;
    }

    for index in 0..node.child_count() {
        if let Some(child) = node.child(index) {
            collect_spans_from_node(child, source, language, spans);
        }
    }
}

fn token_class_from_node(kind: &str, token: &str, language: SyntaxLanguage) -> Option<TokenClass> {
    let lowered = kind.to_ascii_lowercase();
    if lowered.contains("comment") {
        return Some(TokenClass::Comment);
    }

    if lowered.contains("string")
        || lowered.contains("char")
        || lowered.contains("template")
        || lowered.contains("raw")
    {
        return Some(TokenClass::String);
    }

    is_language_keyword(token, language).then_some(TokenClass::Keyword)
}

const fn token_priority(class: TokenClass) -> u8 {
    match class {
        TokenClass::Comment => 3,
        TokenClass::String => 2,
        TokenClass::Keyword => 1,
    }
}

const fn color_for_class(class: TokenClass, theme: SyntaxTheme) -> &'static str {
    match class {
        TokenClass::Keyword => theme.keyword,
        TokenClass::String => theme.string,
        TokenClass::Comment => theme.comment,
    }
}

impl Completer for CliCompleter {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Pair>)> {
        let prefix = line.get(..pos).unwrap_or(line);
        let candidates = completion_candidates(prefix)
            .into_iter()
            .map(|cmd| Pair {
                display: cmd.to_string(),
                replacement: cmd.to_string(),
            })
            .collect();
        Ok((0, candidates))
    }
}

fn completion_candidates(prefix: &str) -> Vec<&'static str> {
    COMPLETION_COMMANDS
        .iter()
        .copied()
        .filter(|candidate| candidate.starts_with(prefix))
        .collect()
}

fn predict_command_hint(prefix: &str, enabled: bool) -> Option<String> {
    if !enabled || prefix.is_empty() || !prefix.starts_with('/') || prefix.ends_with(' ') {
        return None;
    }

    let mut candidates: Vec<&str> = COMPLETION_COMMANDS
        .iter()
        .copied()
        .filter(|candidate| candidate.starts_with(prefix) && candidate.len() > prefix.len())
        .collect();

    if candidates.is_empty() {
        return None;
    }

    candidates
        .sort_unstable_by(|left, right| left.len().cmp(&right.len()).then_with(|| left.cmp(right)));

    candidates
        .first()
        .map(|candidate| candidate[prefix.len()..].to_string())
}

fn default_history_path() -> Option<PathBuf> {
    AppConfig::default_data_dir().map(|dir| dir.join("history").join("ntk.history"))
}

/// Rustyline-based interactive reader with persistent history and command completion.
pub struct RustylineInput {
    editor: Editor<CliCompleter, DefaultHistory>,
    history_path: Option<PathBuf>,
}

impl RustylineInput {
    /// Create a new Rustyline input reader.
    ///
    /// # Errors
    ///
    /// Returns an error when terminal editor initialization fails.
    pub fn new() -> io::Result<Self> {
        Self::new_with_predictive_input(true)
    }

    /// Create a new Rustyline input reader with explicit predictive input behavior.
    ///
    /// # Errors
    ///
    /// Returns an error when terminal editor initialization fails.
    pub fn new_with_predictive_input(predictive_input: bool) -> io::Result<Self> {
        let config = RustylineConfig::builder()
            .completion_type(CompletionType::List)
            .history_ignore_dups(true)
            .map_err(io::Error::other)?
            .auto_add_history(false)
            .build();

        let mut editor = Editor::<CliCompleter, DefaultHistory>::with_config(config)
            .map_err(io::Error::other)?;
        let helper = CliCompleter::with_predictive_input(predictive_input);
        editor.bind_sequence(
            RustylineEvent::from(RustylineKeyEvent::from('/')),
            EventHandler::Conditional(Box::new(helper.slash_menu_handler())),
        );
        editor.bind_sequence(
            RustylineEvent::from(RustylineKeyEvent::ctrl('V')),
            EventHandler::Conditional(Box::new(helper.clipboard_paste_handler())),
        );
        editor.bind_sequence(
            RustylineEvent::from(RustylineKeyEvent::alt('v')),
            EventHandler::Conditional(Box::new(helper.clipboard_paste_handler())),
        );
        editor.bind_sequence(
            RustylineEvent::from(RustylineKeyEvent::alt('c')),
            EventHandler::Conditional(Box::new(helper.clipboard_copy_handler())),
        );
        editor.set_helper(Some(helper));

        let history_path = default_history_path();
        if let Some(path) = &history_path {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            let _ = editor.load_history(path);
        }

        Ok(Self {
            editor,
            history_path,
        })
    }

    /// Read one line using Rustyline, returning normalized CLI input results.
    ///
    /// # Errors
    ///
    /// Returns an error when reading from terminal fails.
    pub fn read_line(&mut self, interrupted: &Arc<AtomicBool>) -> io::Result<InputResult> {
        if interrupted.load(Ordering::SeqCst) {
            return Ok(InputResult::Exit);
        }

        prepare_prompt_line()?;

        match self.editor.readline(PRIMARY_PROMPT) {
            Ok(raw_line) => {
                self.flush_clipboard_status();
                let line = normalize_multiline_submission(&raw_line);
                if line.trim() == "/" {
                    return Ok(InputResult::ShowMenu);
                }

                if !line.trim().is_empty() {
                    let _ = self.editor.add_history_entry(line.as_str());
                    self.persist_history();
                }

                if line.starts_with('/') {
                    Ok(InputResult::Command(line))
                } else {
                    Ok(InputResult::Text(line))
                }
            }
            Err(ReadlineError::Interrupted) => {
                self.flush_clipboard_status();
                if self.take_show_menu_requested() {
                    return Ok(InputResult::ShowMenu);
                }
                interrupted.store(true, Ordering::SeqCst);
                Ok(InputResult::Exit)
            }
            Err(ReadlineError::Eof) => {
                self.flush_clipboard_status();
                Ok(InputResult::Exit)
            }
            Err(err) => Err(io::Error::other(format!("Rustyline input failed: {err}"))),
        }
    }

    fn persist_history(&mut self) {
        if let Some(path) = &self.history_path {
            let _ = self.editor.save_history(path);
        }
    }

    fn take_show_menu_requested(&self) -> bool {
        self.editor
            .helper()
            .is_some_and(CliCompleter::take_show_menu_requested)
    }

    fn flush_clipboard_status(&self) {
        let status = self
            .editor
            .helper()
            .and_then(CliCompleter::take_clipboard_status);
        if let Some(message) = status {
            let _ = append_footer_log(&message);
        }
    }
}

/// Read a single line of interactive user input, handling keyboard events.
///
/// Returns an [`InputResult`] indicating whether the user entered a command,
/// free-form text, requested the menu, or chose to exit. The function also
/// processes terminal resize events while waiting for input.
pub async fn read_line(
    buffer: &mut String,
    interrupted: &Arc<AtomicBool>,
) -> io::Result<InputResult> {
    loop {
        // Check if interrupted before polling
        if interrupted.load(Ordering::SeqCst) {
            return Ok(InputResult::Exit);
        }

        // Use async-utils timeout for consistent timeout handling
        let poll_timeout = scheduled_frame_poll_timeout(std::time::Duration::from_millis(50));

        match with_timeout(poll_timeout, async {
            while !event::poll(std::time::Duration::from_millis(1))? {
                tokio::time::sleep(std::time::Duration::from_millis(1)).await;
            }
            event::read()
        })
        .await
        {
            Ok(Ok(event)) => match event {
                Event::Key(key_event) => match handle_key_event(key_event, buffer, interrupted)? {
                    Some(result) => return Ok(result),
                    None => continue,
                },
                Event::Resize(width, height) => {
                    if let Err(err) = handle_resize(width, height) {
                        let _ =
                            append_footer_log(&format!("Warning: failed to handle resize: {err}"));
                    }
                    request_terminal_frame();
                }
                Event::FocusGained => {
                    set_terminal_focused(true);
                }
                Event::FocusLost => {
                    set_terminal_focused(false);
                }
                _ => {}
            },
            Ok(Err(e)) => return Err(e),
            Err(_) => {
                // Timeout — process any pending deferred resize before continuing
                if let Err(err) = process_pending_resize() {
                    let _ = append_footer_log(&format!("Warning: resize processing failed: {err}"));
                }
                continue;
            }
        }
    }
}

fn handle_key_event(
    key: CrosstermKeyEvent,
    buffer: &mut String,
    interrupted: &Arc<AtomicBool>,
) -> io::Result<Option<InputResult>> {
    if key.kind != KeyEventKind::Press {
        return Ok(None);
    }

    match key.code {
        KeyCode::Char('v' | 'V')
            if key.modifiers.contains(KeyModifiers::CONTROL)
                || key.modifiers.contains(KeyModifiers::ALT) =>
        {
            match paste_from_clipboard() {
                Ok(text) => {
                    let normalized = normalize_clipboard_text_for_input(&text);
                    if normalized.is_empty() {
                        let _ = append_footer_log("Clipboard paste skipped: clipboard is empty");
                        return Ok(None);
                    }

                    buffer.push_str(&normalized);
                    print!("{}", normalized.white());
                    io::stdout().flush()?;
                    let _ = append_footer_log(&format!(
                        "Clipboard paste inserted {} chars",
                        normalized.chars().count()
                    ));
                    return Ok(None);
                }
                Err(err) => {
                    let _ = append_footer_log(&format!("Clipboard paste failed: {err}"));
                    return Ok(None);
                }
            }
        }
        KeyCode::Char('c' | 'C')
            if key.modifiers.contains(KeyModifiers::ALT)
                || (key.modifiers.contains(KeyModifiers::CONTROL)
                    && key.modifiers.contains(KeyModifiers::SHIFT)) =>
        {
            if buffer.trim().is_empty() {
                let _ = append_footer_log("Clipboard copy skipped: current input is empty");
                return Ok(None);
            }

            match copy_to_clipboard(buffer) {
                Ok(()) => {
                    let _ = append_footer_log(&format!(
                        "Clipboard copy captured {} chars",
                        buffer.chars().count()
                    ));
                }
                Err(err) => {
                    let _ = append_footer_log(&format!("Clipboard copy failed: {err}"));
                }
            }
            return Ok(None);
        }
        KeyCode::Char('c' | 'C') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            // Set the interrupted flag and return Exit
            interrupted.store(true, Ordering::SeqCst);
            return Ok(Some(InputResult::Exit));
        }
        KeyCode::Char(c) => {
            buffer.push(c);
            print!("{}", c.to_string().white());
            io::stdout().flush()?;

            // If user types "/" as first character, show menu immediately
            if c == '/' && buffer.len() == 1 {
                return Ok(Some(InputResult::ShowMenu));
            }
        }
        KeyCode::Backspace => {
            if !buffer.is_empty() {
                buffer.pop();
                print!("\x08 \x08");
                io::stdout().flush()?;
            }
        }
        KeyCode::Enter => {
            println!();
            let result = if buffer.starts_with('/') {
                InputResult::Command(buffer.clone())
            } else {
                InputResult::Text(buffer.clone())
            };
            return Ok(Some(result));
        }
        _ => {}
    }

    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
    use std::sync::atomic::AtomicBool;
    use std::sync::Arc;

    /// Creates a `KeyEvent` with `Press` kind.
    fn press(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
        KeyEvent {
            code,
            modifiers,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }
    }

    /// Creates a `KeyEvent` with `Release` kind (should be ignored).
    fn release(code: KeyCode) -> KeyEvent {
        KeyEvent {
            code,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Release,
            state: KeyEventState::NONE,
        }
    }

    #[test]
    fn ctrl_c_returns_exit_and_sets_interrupted() {
        let mut buffer = String::new();
        let interrupted = Arc::new(AtomicBool::new(false));

        let result = handle_key_event(
            press(KeyCode::Char('c'), KeyModifiers::CONTROL),
            &mut buffer,
            &interrupted,
        )
        .expect("should not error");

        assert!(matches!(result, Some(InputResult::Exit)));
        assert!(interrupted.load(Ordering::SeqCst));
    }

    #[test]
    fn enter_with_slash_prefix_returns_command() {
        let mut buffer = String::from("/help");
        let interrupted = Arc::new(AtomicBool::new(false));

        let result = handle_key_event(
            press(KeyCode::Enter, KeyModifiers::NONE),
            &mut buffer,
            &interrupted,
        )
        .expect("should not error");

        assert!(matches!(result, Some(InputResult::Command(cmd)) if cmd == "/help"));
    }

    #[test]
    fn enter_without_slash_prefix_returns_text() {
        let mut buffer = String::from("hello world");
        let interrupted = Arc::new(AtomicBool::new(false));

        let result = handle_key_event(
            press(KeyCode::Enter, KeyModifiers::NONE),
            &mut buffer,
            &interrupted,
        )
        .expect("should not error");

        assert!(matches!(result, Some(InputResult::Text(txt)) if txt == "hello world"));
    }

    #[test]
    fn enter_with_empty_buffer_returns_text() {
        let mut buffer = String::new();
        let interrupted = Arc::new(AtomicBool::new(false));

        let result = handle_key_event(
            press(KeyCode::Enter, KeyModifiers::NONE),
            &mut buffer,
            &interrupted,
        )
        .expect("should not error");

        assert!(matches!(result, Some(InputResult::Text(txt)) if txt.is_empty()));
    }

    #[test]
    fn slash_as_first_char_returns_show_menu() {
        let mut buffer = String::new();
        let interrupted = Arc::new(AtomicBool::new(false));

        let result = handle_key_event(
            press(KeyCode::Char('/'), KeyModifiers::NONE),
            &mut buffer,
            &interrupted,
        )
        .expect("should not error");

        assert_eq!(buffer, "/");
        assert!(matches!(result, Some(InputResult::ShowMenu)));
    }

    #[test]
    fn slash_not_first_char_does_not_show_menu() {
        let mut buffer = String::from("a");
        let interrupted = Arc::new(AtomicBool::new(false));

        let result = handle_key_event(
            press(KeyCode::Char('/'), KeyModifiers::NONE),
            &mut buffer,
            &interrupted,
        )
        .expect("should not error");

        assert_eq!(buffer, "a/");
        assert!(result.is_none());
    }

    #[test]
    fn regular_char_is_appended_to_buffer() {
        let mut buffer = String::new();
        let interrupted = Arc::new(AtomicBool::new(false));

        let result = handle_key_event(
            press(KeyCode::Char('x'), KeyModifiers::NONE),
            &mut buffer,
            &interrupted,
        )
        .expect("should not error");

        assert_eq!(buffer, "x");
        assert!(result.is_none());
    }

    #[test]
    fn backspace_removes_last_char() {
        let mut buffer = String::from("abc");
        let interrupted = Arc::new(AtomicBool::new(false));

        let result = handle_key_event(
            press(KeyCode::Backspace, KeyModifiers::NONE),
            &mut buffer,
            &interrupted,
        )
        .expect("should not error");

        assert_eq!(buffer, "ab");
        assert!(result.is_none());
    }

    #[test]
    fn backspace_on_empty_buffer_is_no_op() {
        let mut buffer = String::new();
        let interrupted = Arc::new(AtomicBool::new(false));

        let result = handle_key_event(
            press(KeyCode::Backspace, KeyModifiers::NONE),
            &mut buffer,
            &interrupted,
        )
        .expect("should not error");

        assert!(buffer.is_empty());
        assert!(result.is_none());
    }

    #[test]
    fn release_event_is_ignored() {
        let mut buffer = String::new();
        let interrupted = Arc::new(AtomicBool::new(false));

        let result = handle_key_event(release(KeyCode::Char('a')), &mut buffer, &interrupted)
            .expect("should not error");

        assert!(buffer.is_empty());
        assert!(result.is_none());
    }

    #[test]
    fn unknown_key_code_returns_none() {
        let mut buffer = String::new();
        let interrupted = Arc::new(AtomicBool::new(false));

        let result = handle_key_event(
            press(KeyCode::F(1), KeyModifiers::NONE),
            &mut buffer,
            &interrupted,
        )
        .expect("should not error");

        assert!(buffer.is_empty());
        assert!(result.is_none());
    }

    #[test]
    fn multiple_chars_build_buffer_correctly() {
        let mut buffer = String::new();
        let interrupted = Arc::new(AtomicBool::new(false));

        for ch in ['h', 'e', 'l', 'l', 'o'] {
            handle_key_event(
                press(KeyCode::Char(ch), KeyModifiers::NONE),
                &mut buffer,
                &interrupted,
            )
            .expect("should not error");
        }

        assert_eq!(buffer, "hello");
    }

    #[test]
    fn completion_candidates_match_known_commands() {
        let candidates = completion_candidates("/man");
        assert!(candidates.contains(&"/manifest"));
        assert!(candidates.contains(&"/manifest list"));

        let ai_candidates = completion_candidates("/ai");
        assert!(ai_candidates.contains(&"/ai ask"));
        assert!(ai_candidates.contains(&"/ai plan"));
        assert!(ai_candidates.contains(&"/ai resume"));
        assert!(ai_candidates.contains(&"/ai apply --approve-write"));

        let task_candidates = completion_candidates("/task");
        assert!(task_candidates.contains(&"/task submit"));
        assert!(task_candidates.contains(&"/task list"));
        assert!(task_candidates.contains(&"/task watch"));
        assert!(task_candidates.contains(&"/task cancel"));
    }

    #[test]
    fn completion_candidates_empty_for_unknown_prefix() {
        let candidates = completion_candidates("/does-not-exist");
        assert!(candidates.is_empty());
    }

    #[test]
    fn predict_command_hint_returns_suffix_for_partial_command() {
        let hint = predict_command_hint("/man", true);
        assert_eq!(hint.as_deref(), Some("ifest"));

        let hint = predict_command_hint("/manifest re", true);
        assert_eq!(hint.as_deref(), Some("nder"));

        let hint = predict_command_hint("/ai pl", true);
        assert_eq!(hint.as_deref(), Some("an"));

        let hint = predict_command_hint("/task su", true);
        assert_eq!(hint.as_deref(), Some("bmit"));
    }

    #[test]
    fn predict_command_hint_returns_none_for_non_command_or_complete_command() {
        assert!(predict_command_hint("manifest", true).is_none());
        assert!(predict_command_hint("/help", true).is_none());
        assert!(predict_command_hint("/manifest ", true).is_none());
        assert!(predict_command_hint("/does-not-exist", true).is_none());
    }

    #[test]
    fn predict_command_hint_respects_runtime_toggle() {
        assert_eq!(predict_command_hint("/man", false), None);
    }

    #[test]
    fn multiline_continuation_marker_detects_single_trailing_backslash() {
        assert!(has_multiline_continuation_marker("render \\"));
        assert!(has_multiline_continuation_marker("render\\"));
    }

    #[test]
    fn multiline_continuation_marker_ignores_even_trailing_backslashes() {
        assert!(!has_multiline_continuation_marker("C:\\\\"));
        assert!(!has_multiline_continuation_marker("literal \\\\"));
    }

    #[test]
    fn strip_multiline_continuation_marker_removes_only_marker() {
        assert_eq!(
            strip_multiline_continuation_marker("line with marker \\"),
            "line with marker "
        );
        assert_eq!(
            strip_multiline_continuation_marker("line without marker"),
            "line without marker"
        );
    }

    #[test]
    fn normalize_multiline_submission_removes_markers_line_by_line() {
        let input = "/manifest apply \\\n--output out \\\nmanifest.yaml";
        let normalized = normalize_multiline_submission(input);
        assert_eq!(normalized, "/manifest apply \n--output out \nmanifest.yaml");
    }

    #[test]
    fn normalize_clipboard_text_for_input_flattens_newlines() {
        let normalized = normalize_clipboard_text_for_input("line 1\r\nline 2\n\nline 3");
        assert_eq!(normalized, "line 1 line 2 line 3");
    }

    #[test]
    fn alt_c_with_empty_buffer_is_noop_not_exit() {
        let mut buffer = String::new();
        let interrupted = Arc::new(AtomicBool::new(false));

        let result = handle_key_event(
            press(KeyCode::Char('c'), KeyModifiers::ALT),
            &mut buffer,
            &interrupted,
        )
        .expect("should not error");

        assert!(result.is_none());
        assert!(!interrupted.load(Ordering::SeqCst));
    }

    #[test]
    fn ctrl_shift_c_with_empty_buffer_is_noop_not_exit() {
        let mut buffer = String::new();
        let interrupted = Arc::new(AtomicBool::new(false));

        let result = handle_key_event(
            press(
                KeyCode::Char('c'),
                KeyModifiers::CONTROL | KeyModifiers::SHIFT,
            ),
            &mut buffer,
            &interrupted,
        )
        .expect("should not error");

        assert!(result.is_none());
        assert!(!interrupted.load(Ordering::SeqCst));
    }

    #[test]
    fn detect_syntax_language_identifies_supported_variants() {
        assert_eq!(
            detect_syntax_language("/manifest apply"),
            SyntaxLanguage::Command
        );
        assert_eq!(
            detect_syntax_language("pub fn run() -> Result<()>"),
            SyntaxLanguage::Rust
        );
        assert_eq!(
            detect_syntax_language("public class AppService"),
            SyntaxLanguage::CSharp
        );
        assert_eq!(
            detect_syntax_language("const run = async () => {}"),
            SyntaxLanguage::JavaScript
        );
        assert_eq!(
            detect_syntax_language("interface AppConfig { retries: number }"),
            SyntaxLanguage::TypeScript
        );
        assert_eq!(detect_syntax_language("free text"), SyntaxLanguage::Plain);
    }

    #[test]
    fn show_menu_requested_flag_resets_after_consume() {
        let helper = CliCompleter::new();
        assert!(!helper.take_show_menu_requested());

        helper.show_menu_requested.store(true, Ordering::SeqCst);
        assert!(helper.take_show_menu_requested());
        assert!(!helper.take_show_menu_requested());
    }

    #[test]
    fn highlight_command_line_includes_command_flags_and_strings() {
        let theme = SyntaxTheme::default();
        let highlighted = highlight_line_for_language(
            "/manifest apply --path \"templates/app\" # note",
            SyntaxLanguage::Command,
            theme,
        )
        .expect("expected highlighted output");

        assert!(highlighted.contains(theme.command));
        assert!(highlighted.contains(theme.flag));
        assert!(highlighted.contains(theme.string));
        assert!(highlighted.contains(theme.comment));
        assert!(highlighted.contains(theme.reset));
    }

    #[test]
    fn highlight_rust_line_includes_keyword_and_comment_colors() {
        let theme = SyntaxTheme::default();
        let highlighted = highlight_line_for_language(
            "pub fn run() { let value = \"ok\"; // status }",
            SyntaxLanguage::Rust,
            theme,
        )
        .expect("expected highlighted output");

        assert!(highlighted.contains(theme.keyword));
        assert!(highlighted.contains(theme.string));
        assert!(highlighted.contains(theme.comment));
        assert!(highlighted.contains(theme.reset));
    }

    #[test]
    fn tree_sitter_highlight_marks_rust_comment_tokens() {
        let theme = SyntaxTheme::default();
        let highlighted =
            tree_sitter_highlight_line("fn run() { // note }", SyntaxLanguage::Rust, theme)
                .expect("expected tree-sitter highlight output");

        assert!(highlighted.contains(theme.comment));
        assert!(highlighted.contains(theme.reset));
    }

    #[test]
    fn highlight_cache_stores_last_line_and_result() {
        let theme = SyntaxTheme::default();
        let line = "pub fn run() { let value = \"ok\"; }";

        let first = highlight_line_for_language(line, SyntaxLanguage::Rust, theme);
        let second = highlight_line_for_language(line, SyntaxLanguage::Rust, theme);
        assert_eq!(first, second);

        SYNTAX_HIGHLIGHT_CACHE.with(|cache| {
            let cache = cache.borrow();
            assert_eq!(cache.line, line);
            assert_eq!(cache.language, Some(SyntaxLanguage::Rust));
            assert_eq!(cache.theme, Some(theme));
            assert_eq!(cache.highlighted, second);
        });
    }

    #[test]
    fn highlight_long_line_uses_fallback_without_panicking() {
        let theme = SyntaxTheme::default();
        let long_payload = "x".repeat(MAX_HIGHLIGHT_LINE_BYTES + 64);
        let line = format!("pub fn run() {{ {long_payload} }}");

        let highlighted = highlight_line_for_language(&line, SyntaxLanguage::Rust, theme)
            .expect("expected lexical fallback output");

        assert!(highlighted.contains(theme.keyword));
    }

    #[test]
    fn highlight_plain_text_returns_none() {
        let theme = SyntaxTheme::default();
        assert!(highlight_line_for_language("plain text", SyntaxLanguage::Plain, theme).is_none());
    }

    #[test]
    fn default_history_path_has_expected_file_name() {
        if let Some(path) = default_history_path() {
            assert_eq!(
                path.file_name().and_then(|value| value.to_str()),
                Some("ntk.history")
            );
        }
    }
}
