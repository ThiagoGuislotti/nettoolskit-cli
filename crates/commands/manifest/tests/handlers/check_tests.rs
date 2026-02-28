//! Check handler tests
//!
//! Tests for manifest and template validation functionality.
//! Category: Unit

use nettoolskit_manifest::handlers::check::{check_file, ValidationError, ValidationResult};
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

// ── Helpers ────────────────────────────────────────────────────────────────

fn write_temp_file(dir: &TempDir, name: &str, content: &str) -> PathBuf {
    let path = dir.path().join(name);
    fs::write(&path, content).unwrap();
    path
}

/// Minimal valid manifest YAML that passes all checks.
fn valid_manifest_yaml() -> &'static str {
    r#"apiVersion: ntk/v1
kind: solution
meta:
  name: test-manifest
solution:
  root: ./
  slnFile: TestSolution.sln
conventions:
  namespaceRoot: Acme.Test
  targetFramework: net9.0
  policy:
    collision: fail
    insertTodoWhenMissing: true
    strict: false
apply:
  mode: artifact
  artifact:
    kind: ValueObject
contexts:
  - name: Sales
    aggregates: []
    useCases: []
templates:
  mapping:
    - artifact: ValueObject
      template: value-object.hbs
      dst: "{{project}}/ValueObjects/{{Name}}.cs"
render:
  rules:
    - expand: "{{Context}}"
      as: ctx
guards:
  requireExistingProjects: false
"#
}

// ── ValidationResult unit tests ────────────────────────────────────────────

#[test]
fn test_validation_result_default_is_valid() {
    let result = ValidationResult::default();
    assert!(result.is_valid());
    assert_eq!(result.error_count(), 0);
    assert_eq!(result.warning_count(), 0);
}

#[test]
fn test_validation_result_invalid_with_error() {
    let mut result = ValidationResult::default();
    result.errors.push(ValidationError {
        line: Some(1),
        message: "Error".to_string(),
    });
    assert!(!result.is_valid());
    assert_eq!(result.error_count(), 1);
}

#[test]
fn test_validation_result_counts() {
    let mut result = ValidationResult::default();
    result.errors.push(ValidationError {
        line: None,
        message: "Error".to_string(),
    });
    result.warnings.push(ValidationError {
        line: Some(5),
        message: "Warning".to_string(),
    });
    assert_eq!(result.error_count(), 1);
    assert_eq!(result.warning_count(), 1);
    assert!(!result.is_valid());
}

#[test]
fn test_validation_result_valid_with_only_warnings() {
    let mut result = ValidationResult::default();
    result.warnings.push(ValidationError {
        line: None,
        message: "Just a warning".to_string(),
    });
    // Warnings alone do not make the result invalid
    assert!(result.is_valid());
    assert_eq!(result.warning_count(), 1);
}

// ── File existence tests ───────────────────────────────────────────────────

#[tokio::test]
async fn test_check_nonexistent_file() {
    let path = PathBuf::from("nonexistent_manifest.yaml");
    let result = check_file(&path, false).await;

    assert!(result.is_ok());
    let validation = result.unwrap();
    assert!(!validation.is_valid());
    assert_eq!(validation.error_count(), 1);
    assert!(validation.errors[0].message.contains("File not found"));
}

#[tokio::test]
async fn test_check_nonexistent_template() {
    let path = PathBuf::from("nonexistent_template.hbs");
    let result = check_file(&path, true).await;

    assert!(result.is_ok());
    let validation = result.unwrap();
    assert!(!validation.is_valid());
    assert!(validation.errors[0].message.contains("File not found"));
}

// ── Valid manifest ─────────────────────────────────────────────────────────

#[tokio::test]
async fn test_check_valid_manifest() {
    let dir = TempDir::new().unwrap();
    let path = write_temp_file(&dir, "manifest.yaml", valid_manifest_yaml());

    let result = check_file(&path, false).await.unwrap();
    assert!(result.is_valid(), "Expected no errors: {:?}", result.errors);
}

// ── Extension warning ──────────────────────────────────────────────────────

#[tokio::test]
async fn test_check_warns_on_non_yaml_extension() {
    let dir = TempDir::new().unwrap();
    let path = write_temp_file(&dir, "manifest.json", valid_manifest_yaml());

    let result = check_file(&path, false).await.unwrap();
    // The YAML is valid so no errors, but extension should trigger a warning
    assert!(result.is_valid());
    assert!(
        result.warnings.iter().any(|w| w.message.contains(".json")),
        "Expected extension warning, got: {:?}",
        result.warnings
    );
}

// ── Invalid YAML syntax ───────────────────────────────────────────────────

#[tokio::test]
async fn test_check_invalid_yaml_syntax() {
    let dir = TempDir::new().unwrap();
    let path = write_temp_file(&dir, "bad.yaml", "apiVersion: ntk/v1\n  bad indent: here\n");

    let result = check_file(&path, false).await.unwrap();
    assert!(!result.is_valid());
    assert_eq!(result.error_count(), 1);
}

// ── Missing required meta.name ─────────────────────────────────────────────

#[tokio::test]
async fn test_check_empty_meta_name() {
    let yaml = r#"apiVersion: ntk/v1
kind: solution
meta:
  name: ""
solution:
  root: ./
  slnFile: Test.sln
conventions:
  namespaceRoot: Acme
  targetFramework: net9.0
  policy:
    collision: fail
    insertTodoWhenMissing: false
    strict: false
apply:
  mode: artifact
  artifact:
    kind: Entity
"#;
    let dir = TempDir::new().unwrap();
    let path = write_temp_file(&dir, "m.yaml", yaml);

    let result = check_file(&path, false).await.unwrap();
    assert!(!result.is_valid());
    assert!(
        result
            .errors
            .iter()
            .any(|e| e.message.contains("meta.name")),
        "errors: {:?}",
        result.errors
    );
}

