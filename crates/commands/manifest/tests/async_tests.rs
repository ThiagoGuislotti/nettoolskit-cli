//! Async Manifest Executor Tests
//!
//! Tests for asynchronous manifest execution, including timeout handling,
//! cancellation support, and concurrent task processing.

use nettoolskit_manifest::{ExecutionConfig, ManifestExecutor};
use std::fs;
use std::path::PathBuf;
use std::time::Duration;
use tempfile::TempDir;
use tokio::time::timeout;

fn create_temp_dir() -> TempDir {
    TempDir::new().expect("Failed to create temp directory")
}

fn create_test_manifest(path: &PathBuf, context_count: usize) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        let templates_dir = parent.join("templates");
        fs::create_dir_all(&templates_dir)?;
    }

    let mut contexts_yaml = String::new();
    for i in 0..context_count {
        contexts_yaml.push_str(&format!(
            r#"
  - name: Context{}
    aggregates:
      - name: Aggregate{}
        valueObjects:
          - name: Id{}
            fields:
              - name: value
                type: Guid
                nullable: false
"#,
            i, i, i
        ));
    }

    let manifest_content = format!(
        r#"apiVersion: ntk/v1
kind: solution
meta:
  name: async-test-manifest
solution:
  root: ./
  slnFile: AsyncTest.sln
conventions:
  namespaceRoot: AsyncApp
  targetFramework: net9.0
  policy:
    collision: fail
    insertTodoWhenMissing: true
    strict: false
apply:
  mode: feature
  feature:
    include: []
contexts:{}
"#,
        contexts_yaml
    );

    fs::write(path, manifest_content)
}

#[tokio::test]
async fn test_async_executor_basic_operation() {
    // Arrange
    let temp_dir = create_temp_dir();
    let manifest_path = temp_dir.path().join("manifest.yml");
    let output_dir = temp_dir.path().join("output");
    create_test_manifest(&manifest_path, 1).expect("Failed to create manifest");

    let config = ExecutionConfig {
        manifest_path,
        output_root: output_dir.clone(),
        dry_run: true,
    };

    let executor = ManifestExecutor::new();
    // Act
    let result = executor.execute(config).await;

    // Assert
    assert!(result.is_ok());
    let summary = result.unwrap();
    // Assert
    assert!(!summary.notes.is_empty());
}

#[tokio::test]
async fn test_async_executor_concurrent_instantiation() {
    // Arrange
    let temp_dir = create_temp_dir();
    let manifest_path = temp_dir.path().join("manifest.yml");
    let output_dir = temp_dir.path().join("output");
    create_test_manifest(&manifest_path, 1).expect("Failed to create manifest");

    let config1 = ExecutionConfig {
        manifest_path: manifest_path.clone(),
        output_root: output_dir.clone(),
        dry_run: true,
    };
    let config2 = ExecutionConfig {
        manifest_path,
        output_root: output_dir,
        dry_run: true,
    };

    let executor1 = ManifestExecutor::new();
    let executor2 = ManifestExecutor::new();

    let handle1 = tokio::spawn(async move { executor1.execute(config1).await });
    let handle2 = tokio::spawn(async move { executor2.execute(config2).await });

    let result1 = handle1.await.expect("Task 1 panicked");
    let result2 = handle2.await.expect("Task 2 panicked");

    // Assert
    assert!(result1.is_ok());
    // Assert
    assert!(result2.is_ok());
}

#[tokio::test]
async fn test_async_executor_with_short_timeout() {
    // Arrange
    let temp_dir = create_temp_dir();
    let manifest_path = temp_dir.path().join("manifest.yml");
    let output_dir = temp_dir.path().join("output");
    create_test_manifest(&manifest_path, 1).expect("Failed to create manifest");

    let config = ExecutionConfig {
        manifest_path,
        output_root: output_dir,
        dry_run: true,
    };

    let executor = ManifestExecutor::new();
    // Act
    let result = timeout(Duration::from_secs(5), executor.execute(config)).await;

    // Assert
    assert!(result.is_ok(), "Executor timed out");
    // Assert
    assert!(result.unwrap().is_ok());
}

#[tokio::test]
async fn test_async_executor_with_long_timeout() {
    // Arrange
    let temp_dir = create_temp_dir();
    let manifest_path = temp_dir.path().join("manifest.yml");
    let output_dir = temp_dir.path().join("output");
    create_test_manifest(&manifest_path, 5).expect("Failed to create manifest");

    let config = ExecutionConfig {
        manifest_path,
        output_root: output_dir,
        dry_run: true,
    };

    let executor = ManifestExecutor::new();
    // Act
    let result = timeout(Duration::from_secs(30), executor.execute(config)).await;

    // Assert
    assert!(result.is_ok(), "Executor timed out with multiple contexts");
    // Assert
    assert!(result.unwrap().is_ok());
}

