//! Manifest and template validation handler
//!
//! This module provides validation capabilities for:
//! - Manifest files (YAML schema, apiVersion, kind, references)
//! - Template files (Handlebars syntax, variables, helpers)
//!
//! **Status**: Placeholder implementation - to be completed in Phase 2.4

use anyhow::Result;
use owo_colors::OwoColorize;
use std::path::Path;

/// Validation error details
#[derive(Debug, Clone)]
pub struct ValidationError {
    pub line: Option<usize>,
    pub message: String,
}

/// Validation result summary
#[derive(Debug, Default)]
pub struct ValidationResult {
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationError>,
}

impl ValidationResult {
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn error_count(&self) -> usize {
        self.errors.len()
    }

    pub fn warning_count(&self) -> usize {
        self.warnings.len()
    }
}

/// Check and validate a manifest or template file
/// **TODO**: Full implementation pending
pub async fn check_file(path: &Path, _is_template: bool) -> Result<ValidationResult> {
    let mut result = ValidationResult::default();

    if !path.exists() {
        result.errors.push(ValidationError {
            line: None,
            message: format!("File not found: {}", path.display()),
        });
    }

    Ok(result)
}

/// Display validation results to the user
pub fn display_validation_result(path: &Path, result: &ValidationResult) {
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