// ── Empty conventions.namespaceRoot ────────────────────────────────────────

#[tokio::test]
async fn test_check_empty_namespace_root() {
    let yaml = r#"apiVersion: ntk/v1
kind: solution
meta:
  name: test
solution:
  root: ./
  slnFile: Test.sln
conventions:
  namespaceRoot: ""
  targetFramework: net9.0
  policy:
    collision: fail
    insertTodoWhenMissing: false
    strict: false
apply:
  mode: artifact
  artifact:
    kind: Entity
"#;
    let dir = TempDir::new().unwrap();
    let path = write_temp_file(&dir, "m.yaml", yaml);

    let result = check_file(&path, false).await.unwrap();
    assert!(!result.is_valid());
    assert!(
        result
            .errors
            .iter()
            .any(|e| e.message.contains("namespaceRoot")),
        "errors: {:?}",
        result.errors
    );
}

// ── Apply mode / section mismatch ──────────────────────────────────────────

#[tokio::test]
async fn test_check_artifact_mode_missing_artifact_section() {
    let yaml = r#"apiVersion: ntk/v1
kind: solution
meta:
  name: test
solution:
  root: ./
  slnFile: Test.sln
conventions:
  namespaceRoot: Acme
  targetFramework: net9.0
  policy:
    collision: fail
    insertTodoWhenMissing: false
    strict: false
apply:
  mode: artifact
"#;
    let dir = TempDir::new().unwrap();
    let path = write_temp_file(&dir, "m.yaml", yaml);

    let result = check_file(&path, false).await.unwrap();
    assert!(!result.is_valid());
    assert!(
        result
            .errors
            .iter()
            .any(|e| e.message.contains("apply.artifact")),
        "errors: {:?}",
        result.errors
    );
}

#[tokio::test]
async fn test_check_feature_mode_missing_feature_section() {
    let yaml = r#"apiVersion: ntk/v1
kind: solution
meta:
  name: test
solution:
  root: ./
  slnFile: Test.sln
conventions:
  namespaceRoot: Acme
  targetFramework: net9.0
  policy:
    collision: fail
    insertTodoWhenMissing: false
    strict: false
apply:
  mode: feature
"#;
    let dir = TempDir::new().unwrap();
    let path = write_temp_file(&dir, "m.yaml", yaml);

    let result = check_file(&path, false).await.unwrap();
    assert!(!result.is_valid());
    assert!(
        result
            .errors
            .iter()
            .any(|e| e.message.contains("apply.feature")),
        "errors: {:?}",
        result.errors
    );
}

// ── No contexts warning ───────────────────────────────────────────────────

#[tokio::test]
async fn test_check_warns_no_contexts() {
    let yaml = r#"apiVersion: ntk/v1
kind: solution
meta:
  name: test
solution:
  root: ./
  slnFile: Test.sln
conventions:
  namespaceRoot: Acme
  targetFramework: net9.0
  policy:
    collision: fail
    insertTodoWhenMissing: false
    strict: false
apply:
  mode: artifact
  artifact:
    kind: Entity
"#;
    let dir = TempDir::new().unwrap();
    let path = write_temp_file(&dir, "m.yaml", yaml);

    let result = check_file(&path, false).await.unwrap();
    assert!(
        result
            .warnings
            .iter()
            .any(|w| w.message.contains("No contexts")),
        "warnings: {:?}",
        result.warnings
    );
}

// ── Template validation ────────────────────────────────────────────────────

#[tokio::test]
async fn test_check_valid_template() {
    let dir = TempDir::new().unwrap();
    let path = write_temp_file(
        &dir,
        "entity.hbs",
        "namespace {{Namespace}};\npublic class {{Name}} { }",
    );

    let result = check_file(&path, true).await.unwrap();
    assert!(result.is_valid(), "errors: {:?}", result.errors);
}

#[tokio::test]
async fn test_check_template_unbalanced_delimiters() {
    let dir = TempDir::new().unwrap();
    let path = write_temp_file(&dir, "bad.hbs", "Hello {{name} world");

    let result = check_file(&path, true).await.unwrap();
    assert!(!result.is_valid());
    assert!(
        result
            .errors
            .iter()
            .any(|e| e.message.contains("Unbalanced")),
        "errors: {:?}",
        result.errors
    );
}

#[tokio::test]
async fn test_check_template_unclosed_block_helper() {
    let dir = TempDir::new().unwrap();
    let path = write_temp_file(&dir, "block.hbs", "{{#if hasId}}\n  public Guid Id;\n");

    let result = check_file(&path, true).await.unwrap();
    assert!(!result.is_valid());
    assert!(
        result
            .errors
            .iter()
            .any(|e| e.message.contains("Unclosed block helper")),
        "errors: {:?}",
        result.errors
    );
}

#[tokio::test]
async fn test_check_template_valid_block_helper() {
    let dir = TempDir::new().unwrap();
    let path = write_temp_file(&dir, "ok.hbs", "{{#if hasId}}\n  public Guid Id;\n{{/if}}");

    let result = check_file(&path, true).await.unwrap();
    assert!(result.is_valid(), "errors: {:?}", result.errors);
}

#[tokio::test]
async fn test_check_template_empty_warns() {
    let dir = TempDir::new().unwrap();
    let path = write_temp_file(&dir, "empty.hbs", "   ");

    let result = check_file(&path, true).await.unwrap();
    assert!(result.is_valid()); // Empty template is not an error
    assert!(
        result.warnings.iter().any(|w| w.message.contains("empty")),
        "warnings: {:?}",
        result.warnings
    );
}
