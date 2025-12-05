use crate::execution::test_helpers::{create_temp_dir, create_test_manifest};
use nettoolskit_manifest::{ExecutionConfig, ManifestExecutor};

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
    assert!(!summary.notes.is_empty());
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
    assert!(dry_run_result.is_ok());

    let actual_config = ExecutionConfig {
        manifest_path,
        output_root: output_dir.clone(),
        dry_run: false,
    };
    let actual_result = executor.execute(actual_config).await;
    assert!(actual_result.is_ok());

    assert!(!output_dir.exists() || output_dir.read_dir().unwrap().next().is_none());
}