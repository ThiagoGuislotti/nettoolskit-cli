//! Manifest and template validation handler
//!
//! This module provides validation capabilities for:
//! - Manifest files (YAML schema, apiVersion, kind, references)
//! - Template files (Handlebars syntax, variables, helpers)

use anyhow::Result;
use owo_colors::OwoColorize;
use std::path::Path;

use crate::core::models::ApplyModeKind;
use crate::parsing::ManifestParser;

/// Validation error details
#[derive(Debug, Clone)]
pub struct ValidationError {
    /// Optional line number where the error was detected.
    pub line: Option<usize>,
    /// Human-readable error description.
    pub message: String,
}

/// Validation result summary
#[derive(Debug, Default)]
pub struct ValidationResult {
    /// Errors found during validation.
    pub errors: Vec<ValidationError>,
    /// Non-fatal warnings found during validation.
    pub warnings: Vec<ValidationError>,
}

impl ValidationResult {
    /// Returns `true` when no errors were found.
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    /// Number of errors collected.
    pub fn error_count(&self) -> usize {
        self.errors.len()
    }

    /// Number of warnings collected.
    pub fn warning_count(&self) -> usize {
        self.warnings.len()
    }

    fn push_error(&mut self, line: Option<usize>, message: impl Into<String>) {
        self.errors.push(ValidationError {
            line,
            message: message.into(),
        });
    }

    fn push_warning(&mut self, line: Option<usize>, message: impl Into<String>) {
        self.warnings.push(ValidationError {
            line,
            message: message.into(),
        });
    }
}

/// Check and validate a manifest or template file.
///
/// When `is_template` is `false` the file is treated as a manifest YAML and
/// goes through full schema + constraint validation.  When `true` only basic
/// Handlebars syntax checks are performed.
pub async fn check_file(path: &Path, is_template: bool) -> Result<ValidationResult> {
    let mut result = ValidationResult::default();

    // ── Existence ──────────────────────────────────────────────────────
    if !path.exists() {
        result.push_error(None, format!("File not found: {}", path.display()));
        return Ok(result);
    }

    if is_template {
        validate_template(path, &mut result);
        return Ok(result);
    }

    // ── Extension hint ─────────────────────────────────────────────────
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        if !matches!(ext, "yaml" | "yml") {
            result.push_warning(
                None,
                format!(
                    "File extension '.{ext}' is not '.yaml' or '.yml' — it may not be a manifest"
                ),
            );
        }
    }

    // ── YAML deserialization ───────────────────────────────────────────
    let manifest = match ManifestParser::from_file(path) {
        Ok(m) => m,
        Err(e) => {
            // Try to extract a line number from serde_yaml errors
            let msg = e.to_string();
            let line = extract_yaml_error_line(&msg);
            result.push_error(line, msg);
            return Ok(result);
        }
    };

    // ── Structural validation (ManifestParser::validate) ───────────────
    if let Err(e) = ManifestParser::validate(&manifest) {
        result.push_error(None, e.to_string());
        // Continue — collect as many issues as possible
    }

    // ── Constraint validation ──────────────────────────────────────────
    validate_constraints(&manifest, &mut result);

    Ok(result)
}

// ── Private helpers ──────────────────────────────────────────────────────

/// Validate manifest constraints beyond basic schema.
fn validate_constraints(
    manifest: &crate::core::models::ManifestDocument,
    result: &mut ValidationResult,
) {
    // ── meta ───────────────────────────────────────────────────────────
    if manifest.meta.name.trim().is_empty() {
        result.push_error(None, "meta.name is required and cannot be empty");
    }

    // ── conventions ────────────────────────────────────────────────────
    if manifest.conventions.namespace_root.trim().is_empty() {
        result.push_error(
            None,
            "conventions.namespaceRoot is required and cannot be empty",
        );
    }

    // ── solution ───────────────────────────────────────────────────────
    if manifest.solution.root.as_os_str().is_empty() {
        result.push_error(None, "solution.root is required and cannot be empty");
    }

    // ── apply mode / section coherence ─────────────────────────────────
    match manifest.apply.mode {
        ApplyModeKind::Artifact => {
            if manifest.apply.artifact.is_none() {
                result.push_error(
                    None,
                    "apply.artifact section is required when apply.mode is 'artifact'",
                );
            }
        }
        ApplyModeKind::Feature => {
            if manifest.apply.feature.is_none() {
                result.push_error(
                    None,
                    "apply.feature section is required when apply.mode is 'feature'",
                );
            }
        }
        ApplyModeKind::Layer => {
            if manifest.apply.layer.is_none() {
                result.push_error(
                    None,
                    "apply.layer section is required when apply.mode is 'layer'",
                );
            }
        }
    }

    // ── contexts ───────────────────────────────────────────────────────
    if manifest.contexts.is_empty() {
        result.push_warning(
            None,
            "No contexts defined — the manifest will not generate any contextual artifacts",
        );
    }

    for (idx, ctx) in manifest.contexts.iter().enumerate() {
        if ctx.name.trim().is_empty() {
            result.push_error(
                None,
                format!("contexts[{idx}].name is required and cannot be empty"),
            );
        }
    }

    // ── templates ──────────────────────────────────────────────────────
    if manifest.templates.mapping.is_empty() {
        result.push_warning(
            None,
            "templates.mapping is empty — no template mappings defined",
        );
    }

    for (idx, m) in manifest.templates.mapping.iter().enumerate() {
        if m.template.trim().is_empty() {
            result.push_error(
                None,
                format!("templates.mapping[{idx}].template is required and cannot be empty"),
            );
        }
        if m.dst.trim().is_empty() {
            result.push_error(
                None,
                format!("templates.mapping[{idx}].dst is required and cannot be empty"),
            );
        }
    }

    // ── render rules ───────────────────────────────────────────────────
    for (idx, rule) in manifest.render.rules.iter().enumerate() {
        if rule.expand.trim().is_empty() {
            result.push_error(
                None,
                format!("render.rules[{idx}].expand is required and cannot be empty"),
            );
        }
    }

    // ── guards warnings ────────────────────────────────────────────────
    if manifest.guards.require_existing_projects && manifest.projects.is_empty() {
        result.push_warning(
            None,
            "guards.requireExistingProjects is true but no projects are defined",
        );
    }
}

