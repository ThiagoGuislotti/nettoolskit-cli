//! Tests for executor error handling
//!
//! Validates proper error responses for missing files, invalid YAML, etc.

use nettoolskit_manifest::{ExecutionConfig, ManifestExecutor};
use std::fs;
use super::test_helpers::create_temp_dir;

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