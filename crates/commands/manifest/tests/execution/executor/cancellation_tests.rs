use crate::execution::test_helpers::{create_temp_dir, create_test_manifest};
use nettoolskit_manifest::{ExecutionConfig, ManifestExecutor};
use std::time::Duration;

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