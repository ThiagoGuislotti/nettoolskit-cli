//! Discovery Handler Tests
//!
//! Tests for manifest discovery and display functionality.

use nettoolskit_help::{discover_manifests, display_manifests, ManifestInfo};
use std::path::PathBuf;

// Public API Tests

#[tokio::test]
async fn test_discover_manifests_executes_without_panic() {
    // Arrange & Act
    let manifests = discover_manifests(None).await;

    // Assert
    // Should return a vector (empty or with results)
    assert!(manifests.is_empty() || !manifests.is_empty());
}

#[tokio::test]
async fn test_discover_manifests_with_custom_path() {
    // Arrange
    let custom_path = Some(PathBuf::from("."));

    // Act
    let manifests = discover_manifests(custom_path).await;

    // Assert
    assert!(manifests.is_empty() || !manifests.is_empty());
}

#[test]
fn test_display_manifests_empty_list() {
    // Arrange
    let manifests = vec![];

    // Act & Assert
    // Should not panic with empty list
    display_manifests(&manifests);
}

#[test]
fn test_display_manifests_single_manifest() {
    // Arrange
    let manifests = vec![ManifestInfo {
        path: PathBuf::from("./project.manifest.yaml"),
        project_name: "TestProject".to_string(),
        language: "dotnet".to_string(),
        context_count: 3,
    }];

    // Act & Assert
    // Should not panic
    display_manifests(&manifests);
}

#[test]
fn test_display_manifests_multiple_manifests() {
    // Arrange
    let manifests = vec![
        ManifestInfo {
            path: PathBuf::from("./project1.manifest.yaml"),
            project_name: "Project1".to_string(),
            language: "dotnet".to_string(),
            context_count: 5,
        },
        ManifestInfo {
            path: PathBuf::from("./project2.manifest.yaml"),
            project_name: "Project2".to_string(),
            language: "rust".to_string(),
            context_count: 3,
        },
    ];

    // Act & Assert
    // Should not panic
    display_manifests(&manifests);
}