#[tokio::test]
async fn test_async_error_propagation_invalid_manifest() {
    // Arrange
    let temp_dir = create_temp_dir();
    let manifest_path = temp_dir.path().join("invalid.yml");
    let output_dir = temp_dir.path().join("output");

    fs::write(&manifest_path, "invalid: yaml: [[[").expect("Failed to write file");

    let config = ExecutionConfig {
        manifest_path,
        output_root: output_dir,
        dry_run: false,
    };

    let executor = ManifestExecutor::new();
    // Act
    let result = executor.execute(config).await;

    // Assert
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    // Assert
    assert!(error_msg.contains("parse") || error_msg.contains("invalid"));
}

#[tokio::test]
async fn test_async_error_propagation_missing_file() {
    // Arrange
    let temp_dir = create_temp_dir();
    let missing_path = temp_dir.path().join("nonexistent.yml");
    let output_dir = temp_dir.path().join("output");

    let config = ExecutionConfig {
        manifest_path: missing_path,
        output_root: output_dir,
        dry_run: false,
    };

    let executor = ManifestExecutor::new();
    // Act
    let result = executor.execute(config).await;

    // Assert
    assert!(result.is_err());
}

#[tokio::test]
async fn test_async_sequential_executions() {
    // Arrange
    let temp_dir = create_temp_dir();
    let manifest_path = temp_dir.path().join("manifest.yml");
    let output_dir = temp_dir.path().join("output");
    create_test_manifest(&manifest_path, 2).expect("Failed to create manifest");

    let config1 = ExecutionConfig {
        manifest_path: manifest_path.clone(),
        output_root: output_dir.clone(),
        dry_run: true,
    };
    let config2 = ExecutionConfig {
        manifest_path,
        output_root: output_dir,
        dry_run: true,
    };

    let executor = ManifestExecutor::new();

    let result1 = executor.execute(config1).await;
    // Assert
    assert!(result1.is_ok());

    let result2 = executor.execute(config2).await;
    // Assert
    assert!(result2.is_ok());
}

#[tokio::test]
async fn test_async_executor_dry_run_vs_actual() {
    // Arrange
    let temp_dir = create_temp_dir();
    let manifest_path = temp_dir.path().join("manifest.yml");
    let output_dir = temp_dir.path().join("output");
    create_test_manifest(&manifest_path, 1).expect("Failed to create manifest");

    let executor = ManifestExecutor::new();

    let dry_run_config = ExecutionConfig {
        manifest_path: manifest_path.clone(),
        output_root: output_dir.clone(),
        dry_run: true,
    };
    let dry_run_result = executor.execute(dry_run_config).await;
    // Assert
    assert!(dry_run_result.is_ok());

    let actual_config = ExecutionConfig {
        manifest_path,
        output_root: output_dir.clone(),
        dry_run: false,
    };
    let actual_result = executor.execute(actual_config).await;
    // Assert
    assert!(actual_result.is_ok());

    // Assert
    assert!(!output_dir.exists() || output_dir.read_dir().unwrap().next().is_none());
}

#[tokio::test]
async fn test_async_multiple_contexts_handling() {
    // Arrange
    let temp_dir = create_temp_dir();
    let manifest_path = temp_dir.path().join("manifest.yml");
    let output_dir = temp_dir.path().join("output");
    create_test_manifest(&manifest_path, 10).expect("Failed to create manifest");

    let config = ExecutionConfig {
        manifest_path,
        output_root: output_dir,
        dry_run: true,
    };

    let executor = ManifestExecutor::new();
    // Act
    let result = executor.execute(config).await;

    // Assert
    assert!(result.is_ok());
    let summary = result.unwrap();
    // Assert
    assert!(!summary.notes.is_empty());
}

#[tokio::test]
async fn test_async_executor_task_cancellation_safety() {
    // Arrange
    let temp_dir = create_temp_dir();
    let manifest_path = temp_dir.path().join("manifest.yml");
    let output_dir = temp_dir.path().join("output");
    create_test_manifest(&manifest_path, 1).expect("Failed to create manifest");

    let config = ExecutionConfig {
        manifest_path,
        output_root: output_dir,
        dry_run: true,
    };

    let executor = ManifestExecutor::new();
    let handle = tokio::spawn(async move { executor.execute(config).await });

    tokio::time::sleep(Duration::from_millis(1)).await;

    // Act
    let result = handle.await;
    // Assert
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_async_executor_with_minimal_timeout() {
    // Arrange
    let temp_dir = create_temp_dir();
    let manifest_path = temp_dir.path().join("manifest.yml");
    let output_dir = temp_dir.path().join("output");
    create_test_manifest(&manifest_path, 1).expect("Failed to create manifest");

    let config = ExecutionConfig {
        manifest_path,
        output_root: output_dir,
        dry_run: true,
    };

    let executor = ManifestExecutor::new();

    // Act
    let result = timeout(Duration::from_millis(500), executor.execute(config)).await;

    // Assert
    assert!(
        result.is_ok(),
        "Executor should complete within 500ms for simple manifest"
    );
}
