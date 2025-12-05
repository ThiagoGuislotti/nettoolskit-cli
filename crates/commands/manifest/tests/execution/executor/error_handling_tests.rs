use crate::execution::test_helpers::create_temp_dir;
use nettoolskit_manifest::{ExecutionConfig, ManifestExecutor};
use std::fs;

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