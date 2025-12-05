//! Tests for async operation behavior
//!
//! Validates timeout handling and async execution correctness.

use nettoolskit_manifest::{ExecutionConfig, ManifestExecutor};
use super::test_helpers::{create_minimal_manifest, create_temp_dir};

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
    let result =
        tokio::time::timeout(std::time::Duration::from_secs(5), executor.execute(config)).await;

    // Assert
    assert!(result.is_ok(), "Execution should complete within timeout");
    assert!(
        result.unwrap().is_ok(),
        "Execution should succeed within timeout"
    );
}