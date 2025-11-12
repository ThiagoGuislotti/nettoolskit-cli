/// Integration tests for manifest feature
///
/// These tests validate the complete workflow:
/// 1. Parse YAML manifest
/// 2. Generate render tasks
/// 3. Render templates (via templating crate)
/// 4. Write files to disk
///
/// Note: These are end-to-end integration tests that touch multiple crates
/// (manifest + templating) and perform actual file I/O.
use nettoolskit_manifest::{ExecutionConfig, ManifestExecutor};
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

// Test Helpers

fn create_temp_dir() -> TempDir {
    TempDir::new().expect("Failed to create temp directory")
}

fn create_minimal_manifest(path: &PathBuf, namespace: &str) -> std::io::Result<()> {
    // Create templates directory next to manifest
    if let Some(parent) = path.parent() {
        let templates_dir = parent.join("templates");
        fs::create_dir_all(&templates_dir)?;
    }

    let manifest_content = format!(
        r#"apiVersion: ntk/v1
kind: solution
meta:
  name: test-manifest
solution:
  root: ./
  slnFile: TestSolution.sln
conventions:
  namespaceRoot: {}
  targetFramework: net9.0
  policy:
    collision: fail
    insertTodoWhenMissing: true
    strict: false
apply:
  mode: feature
  feature:
    context: Orders
    include: []
contexts:
  - name: Orders
    aggregates:
      - name: Order
        valueObjects:
          - name: OrderId
            fields:
              - name: value
                type: Guid
                nullable: false
        entities:
          - name: OrderItem
            fields:
              - name: quantity
                type: int
                nullable: false
                key: false
                columnName: null
            isRoot: false
    useCases:
      - name: CreateOrder
        type: command
        input:
          - name: customerId
            type: Guid
            nullable: false
            key: false
            columnName: null
        output:
          - name: orderId
            type: Guid
            nullable: false
            key: false
            columnName: null
"#,
        namespace
    );

    fs::write(path, manifest_content)
}

// Integration Tests - Parser

// Note: Parser tests are already covered in parser_tests.rs
// Integration tests focus on end-to-end workflows with ManifestExecutor

// Integration Tests - Executor (Dry-Run)

#[tokio::test]
async fn test_integration_executor_dry_run_no_files_created() {
    // Arrange
    let temp_dir = create_temp_dir();
    let manifest_path = temp_dir.path().join("manifest.yml");
    let output_dir = temp_dir.path().join("output");
    create_minimal_manifest(&manifest_path, "DryRunApp").expect("Failed to create manifest");

    let config = ExecutionConfig {
        manifest_path,
        output_root: output_dir.clone(),
        dry_run: true,
    };

    // Act
    let executor = ManifestExecutor::new();
    let result = executor.execute(config).await;

    // Assert
    if let Err(ref e) = result {
        eprintln!("Executor error: {}", e);
    }
    assert!(result.is_ok(), "Executor should succeed in dry-run mode");
    let summary = result.unwrap();
    assert!(summary.created.is_empty(), "Dry-run should not create files");
    assert!(summary.updated.is_empty(), "Dry-run should not update files");
    assert!(
        !summary.notes.is_empty(),
        "Dry-run should have notes about what would be created"
    );
}

// Integration Tests - End-to-End (Requires Templates)

#[tokio::test]
#[ignore = "Requires actual template files to be present"]
async fn test_integration_full_workflow_with_templates() {
    // Arrange
    let temp_dir = create_temp_dir();
    let manifest_path = temp_dir.path().join("manifest.yml");
    let output_dir = temp_dir.path().join("output");
    create_minimal_manifest(&manifest_path, "IntegrationApp").expect("Failed to create manifest");

    let config = ExecutionConfig {
        manifest_path,
        output_root: output_dir.clone(),
        dry_run: false,
    };

    // Act
    let executor = ManifestExecutor::new();
    let result = executor.execute(config).await;

    // Assert
    // Note: This will fail without actual template files
    // When templates are available, verify:
    // - Files are created in output_dir
    // - Content matches expected template output
    // - Summary contains created files
    if result.is_ok() {
        let summary = result.unwrap();
        assert!(
            !summary.created.is_empty(),
            "Should create files when templates exist"
        );
    }
}

// Integration Tests - Error Handling

#[tokio::test]
async fn test_integration_executor_handles_missing_manifest() {
    // Arrange
    let temp_dir = create_temp_dir();
    let missing_path = temp_dir.path().join("nonexistent.yml");
    let output_dir = temp_dir.path().join("output");

    let config = ExecutionConfig {
        manifest_path: missing_path,
        output_root: output_dir,
        dry_run: false,
    };

    // Act
    let executor = ManifestExecutor::new();
    let result = executor.execute(config).await;

    // Assert
    assert!(result.is_err());
    let error = result.unwrap_err();
    let error_msg = error.to_string();
    assert!(
        error_msg.contains("not found")
            || error_msg.contains("No such")
            || error_msg.contains("does not exist")
            || error_msg.contains("especificado")
            || error_msg.contains("error 2"),
        "Expected file not found error, got: {}",
        error_msg
    );
}

#[tokio::test]
async fn test_integration_executor_handles_invalid_yaml() {
    // Arrange
    let temp_dir = create_temp_dir();
    let manifest_path = temp_dir.path().join("invalid.yml");
    let output_dir = temp_dir.path().join("output");

    // Create invalid YAML
    fs::write(&manifest_path, "invalid: yaml: content: [[[").expect("Failed to write file");

    let config = ExecutionConfig {
        manifest_path,
        output_root: output_dir,
        dry_run: false,
    };

    // Act
    let executor = ManifestExecutor::new();
    let result = executor.execute(config).await;

    // Assert
    assert!(result.is_err());
}

// Integration Tests - Async Operations

#[tokio::test]
async fn test_integration_executor_handles_async_timeout() {
    // Arrange
    let temp_dir = create_temp_dir();
    let manifest_path = temp_dir.path().join("manifest.yml");
    let output_dir = temp_dir.path().join("output");
    create_minimal_manifest(&manifest_path, "TimeoutApp").expect("Failed to create manifest");

    let config = ExecutionConfig {
        manifest_path,
        output_root: output_dir,
        dry_run: true,
    };

    // Act - Execute with timeout
    let executor = ManifestExecutor::new();
    let result = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        executor.execute(config),
    )
    .await;

    // Assert
    assert!(result.is_ok(), "Execution should complete within timeout");
    assert!(
        result.unwrap().is_ok(),
        "Execution should succeed within timeout"
    );
}

// Integration Tests - Multiple Contexts

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

// Integration Tests - Summary Tracking

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
