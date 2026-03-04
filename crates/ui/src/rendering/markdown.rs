//! Markdown rendering helpers for terminal output.
//!
//! Converts Markdown into readable terminal text, preserving headings, lists,
//! code spans/blocks, links, and basic emphasis.

use crate::core::capabilities::capabilities;
use owo_colors::OwoColorize;
use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag, TagEnd};

#[derive(Debug, Clone, Copy, Default)]
struct InlineState {
    strong: usize,
    emphasis: usize,
}

#[derive(Debug, Clone, Copy)]
struct ListState {
    ordered: bool,
    next_index: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CodeBlockLanguage {
    Plain,
    Rust,
    CSharp,
    JavaScript,
    TypeScript,
    Json,
    Toml,
    Bash,
    PowerShell,
}

const ANSI_CODE_DEFAULT: &str = "\x1b[90m";
const ANSI_CODE_KEYWORD: &str = "\x1b[1;94m";
const ANSI_CODE_STRING: &str = "\x1b[33m";
const ANSI_CODE_COMMENT: &str = "\x1b[2;90m";
const ANSI_CODE_NUMBER: &str = "\x1b[36m";
const ANSI_RESET: &str = "\x1b[0m";

const RUST_CODE_KEYWORDS: &[&str] = &[
    "as", "break", "const", "continue", "crate", "else", "enum", "extern", "fn", "for", "if",
    "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref", "return", "self",
    "Self", "static", "struct", "trait", "type", "unsafe", "use", "where", "while",
];
const CSHARP_CODE_KEYWORDS: &[&str] = &[
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
const JAVASCRIPT_CODE_KEYWORDS: &[&str] = &[
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
const TYPESCRIPT_CODE_KEYWORDS: &[&str] = &[
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
const JSON_CODE_KEYWORDS: &[&str] = &["true", "false", "null"];
const BASH_CODE_KEYWORDS: &[&str] = &[
    "if", "then", "else", "fi", "for", "in", "do", "done", "while", "case", "esac", "function",
];
const POWERSHELL_CODE_KEYWORDS: &[&str] = &[
    "if", "else", "foreach", "for", "while", "function", "param", "return", "switch", "try",
    "catch",
];

/// Render Markdown into a terminal-friendly string.
///
/// The renderer is intentionally lightweight and optimized for CLI help output.
/// It supports headings, paragraphs, ordered/unordered lists, links, inline
/// code, fenced code blocks, and emphasis markers.
#[must_use]
pub fn render_markdown(markdown: &str) -> String {
    let parser = Parser::new_ext(
        markdown,
        Options::ENABLE_STRIKETHROUGH
            | Options::ENABLE_TABLES
            | Options::ENABLE_TASKLISTS
            | Options::ENABLE_FOOTNOTES,
    );

    let color_enabled = capabilities().color.has_color();

    let mut out = String::new();
    let mut inline = InlineState::default();
    let mut heading_level: Option<HeadingLevel> = None;
    let mut list_stack: Vec<ListState> = Vec::new();
    let mut code_block = false;
    let mut code_block_language: Option<CodeBlockLanguage> = None;
    let mut current_link: Option<String> = None;

    for event in parser {
        match event {
            Event::Start(tag) => match tag {
                Tag::Paragraph => {}
                Tag::Heading { level, .. } => {
                    ensure_section_break(&mut out);
                    heading_level = Some(level);
                    if !color_enabled {
                        out.push_str(&"#".repeat(heading_rank(level)));
                        out.push(' ');
                    }
                }
                Tag::List(start) => {
                    ensure_line_break(&mut out);
                    let next_index = start.unwrap_or(1);
                    list_stack.push(ListState {
                        ordered: start.is_some(),
                        next_index,
                    });
                }
                Tag::Item => {
                    ensure_line_break(&mut out);
                    let depth = list_stack.len().saturating_sub(1);
                    out.push_str(&"  ".repeat(depth));
                    if let Some(last) = list_stack.last_mut() {
                        if last.ordered {
                            out.push_str(&format!("{}. ", last.next_index));
                            last.next_index = last.next_index.saturating_add(1);
                        } else {
                            out.push_str("- ");
                        }
                    } else {
                        out.push_str("- ");
                    }
                }
                Tag::CodeBlock(kind) => {
                    ensure_section_break(&mut out);
                    code_block = true;
                    code_block_language = extract_code_block_language(&kind);
                }
                Tag::Emphasis => inline.emphasis = inline.emphasis.saturating_add(1),
                Tag::Strong => inline.strong = inline.strong.saturating_add(1),
                Tag::Link { dest_url, .. } => current_link = Some(dest_url.to_string()),
                _ => {}
            },
            Event::End(tag_end) => match tag_end {
                TagEnd::Paragraph => out.push_str("\n\n"),
                TagEnd::Heading(_) => {
                    heading_level = None;
                    out.push_str("\n\n");
                }
                TagEnd::List(_) => {
                    list_stack.pop();
                    out.push('\n');
                }
                TagEnd::Item => out.push('\n'),
                TagEnd::CodeBlock => {
                    code_block = false;
                    code_block_language = None;
                    out.push_str("\n\n");
                }
                TagEnd::Emphasis => inline.emphasis = inline.emphasis.saturating_sub(1),
                TagEnd::Strong => inline.strong = inline.strong.saturating_sub(1),
                TagEnd::Link => {
                    if let Some(link) = current_link.take() {
                        if color_enabled {
                            out.push_str(&format!(" {}", format!("({link})").blue()));
                        } else {
                            out.push_str(&format!(" ({link})"));
                        }
                    }
                }
                _ => {}
            },
            Event::Text(text) => {
                let rendered = if code_block {
                    render_code_block_segment(
                        &text,
                        code_block_language.unwrap_or(CodeBlockLanguage::Plain),
                        color_enabled,
                    )
                } else {
                    render_text_segment(&text, inline, heading_level, false, color_enabled)
                };
                out.push_str(&rendered);
            }
            Event::Code(code) => {
                if color_enabled {
                    out.push_str(&format!("{}", format!("`{code}`").bright_yellow()));
                } else {
                    out.push_str(&format!("`{code}`"));
                }
            }
            Event::SoftBreak | Event::HardBreak => out.push('\n'),
            Event::Rule => out.push_str("\n---\n"),
            Event::Html(text) | Event::InlineHtml(text) => out.push_str(&text),
            Event::FootnoteReference(name) => out.push_str(&format!("[{name}]")),
            Event::TaskListMarker(checked) => {
                if checked {
                    out.push_str("[x] ");
                } else {
                    out.push_str("[ ] ");
                }
            }
            _ => {}
        }
    }

    while out.ends_with('\n') {
        out.pop();
    }
    out
}

fn render_text_segment(
    text: &str,
    inline: InlineState,
    heading_level: Option<HeadingLevel>,
    code_block: bool,
    color_enabled: bool,
) -> String {
    if !color_enabled {
        return text.to_string();
    }

    if code_block {
        return format!("{}", text.bright_black());
    }

    if let Some(level) = heading_level {
        let styled = match level {
            HeadingLevel::H1 => format!("{}", text.bold().cyan()),
            HeadingLevel::H2 => format!("{}", text.bold().blue()),
            HeadingLevel::H3 => format!("{}", text.bold().green()),
            _ => format!("{}", text.bold().white()),
        };
        return styled;
    }

    if inline.strong > 0 && inline.emphasis > 0 {
        return format!("{}", text.bold().italic());
    }
    if inline.strong > 0 {
        return format!("{}", text.bold());
    }
    if inline.emphasis > 0 {
        return format!("{}", text.italic());
    }

    text.to_string()
}

fn extract_code_block_language(kind: &CodeBlockKind<'_>) -> Option<CodeBlockLanguage> {
    match kind {
        CodeBlockKind::Indented => Some(CodeBlockLanguage::Plain),
        CodeBlockKind::Fenced(info) => parse_code_block_language(info),
    }
}

fn parse_code_block_language(info: &str) -> Option<CodeBlockLanguage> {
    let label = info
        .split_whitespace()
        .next()
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase();

    match label.as_str() {
        "" => None,
        "rust" | "rs" => Some(CodeBlockLanguage::Rust),
        "csharp" | "c#" | "cs" => Some(CodeBlockLanguage::CSharp),
        "javascript" | "js" => Some(CodeBlockLanguage::JavaScript),
        "typescript" | "ts" => Some(CodeBlockLanguage::TypeScript),
        "json" | "jsonc" => Some(CodeBlockLanguage::Json),
        "toml" => Some(CodeBlockLanguage::Toml),
        "bash" | "sh" | "zsh" | "shell" => Some(CodeBlockLanguage::Bash),
        "powershell" | "ps1" | "pwsh" => Some(CodeBlockLanguage::PowerShell),
        _ => Some(CodeBlockLanguage::Plain),
    }
}

fn render_code_block_segment(
    text: &str,
    language: CodeBlockLanguage,
    color_enabled: bool,
) -> String {
    if !color_enabled {
        return text.to_string();
    }

    text.split_inclusive('\n')
        .map(|line| render_code_block_line(line, language))
        .collect::<Vec<_>>()
        .join("")
}

fn render_code_block_line(line: &str, language: CodeBlockLanguage) -> String {
    let (content, newline) = if let Some(stripped) = line.strip_suffix('\n') {
        (stripped, "\n")
    } else {
        (line, "")
    };

    let mut out = String::new();
    if let Some(comment_start) = find_comment_start(content, language) {
        let (code_part, comment_part) = content.split_at(comment_start);
        out.push_str(&render_code_without_comment(code_part, language));
        push_ansi_segment(&mut out, ANSI_CODE_COMMENT, comment_part);
    } else {
        out.push_str(&render_code_without_comment(content, language));
    }
    out.push_str(newline);
    out
}

fn render_code_without_comment(content: &str, language: CodeBlockLanguage) -> String {
    if content.is_empty() {
        return String::new();
    }

    let chars: Vec<(usize, char)> = content.char_indices().collect();
    let mut out = String::with_capacity(content.len() + 16);
    let mut plain_buffer = String::new();
    let mut index = 0usize;

    while index < chars.len() {
        let start_byte = chars[index].0;
        let current = chars[index].1;

        if is_string_delimiter(language, current) {
            flush_plain_buffer(&mut out, &mut plain_buffer);
            let mut next_index = index + 1;
            let mut escaped = false;
            while next_index < chars.len() {
                let ch = chars[next_index].1;
                if escaped {
                    escaped = false;
                } else if ch == '\\' {
                    escaped = true;
                } else if ch == current {
                    next_index += 1;
                    break;
                }
                next_index += 1;
            }
            let token = slice_chars(content, &chars, start_byte, next_index);
            push_ansi_segment(&mut out, ANSI_CODE_STRING, token);
            index = next_index;
            continue;
        }

        if is_identifier_start(current) {
            let mut next_index = index + 1;
            while next_index < chars.len() && is_identifier_continue(chars[next_index].1) {
                next_index += 1;
            }
            let token = slice_chars(content, &chars, start_byte, next_index);
            if is_code_keyword(language, token) {
                flush_plain_buffer(&mut out, &mut plain_buffer);
                push_ansi_segment(&mut out, ANSI_CODE_KEYWORD, token);
            } else {
                plain_buffer.push_str(token);
            }
            index = next_index;
            continue;
        }

        if current.is_ascii_digit() {
            flush_plain_buffer(&mut out, &mut plain_buffer);
            let mut next_index = index + 1;
            while next_index < chars.len() && is_number_continue(chars[next_index].1) {
                next_index += 1;
            }
            let token = slice_chars(content, &chars, start_byte, next_index);
            push_ansi_segment(&mut out, ANSI_CODE_NUMBER, token);
            index = next_index;
            continue;
        }

        plain_buffer.push(current);
        index += 1;
    }

    flush_plain_buffer(&mut out, &mut plain_buffer);
    out
}

fn find_comment_start(content: &str, language: CodeBlockLanguage) -> Option<usize> {
    let marker = match language {
        CodeBlockLanguage::Rust
        | CodeBlockLanguage::CSharp
        | CodeBlockLanguage::JavaScript
        | CodeBlockLanguage::TypeScript => "//",
        CodeBlockLanguage::Toml | CodeBlockLanguage::Bash | CodeBlockLanguage::PowerShell => "#",
        CodeBlockLanguage::Json | CodeBlockLanguage::Plain => return None,
    };

    let chars: Vec<(usize, char)> = content.char_indices().collect();
    let mut index = 0usize;
    let mut in_string: Option<char> = None;
    let mut escaped = false;

    while index < chars.len() {
        let byte_start = chars[index].0;
        let current = chars[index].1;

        if let Some(delimiter) = in_string {
            if escaped {
                escaped = false;
            } else if current == '\\' {
                escaped = true;
            } else if current == delimiter {
                in_string = None;
            }
            index += 1;
            continue;
        }

        if is_string_delimiter(language, current) {
            in_string = Some(current);
            index += 1;
            continue;
        }

        if marker == "#" && current == '#' {
            return Some(byte_start);
        }

        if marker == "//" && current == '/' && index + 1 < chars.len() && chars[index + 1].1 == '/'
        {
            return Some(byte_start);
        }

        index += 1;
    }

    None
}

fn is_string_delimiter(language: CodeBlockLanguage, ch: char) -> bool {
    match language {
        CodeBlockLanguage::JavaScript | CodeBlockLanguage::TypeScript => {
            ch == '"' || ch == '\'' || ch == '`'
        }
        CodeBlockLanguage::PowerShell => ch == '"' || ch == '\'',
        _ => ch == '"',
    }
}

fn is_identifier_start(ch: char) -> bool {
    ch == '_' || ch.is_ascii_alphabetic()
}

fn is_identifier_continue(ch: char) -> bool {
    is_identifier_start(ch) || ch.is_ascii_digit()
}

fn is_number_continue(ch: char) -> bool {
    ch.is_ascii_hexdigit() || matches!(ch, '_' | '.' | 'x' | 'X' | 'b' | 'B' | 'o' | 'O')
}

fn is_code_keyword(language: CodeBlockLanguage, token: &str) -> bool {
    let keywords: &[&str] = match language {
        CodeBlockLanguage::Rust => RUST_CODE_KEYWORDS,
        CodeBlockLanguage::CSharp => CSHARP_CODE_KEYWORDS,
        CodeBlockLanguage::JavaScript => JAVASCRIPT_CODE_KEYWORDS,
        CodeBlockLanguage::TypeScript => TYPESCRIPT_CODE_KEYWORDS,
        CodeBlockLanguage::Json => JSON_CODE_KEYWORDS,
        CodeBlockLanguage::Toml => &[],
        CodeBlockLanguage::Bash => BASH_CODE_KEYWORDS,
        CodeBlockLanguage::PowerShell => POWERSHELL_CODE_KEYWORDS,
        CodeBlockLanguage::Plain => &[],
    };
    keywords.contains(&token)
}

fn slice_chars<'a>(
    content: &'a str,
    chars: &[(usize, char)],
    start_byte: usize,
    end_index: usize,
) -> &'a str {
    let end_byte = chars
        .get(end_index)
        .map_or(content.len(), |(byte, _)| *byte);
    &content[start_byte..end_byte]
}

fn flush_plain_buffer(out: &mut String, plain_buffer: &mut String) {
    if plain_buffer.is_empty() {
        return;
    }
    push_ansi_segment(out, ANSI_CODE_DEFAULT, plain_buffer);
    plain_buffer.clear();
}

fn push_ansi_segment(out: &mut String, style: &str, segment: &str) {
    if segment.is_empty() {
        return;
    }
    out.push_str(style);
    out.push_str(segment);
    out.push_str(ANSI_RESET);
}

fn ensure_line_break(out: &mut String) {
    if !out.is_empty() && !out.ends_with('\n') {
        out.push('\n');
    }
}

fn ensure_section_break(out: &mut String) {
    if out.is_empty() || out.ends_with("\n\n") {
        return;
    }
    if out.ends_with('\n') {
        out.push('\n');
    } else {
        out.push_str("\n\n");
    }
}

fn heading_rank(level: HeadingLevel) -> usize {
    match level {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        find_comment_start, parse_code_block_language, render_code_block_segment,
        CodeBlockLanguage, ANSI_CODE_KEYWORD, ANSI_CODE_STRING,
    };

    #[test]
    fn parse_code_block_language_maps_common_aliases() {
        assert_eq!(
            parse_code_block_language("rust"),
            Some(CodeBlockLanguage::Rust)
        );
        assert_eq!(
            parse_code_block_language("c#"),
            Some(CodeBlockLanguage::CSharp)
        );
        assert_eq!(
            parse_code_block_language("typescript title=test"),
            Some(CodeBlockLanguage::TypeScript)
        );
        assert_eq!(
            parse_code_block_language("pwsh"),
            Some(CodeBlockLanguage::PowerShell)
        );
    }

    #[test]
    fn render_code_block_segment_highlights_keywords_and_strings() {
        let rendered =
            render_code_block_segment("let name = \"ntk\";", CodeBlockLanguage::Rust, true);
        assert!(rendered.contains(&format!("{ANSI_CODE_KEYWORD}let")));
        assert!(rendered.contains(&format!("{ANSI_CODE_STRING}\"ntk\"")));
    }

    #[test]
    fn render_code_block_segment_returns_plain_text_when_color_disabled() {
        let rendered =
            render_code_block_segment("let name = \"ntk\";", CodeBlockLanguage::Rust, false);
        assert_eq!(rendered, "let name = \"ntk\";");
    }

    #[test]
    fn comment_detection_ignores_markers_inside_strings() {
        let idx = find_comment_start(
            "let x = \"http://example\"; // trailing comment",
            CodeBlockLanguage::Rust,
        )
        .expect("must detect trailing comment");
        assert_eq!(
            &"let x = \"http://example\"; // trailing comment"[idx..],
            "// trailing comment"
        );
    }
}
