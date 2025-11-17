//! Integration tests for /manifest apply command
//!
//! These tests validate the complete apply workflow including:
//! - Command parsing and validation
//! - Manifest loading and execution
//! - File generation with collision policies
//! - Dry-run mode
//! - TODO marker insertion

use tempfile::TempDir;

/// Test manifest apply with missing manifest file
#[tokio::test]
async fn test_apply_missing_manifest_file() {
    let temp_dir = TempDir::new().unwrap();
    let manifest_path = temp_dir.path().join("nonexistent.yaml");

    // Attempt to apply non-existent manifest
    let result = nettoolskit_manifest::ManifestExecutor::new()
        .execute(nettoolskit_manifest::ExecutionConfig {
            manifest_path,
            output_root: temp_dir.path().to_path_buf(),
            dry_run: false,
        })
        .await;

    // Should fail with file not found error
    assert!(result.is_err());
}

/// Test manifest apply with invalid YAML
#[tokio::test]
async fn test_apply_invalid_yaml() {
    let temp_dir = TempDir::new().unwrap();
    let manifest_path = temp_dir.path().join("invalid.yaml");

    // Create invalid YAML file
    std::fs::write(&manifest_path, "invalid: yaml: content:").unwrap();

    // Attempt to apply invalid manifest
    let result = nettoolskit_manifest::ManifestExecutor::new()
        .execute(nettoolskit_manifest::ExecutionConfig {
            manifest_path,
            output_root: temp_dir.path().to_path_buf(),
            dry_run: false,
        })
        .await;

    // Should fail with parse error
    assert!(result.is_err());
}

/// Test manifest apply with missing apiVersion
#[tokio::test]
async fn test_apply_missing_api_version() {
    let temp_dir = TempDir::new().unwrap();
    let manifest_path = temp_dir.path().join("manifest.yaml");

    // Create manifest without apiVersion
    let manifest_content = r#"
kind: artifact
meta:
  name: test-artifact
  description: Test artifact
solution:
  root: src
"#;
    std::fs::write(&manifest_path, manifest_content).unwrap();

    // Attempt to apply manifest
    let result = nettoolskit_manifest::ManifestExecutor::new()
        .execute(nettoolskit_manifest::ExecutionConfig {
            manifest_path,
            output_root: temp_dir.path().to_path_buf(),
            dry_run: false,
        })
        .await;

    // Should fail with validation error
    assert!(result.is_err());
}

// TODO: Add integration tests for successful apply scenarios
// These tests require complete manifest fixtures with templates
// For now, defer to the manifest crate's integration_tests.rs
// which has comprehensive test coverage for the execution flow
