//! Tests for ManifestExecutor dry-run mode
//!
//! Validates that dry-run executes without creating/modifying files.

use nettoolskit_manifest::{ExecutionConfig, ManifestExecutor};
use super::test_helpers::{create_minimal_manifest, create_temp_dir};

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
    assert!(
        summary.created.is_empty(),
        "Dry-run should not create files"
    );
    assert!(
        summary.updated.is_empty(),
        "Dry-run should not update files"
    );
    assert!(
        !summary.notes.is_empty(),
        "Dry-run should have notes about what would be created"
    );
}