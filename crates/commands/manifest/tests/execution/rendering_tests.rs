//! Rendering module tests
//!
//! Tests for template rendering utilities in the execution module.

use nettoolskit_manifest::execution::rendering::*;
use serde_json::json;
use std::path::PathBuf;
use tempfile::TempDir;
use tokio::fs;

#[tokio::test]
async fn test_render_template_with_valid_data() {
    // Create temp directory with a test template
    let temp_dir = TempDir::new().unwrap();
    let templates_dir = temp_dir.path().join("templates");
    fs::create_dir_all(&templates_dir).await.unwrap();

    let template_path = templates_dir.join("test.hbs");
    fs::write(&template_path, "Hello {{name}}!").await.unwrap();

    // Test rendering
    let data = json!({ "name": "World" });
    let result = render_template(&templates_dir, "test.hbs", &data, false).await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "Hello World!");
}

#[tokio::test]
async fn test_render_template_not_found() {
    let temp_dir = TempDir::new().unwrap();
    let templates_dir = temp_dir.path().join("templates");
    fs::create_dir_all(&templates_dir).await.unwrap();

    let data = json!({});
    let result = render_template(&templates_dir, "nonexistent.hbs", &data, false).await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        matches!(err, nettoolskit_manifest::core::error::ManifestError::TemplateNotFound { .. }),
        "Expected TemplateNotFound error"
    );
}

#[tokio::test]
async fn test_render_template_with_todo_insertion() {
    let temp_dir = TempDir::new().unwrap();
    let templates_dir = temp_dir.path().join("templates");
    fs::create_dir_all(&templates_dir).await.unwrap();

    let template_path = templates_dir.join("todo.hbs");
    fs::write(&template_path, "Function stub").await.unwrap();

    let data = json!({});
    let result = render_template(&templates_dir, "todo.hbs", &data, true).await;

    assert!(result.is_ok());
    let rendered = result.unwrap();
    assert!(rendered.contains("Function stub"));
}

#[test]
fn test_build_solution_stub() {
    let stub = build_solution_stub("TestSolution");

    assert!(stub.contains("Microsoft Visual Studio Solution File"));
    assert!(stub.contains("Format Version 12.00"));
    assert!(stub.contains("Global"));
    assert!(stub.contains("EndGlobal"));
}

#[test]
fn test_build_project_stub() {
    let stub = build_project_stub("TestProject", "net8.0", "Test Author");

    assert!(stub.contains("<Project Sdk=\"Microsoft.NET.Sdk\">"));
    assert!(stub.contains("<TargetFramework>net8.0</TargetFramework>"));
    assert!(stub.contains("<Authors>Test Author</Authors>"));
    assert!(stub.contains("<Product>TestProject</Product>"));
}

#[test]
fn test_build_project_stub_with_different_framework() {
    let stub = build_project_stub("AnotherProject", "net9.0", "Another Author");

    assert!(stub.contains("<TargetFramework>net9.0</TargetFramework>"));
    assert!(stub.contains("<Authors>Another Author</Authors>"));
    assert!(stub.contains("<Product>AnotherProject</Product>"));
}

#[test]
fn test_normalize_line_endings_crlf() {
    let input = "Line 1\r\nLine 2\r\nLine 3";
    let normalized = normalize_line_endings(input);

    assert_eq!(normalized, "Line 1\nLine 2\nLine 3");
    assert!(!normalized.contains("\r\n"));
}

#[test]
fn test_normalize_line_endings_lf_unchanged() {
    let input = "Line 1\nLine 2\nLine 3";
    let normalized = normalize_line_endings(input);

    assert_eq!(normalized, input);
}

#[test]
fn test_normalize_line_endings_empty() {
    let input = "";
    let normalized = normalize_line_endings(input);

    assert_eq!(normalized, "");
}

#[test]
fn test_normalize_line_endings_mixed() {
    let input = "Line 1\r\nLine 2\nLine 3\r\nLine 4";
    let normalized = normalize_line_endings(input);

    assert_eq!(normalized, "Line 1\nLine 2\nLine 3\nLine 4");
}