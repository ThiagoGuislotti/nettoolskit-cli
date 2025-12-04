//! ManifestInfo Model Tests
//!
//! Tests for ManifestInfo struct and its properties.

use nettoolskit_help::ManifestInfo;
use std::path::PathBuf;

// Constructor and Field Tests

#[test]
fn test_manifest_info_construction() {
    // Arrange
    let path = PathBuf::from("/test/path.yaml");
    let project_name = String::from("TestProject");
    let language = String::from("dotnet");
    let context_count = 3;

    // Act
    let info = ManifestInfo {
        path: path.clone(),
        project_name: project_name.clone(),
        language: language.clone(),
        context_count,
    };

    // Assert
    assert_eq!(info.path, path);
    assert_eq!(info.project_name, project_name);
    assert_eq!(info.language, language);
    assert_eq!(info.context_count, context_count);
}

#[test]
fn test_manifest_info_fields() {
    // Arrange & Act
    let info = ManifestInfo {
        path: PathBuf::from("/test/path.yaml"),
        project_name: String::from("TestProject"),
        language: String::from("dotnet"),
        context_count: 3,
    };

    // Assert
    assert_eq!(info.project_name, "TestProject");
    assert_eq!(info.language, "dotnet");
    assert_eq!(info.context_count, 3);
}

// Edge Cases

#[test]
fn test_manifest_info_zero_contexts() {
    // Arrange & Act
    let info = ManifestInfo {
        path: PathBuf::from("./empty.manifest.yaml"),
        project_name: "EmptyProject".to_string(),
        language: "rust".to_string(),
        context_count: 0,
    };

    // Assert
    assert_eq!(info.context_count, 0);
}

#[test]
fn test_manifest_info_many_contexts() {
    // Arrange & Act
    let info = ManifestInfo {
        path: PathBuf::from("./large.manifest.yaml"),
        project_name: "LargeProject".to_string(),
        language: "java".to_string(),
        context_count: 100,
    };

    // Assert
    assert_eq!(info.context_count, 100);
}

// Trait Tests

#[test]
fn test_manifest_info_debug() {
    // Arrange
    let info = ManifestInfo {
        path: PathBuf::from("./test.yaml"),
        project_name: "Test".to_string(),
        language: "rust".to_string(),
        context_count: 1,
    };

    // Act
    let debug_str = format!("{:?}", info);

    // Assert
    assert!(debug_str.contains("ManifestInfo"));
    assert!(debug_str.contains("Test"));
}

#[test]
fn test_manifest_info_clone() {
    // Arrange
    let original = ManifestInfo {
        path: PathBuf::from("./original.yaml"),
        project_name: "Original".to_string(),
        language: "dotnet".to_string(),
        context_count: 5,
    };

    // Act
    let cloned = original.clone();

    // Assert
    assert_eq!(cloned.project_name, original.project_name);
    assert_eq!(cloned.language, original.language);
    assert_eq!(cloned.context_count, original.context_count);
}
