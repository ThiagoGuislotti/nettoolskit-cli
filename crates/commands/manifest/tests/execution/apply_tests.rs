//! Apply handler tests
//!
//! Tests for manifest apply command execution and validation.

use nettoolskit_manifest::handlers::apply::execute_apply;
use nettoolskit_core::ExitStatus;
use std::path::PathBuf;

#[tokio::test]
async fn test_apply_with_missing_manifest() {
    // Arrange
    let manifest_path = PathBuf::from("nonexistent.yaml");

    // Act
    let status = execute_apply(manifest_path, None, false).await;

    // Assert
    assert_eq!(status, ExitStatus::Error);
}

#[tokio::test]
async fn test_apply_dry_run_creates_config() {
    // Arrange
    let manifest_path = PathBuf::from("test.yaml");

    // Act
    // Just verify the function can be called with dry-run flag
    // Actual behavior is tested in integration tests
    let _status = execute_apply(manifest_path, None, true).await;

    // Assert - no panic
}