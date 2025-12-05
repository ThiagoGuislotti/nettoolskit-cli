//! Tests for multi-context and execution summary tracking
//!
//! Validates handling of multiple bounded contexts and summary data.

use nettoolskit_manifest::{ExecutionConfig, ManifestExecutor};
use std::fs;
use super::test_helpers::{create_minimal_manifest, create_temp_dir};

#[tokio::test]
async fn test_integration_executor_handles_multiple_contexts() {
    // Arrange
    let temp_dir = create_temp_dir();
    let manifest_path = temp_dir.path().join("multi-context.yml");
    let output_dir = temp_dir.path().join("output");

    // Create templates directory
    let templates_dir = temp_dir.path().join("templates");
    fs::create_dir_all(&templates_dir).expect("Failed to create templates dir");

    let manifest_content = r#"apiVersion: ntk/v1
kind: solution
meta:
  name: multi-context-test
solution:
  root: ./
  slnFile: MultiContextSolution.sln
conventions:
  namespaceRoot: MultiApp
  targetFramework: net9.0
  policy:
    collision: fail
    insertTodoWhenMissing: true
    strict: false
apply:
  mode: feature
  feature:
    include: []
contexts:
  - name: Sales
    aggregates:
      - name: Order
        valueObjects:
          - name: OrderId
            fields:
              - name: value
                type: Guid
                nullable: false
                key: false
                columnName: null
  - name: Inventory
    aggregates:
      - name: Product
        valueObjects:
          - name: SKU
            fields:
              - name: code
                type: string
                nullable: false
                key: false
                columnName: null
"#;
    fs::write(&manifest_path, manifest_content).expect("Failed to write manifest");

    let config = ExecutionConfig {
        manifest_path,
        output_root: output_dir,
        dry_run: true,
    };

    // Act
    let executor = ManifestExecutor::new();
    let result = executor.execute(config).await;

    // Assert
    if let Err(ref e) = result {
        eprintln!("Multi-context test error: {}", e);
    }
    assert!(result.is_ok());
    let summary = result.unwrap();
    // Should process both contexts
    assert!(
        !summary.notes.is_empty(),
        "Should generate tasks for multiple contexts"
    );
}

#[tokio::test]
async fn test_integration_executor_tracks_execution_summary() {
    // Arrange
    let temp_dir = create_temp_dir();
    let manifest_path = temp_dir.path().join("manifest.yml");
    let output_dir = temp_dir.path().join("output");
    create_minimal_manifest(&manifest_path, "SummaryApp").expect("Failed to create manifest");

    let config = ExecutionConfig {
        manifest_path,
        output_root: output_dir,
        dry_run: true,
    };

    // Act
    let executor = ManifestExecutor::new();
    let result = executor.execute(config).await;

    // Assert
    assert!(result.is_ok());
    let summary = result.unwrap();

    // Verify summary structure
    assert!(summary.created.is_empty(), "Dry-run creates no files");
    assert!(summary.updated.is_empty(), "Dry-run updates no files");
    assert!(summary.skipped.is_empty(), "Should have no skipped files");
    assert!(!summary.notes.is_empty(), "Should have execution notes");
}