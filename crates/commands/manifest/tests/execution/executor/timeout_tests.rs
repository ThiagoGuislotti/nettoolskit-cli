use crate::execution::test_helpers::{create_temp_dir, create_test_manifest};
use nettoolskit_manifest::{ExecutionConfig, ManifestExecutor};
use std::time::Duration;
use tokio::time::timeout;

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
    assert!(result.unwrap().is_ok());
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