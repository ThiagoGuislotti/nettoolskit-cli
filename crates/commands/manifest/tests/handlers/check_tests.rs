//! Check handler tests
//!
//! Tests for manifest and template validation functionality.

use nettoolskit_manifest::handlers::check::{check_file, ValidationResult};
use std::path::PathBuf;

#[tokio::test]
async fn test_check_nonexistent_file() {
    let path = PathBuf::from("nonexistent.yaml");
    let result = check_file(&path, false).await;

    assert!(result.is_ok());
    let validation = result.unwrap();
    assert!(!validation.is_valid());
    assert_eq!(validation.error_count(), 1);
    assert!(validation.errors[0].message.contains("File not found"));
}

#[tokio::test]
async fn test_check_template_flag() {
    let path = PathBuf::from("test.yaml");
    let result = check_file(&path, true).await;

    assert!(result.is_ok());
}

#[test]
fn test_validation_result_is_valid() {
    let mut result = ValidationResult::default();
    assert!(result.is_valid());

    result.errors.push(nettoolskit_manifest::handlers::check::ValidationError {
        line: Some(1),
        message: "Error".to_string(),
    });
    assert!(!result.is_valid());
}

#[test]
fn test_validation_result_counts() {
    let mut result = ValidationResult::default();
    assert_eq!(result.error_count(), 0);
    assert_eq!(result.warning_count(), 0);

    result.errors.push(nettoolskit_manifest::handlers::check::ValidationError {
        line: None,
        message: "Error".to_string(),
    });
    result.warnings.push(nettoolskit_manifest::handlers::check::ValidationError {
        line: Some(5),
        message: "Warning".to_string(),
    });

    assert_eq!(result.error_count(), 1);
    assert_eq!(result.warning_count(), 1);
}