/// Basic Handlebars template syntax validation.
fn validate_template(path: &Path, result: &mut ValidationResult) {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            result.push_error(None, format!("Cannot read template file: {e}"));
            return;
        }
    };

    // Check for unbalanced Handlebars delimiters
    let open_count = content.matches("{{").count();
    let close_count = content.matches("}}").count();

    if open_count != close_count {
        result.push_error(
            None,
            format!(
                "Unbalanced Handlebars delimiters: {open_count} opening '{{{{' vs {close_count} closing '}}}}'"
            ),
        );
    }

    // Detect unclosed block helpers (e.g., {{#if}} without {{/if}})
    for (line_num, line) in content.lines().enumerate() {
        let line_1based = line_num + 1;

        // Opening block: {{#helper ...}}
        if let Some(pos) = line.find("{{#") {
            let after = &line[pos + 3..];
            if let Some(name_end) = after.find([' ', '}']) {
                let helper_name = &after[..name_end];
                // Verify there's a corresponding closing tag somewhere in the file
                let closing_tag = format!("{{{{/{helper_name}}}}}");
                if !content.contains(&closing_tag) {
                    result.push_error(
                        Some(line_1based),
                        format!("Unclosed block helper '{{{{#{helper_name}}}}}' — missing '{closing_tag}'"),
                    );
                }
            }
        }
    }

    if result.is_valid() && content.trim().is_empty() {
        result.push_warning(None, "Template file is empty");
    }
}

/// Display validation results to the user.
pub(crate) fn display_validation_result(path: &Path, result: &ValidationResult) {
    println!();

    if result.is_valid() {
        println!(
            "{} {} {}",
            "✅".green(),
            path.display().to_string().cyan().bold(),
            "is valid".green()
        );
        println!("\n{}", "No issues found.".green());
    } else {
        println!(
            "{} {} {}",
            "❌".red(),
            path.display().to_string().cyan().bold(),
            "has errors".red()
        );

        println!("\n{}", "Validation Errors:".red().bold());
        for error in &result.errors {
            if let Some(line) = error.line {
                println!(
                    "  ❌ [Line {}] {}",
                    line.to_string().dimmed(),
                    error.message
                );
            } else {
                println!("  ❌ {}", error.message);
            }
        }

        if !result.warnings.is_empty() {
            println!("\n{}", "Warnings:".yellow().bold());
            for warning in &result.warnings {
                if let Some(line) = warning.line {
                    println!(
                        "  ⚠️ [Line {}] {}",
                        line.to_string().dimmed(),
                        warning.message
                    );
                } else {
                    println!("  ⚠️ {}", warning.message);
                }
            }
        }

        println!(
            "\n{} error(s), {} warning(s)",
            result.error_count().to_string().red().bold(),
            result.warning_count().to_string().yellow()
        );
    }

    println!();
}

/// Try to extract a line number from a serde_yaml error message.
fn extract_yaml_error_line(msg: &str) -> Option<usize> {
    // serde_yaml errors often contain "at line N column M"
    if let Some(idx) = msg.find("at line ") {
        let after = &msg[idx + 8..];
        if let Some(end) = after.find(|c: char| !c.is_ascii_digit()) {
            return after[..end].parse().ok();
        }
    }
    None
}
