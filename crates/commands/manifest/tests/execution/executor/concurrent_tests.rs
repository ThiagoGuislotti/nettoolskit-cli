use crate::execution::test_helpers::{create_temp_dir, create_test_manifest};
use nettoolskit_manifest::{ExecutionConfig, ManifestExecutor};

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
    assert!(result2.is_ok());
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
    assert!(result1.is_ok());

    let result2 = executor.execute(config2).await;
    assert!(result2.is_ok());
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
    assert!(!summary.notes.is_empty());
}