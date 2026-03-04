//! Workspace context collection utilities for AI-assisted workflows.
//!
//! This module provides a deterministic, local-only context collector with:
//! - Explicit allowlisted file paths
//! - Byte-budget enforcement (per file and total)
//! - Basic secret redaction before prompt assembly

use std::fs;
use std::path::{Component, Path, PathBuf};

const REDACTION_MARKER: &str = "[REDACTED]";
const SECRET_HINTS: [&str; 8] = [
    "api_key",
    "apikey",
    "access_token",
    "token",
    "secret",
    "password",
    "passwd",
    "authorization",
];

/// Budget constraints for collected AI context.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AiContextBudget {
    /// Maximum number of files to include.
    pub max_files: usize,
    /// Maximum bytes included per file.
    pub max_file_bytes: usize,
    /// Maximum bytes included across all files.
    pub max_total_bytes: usize,
}

impl Default for AiContextBudget {
    fn default() -> Self {
        Self {
            max_files: 4,
            max_file_bytes: 2_048,
            max_total_bytes: 8_192,
        }
    }
}

/// Single context file entry captured for AI request enrichment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AiContextFile {
    /// Relative path used in the prompt context.
    pub relative_path: PathBuf,
    /// Redacted and budgeted content included in context.
    pub content: String,
    /// Original text size in bytes before truncation.
    pub original_bytes: usize,
    /// Included text size in bytes after truncation.
    pub included_bytes: usize,
    /// Indicates whether this entry content was truncated.
    pub truncated: bool,
    /// Number of redaction events applied for this file.
    pub redactions: usize,
}

/// Aggregate context payload built from allowlisted files.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AiContextBundle {
    /// Included files in deterministic order.
    pub files: Vec<AiContextFile>,
    /// Total included bytes across all files.
    pub total_bytes: usize,
    /// Indicates whether truncation happened due to budget constraints.
    pub truncated: bool,
    /// Non-fatal skips collected while scanning allowlisted files.
    pub skipped: Vec<String>,
}

impl AiContextBundle {
    /// Returns `true` when no files were included.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }
}

/// Collect workspace context from explicit allowlisted relative paths.
///
/// The collector:
/// - Rejects absolute and parent-traversal (`..`) paths
/// - Requires final canonical path to remain under `workspace_root`
/// - Applies secret redaction before byte truncation
/// - Enforces both per-file and total byte budgets
#[must_use]
pub fn collect_workspace_context(
    workspace_root: &Path,
    allowlist_relative_paths: &[PathBuf],
    budget: AiContextBudget,
) -> AiContextBundle {
    let mut bundle = AiContextBundle::default();
    let canonical_root = workspace_root
        .canonicalize()
        .unwrap_or_else(|_| workspace_root.to_path_buf());

    if budget.max_files == 0 || budget.max_total_bytes == 0 || budget.max_file_bytes == 0 {
        bundle.truncated = true;
        return bundle;
    }

    for relative_candidate in allowlist_relative_paths.iter().take(budget.max_files) {
        if bundle.total_bytes >= budget.max_total_bytes {
            bundle.truncated = true;
            break;
        }

        let Some(relative_path) = normalize_relative_path(relative_candidate) else {
            bundle.skipped.push(format!(
                "invalid allowlist path: {}",
                relative_candidate.display()
            ));
            continue;
        };

        let absolute_candidate = canonical_root.join(&relative_path);
        let canonical_candidate = match absolute_candidate.canonicalize() {
            Ok(path) => path,
            Err(_) => {
                bundle
                    .skipped
                    .push(format!("missing file: {}", relative_path.display()));
                continue;
            }
        };

        if !canonical_candidate.starts_with(&canonical_root) {
            bundle.skipped.push(format!(
                "path outside workspace rejected: {}",
                relative_path.display()
            ));
            continue;
        }

        let raw_text = match fs::read_to_string(&canonical_candidate) {
            Ok(text) => text,
            Err(_) => {
                bundle.skipped.push(format!(
                    "non-text or unreadable file skipped: {}",
                    relative_path.display()
                ));
                continue;
            }
        };

        let original_bytes = raw_text.len();
        let (redacted_text, redactions) = redact_secrets(&raw_text);

        let (per_file_limited, per_file_truncated) =
            truncate_utf8(&redacted_text, budget.max_file_bytes);
        let remaining_budget = budget.max_total_bytes.saturating_sub(bundle.total_bytes);
        let (final_limited, total_truncated) = truncate_utf8(per_file_limited, remaining_budget);

        let included_bytes = final_limited.len();
        if included_bytes == 0 {
            bundle.truncated = true;
            break;
        }

        bundle.total_bytes += included_bytes;
        bundle.truncated |= per_file_truncated || total_truncated;
        bundle.files.push(AiContextFile {
            relative_path,
            content: final_limited.to_string(),
            original_bytes,
            included_bytes,
            truncated: per_file_truncated || total_truncated,
            redactions,
        });
    }

    bundle
}

/// Convert a collected context bundle into a system-message payload.
#[must_use]
pub fn render_context_system_message(bundle: &AiContextBundle) -> Option<String> {
    if bundle.files.is_empty() {
        return None;
    }

    let mut message =
        String::from("Workspace context snapshot (allowlisted, local-only, redacted):\n");

    for file in &bundle.files {
        message.push_str(&format!("\n### file: {}\n", file.relative_path.display()));
        message.push_str("```text\n");
        message.push_str(&file.content);
        if !file.content.ends_with('\n') {
            message.push('\n');
        }
        message.push_str("```\n");
    }

    if bundle.truncated {
        message.push_str("\n[context truncated to configured byte budget]\n");
    }

    Some(message)
}

/// Redact likely secret values from text content.
///
/// This operation is deterministic and intentionally conservative:
/// - `key=value` or `key: value` lines containing secret hints are redacted
/// - `Authorization: Bearer <token>` patterns are redacted
#[must_use]
pub fn redact_secrets(input: &str) -> (String, usize) {
    let mut output = String::with_capacity(input.len());
    let mut redactions = 0usize;

    for segment in input.split_inclusive('\n') {
        let (line, newline) = match segment.strip_suffix('\n') {
            Some(without_newline) => (without_newline, "\n"),
            None => (segment, ""),
        };

        let (line_out, redacted) = redact_line(line);
        if redacted {
            redactions += 1;
        }

        output.push_str(&line_out);
        output.push_str(newline);
    }

    if output.is_empty() && !input.is_empty() {
        let (line_out, redacted) = redact_line(input);
        if redacted {
            redactions += 1;
        }
        output.push_str(&line_out);
    }

    (output, redactions)
}

fn redact_line(line: &str) -> (String, bool) {
    if line.is_empty() {
        return (String::new(), false);
    }

    if let Some(redacted) = redact_bearer_token(line) {
        return (redacted, true);
    }

    let lowered = line.to_ascii_lowercase();
    let contains_secret_hint = SECRET_HINTS.iter().any(|hint| lowered.contains(hint));
    if !contains_secret_hint {
        return (line.to_string(), false);
    }

    let delimiter_index = line.find('=').or_else(|| line.find(':'));
    let Some(index) = delimiter_index else {
        return (line.to_string(), false);
    };

    let prefix = &line[..=index];
    (format!("{prefix} {REDACTION_MARKER}"), true)
}

fn redact_bearer_token(line: &str) -> Option<String> {
    let lowered = line.to_ascii_lowercase();
    let needle = "bearer ";
    let start = lowered.find(needle)?;
    let token_start = start + needle.len();
    let suffix = &line[token_start..];
    let token_len = suffix.find(char::is_whitespace).unwrap_or(suffix.len());
    if token_len == 0 {
        return None;
    }

    let token_end = token_start + token_len;
    let mut redacted = String::with_capacity(line.len());
    redacted.push_str(&line[..token_start]);
    redacted.push_str(REDACTION_MARKER);
    redacted.push_str(&line[token_end..]);
    Some(redacted)
}

fn normalize_relative_path(path: &Path) -> Option<PathBuf> {
    if path.is_absolute() {
        return None;
    }

    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::Normal(segment) => normalized.push(segment),
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => return None,
        }
    }

    if normalized.as_os_str().is_empty() {
        None
    } else {
        Some(normalized)
    }
}

fn truncate_utf8(input: &str, max_bytes: usize) -> (&str, bool) {
    if input.len() <= max_bytes {
        return (input, false);
    }

    if max_bytes == 0 {
        return ("", true);
    }

    let mut end = max_bytes.min(input.len());
    while end > 0 && !input.is_char_boundary(end) {
        end -= 1;
    }

    (&input[..end], true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redact_secrets_masks_assignments_and_bearer_tokens() {
        let input = "\
api_key = \"abc123\"
Authorization: Bearer top-secret-token
safe = true
";

        let (redacted, count) = redact_secrets(input);

        assert_eq!(count, 2);
        assert!(redacted.contains("api_key = [REDACTED]"));
        assert!(redacted.contains("Authorization: Bearer [REDACTED]"));
        assert!(redacted.contains("safe = true"));
        assert!(!redacted.contains("abc123"));
        assert!(!redacted.contains("top-secret-token"));
    }

    #[test]
    fn collect_workspace_context_enforces_budget_and_allowlist() {
        let temp = tempfile::tempdir().expect("tempdir should be created");
        let root = temp.path();

        let cargo = root.join("Cargo.toml");
        let readme = root.join("README.md");
        let ignored = root.join("ignored.bin");

        fs::write(&cargo, "token = very-secret-value\nname = \"demo\"\n")
            .expect("cargo file should be written");
        fs::write(
            &readme,
            "NetToolsKit ".repeat(80), // intentionally long for truncation
        )
        .expect("readme file should be written");
        fs::write(&ignored, [0_u8, 1_u8, 2_u8]).expect("ignored file should be written");

        let allowlist = vec![
            PathBuf::from("Cargo.toml"),
            PathBuf::from("../outside.txt"),
            PathBuf::from("README.md"),
            PathBuf::from("ignored.bin"),
        ];

        let bundle = collect_workspace_context(
            root,
            &allowlist,
            AiContextBudget {
                max_files: 4,
                max_file_bytes: 64,
                max_total_bytes: 96,
            },
        );

        assert_eq!(bundle.files.len(), 2);
        assert!(bundle.total_bytes <= 96);
        assert!(bundle.truncated);
        assert!(bundle.files.iter().any(|file| file.redactions > 0));
        assert!(bundle.skipped.iter().any(|skip| skip.contains("outside")));
    }

    #[test]
    fn render_context_system_message_includes_files_and_truncation_note() {
        let bundle = AiContextBundle {
            files: vec![AiContextFile {
                relative_path: PathBuf::from("Cargo.toml"),
                content: "name = \"ntk\"".to_string(),
                original_bytes: 12,
                included_bytes: 12,
                truncated: true,
                redactions: 0,
            }],
            total_bytes: 12,
            truncated: true,
            skipped: Vec::new(),
        };

        let message = render_context_system_message(&bundle).expect("message should be produced");
        assert!(message.contains("Workspace context snapshot"));
        assert!(message.contains("file: Cargo.toml"));
        assert!(message.contains("name = \"ntk\""));
        assert!(message.contains("context truncated"));
    }
}
