//! Tests for help command and manifest discovery
//!
//! These tests verify the help command functionality and manifest discovery
//! including file search, metadata parsing, and display formatting.

use nettoolskit_management::handlers::help::{discover_manifests, display_manifests, ManifestInfo};
use std::path::PathBuf;
use tempfile::TempDir;
use tokio::fs;

// ═══════════════════════════════════════════════════════════════
//  Helper Functions
// ═══════════════════════════════════════════════════════════════

/// Create a test manifest file
async fn create_test_manifest(dir: &TempDir, filename: &str, content: &str) -> PathBuf {
    let manifest_path = dir.path().join(filename);
    fs::write(&manifest_path, content)
        .await
        .expect("Failed to write test manifest");
    manifest_path
}

/// Sample manifest YAML for testing
fn sample_manifest_yaml() -> &'static str {
    r#"
apiVersion: ntk/v1
kind: solution
meta:
  name: TestProject
  description: Test manifest for discovery
  author: Test Author

conventions:
  namespaceRoot: TestProject
  targetFramework: net8.0

solution:
  root: ./
  slnFile: TestProject.sln

contexts:
  - name: Orders
    aggregates:
      - name: Order
  - name: Customers
    aggregates:
      - name: Customer

apply:
  mode: feature
  feature:
    contexts:
      - Orders
      - Customers
"#
}

// ═══════════════════════════════════════════════════════════════
//  Test Cases - Discovery
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_discover_manifests_finds_yaml_files() {
    // Arrange
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    create_test_manifest(&temp_dir, "manifest.yaml", sample_manifest_yaml()).await;

    // Act
    let manifests = discover_manifests(Some(temp_dir.path().to_path_buf())).await;

    // Assert
    assert_eq!(
        manifests.len(),
        1,
        "Should find exactly one manifest file"
    );
    assert_eq!(manifests[0].project_name, "TestProject");
    assert_eq!(manifests[0].language, "net8.0");
    assert_eq!(manifests[0].context_count, 2);
}

#[tokio::test]
async fn test_discover_manifests_finds_yml_files() {
    // Arrange
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    create_test_manifest(&temp_dir, "manifest.yml", sample_manifest_yaml()).await;

    // Act
    let manifests = discover_manifests(Some(temp_dir.path().to_path_buf())).await;

    // Assert
    assert_eq!(
        manifests.len(),
        1,
        "Should find manifest with .yml extension"
    );
}

#[tokio::test]
async fn test_discover_manifests_multiple_files() {
    // Arrange
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Create subdirectory with another manifest
    let subdir = temp_dir.path().join("subproject");
    fs::create_dir(&subdir).await.expect("Failed to create subdir");

    create_test_manifest(&temp_dir, "manifest.yaml", sample_manifest_yaml()).await;

    let submanifest_path = subdir.join("manifest.yaml");
    fs::write(&submanifest_path, sample_manifest_yaml())
        .await
        .expect("Failed to write submanifest");

    // Act
    let manifests = discover_manifests(Some(temp_dir.path().to_path_buf())).await;

    // Assert
    assert_eq!(
        manifests.len(),
        2,
        "Should find manifests in root and subdirectory"
    );
}

#[tokio::test]
async fn test_discover_manifests_empty_directory() {
    // Arrange
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Act
    let manifests = discover_manifests(Some(temp_dir.path().to_path_buf())).await;

    // Assert
    assert!(manifests.is_empty(), "Should return empty vector for directory with no manifests");
}

#[tokio::test]
async fn test_discover_manifests_invalid_yaml() {
    // Arrange
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    create_test_manifest(&temp_dir, "manifest.yaml", "invalid: yaml: content: [[[").await;

    // Act
    let manifests = discover_manifests(Some(temp_dir.path().to_path_buf())).await;

    // Assert
    assert!(
        manifests.is_empty(),
        "Should skip manifests with invalid YAML"
    );
}

#[tokio::test]
async fn test_discover_manifests_missing_required_fields() {
    // Arrange
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let incomplete_manifest = r#"
apiVersion: v1
kind: solution
meta:
  name: Incomplete
"#;
    create_test_manifest(&temp_dir, "manifest.yaml", incomplete_manifest).await;

    // Act
    let manifests = discover_manifests(Some(temp_dir.path().to_path_buf())).await;

    // Assert
    assert!(
        manifests.is_empty(),
        "Should skip manifests with missing required fields"
    );
}

// ═══════════════════════════════════════════════════════════════
//  Test Cases - Display
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_display_manifests_empty_list() {
    // Arrange
    let manifests: Vec<ManifestInfo> = vec![];

    // Act & Assert (should not panic)
    display_manifests(&manifests);
}

#[test]
fn test_display_manifests_single_manifest() {
    // Arrange
    let manifests = vec![ManifestInfo {
        path: PathBuf::from("./test/manifest.yaml"),
        project_name: "TestProject".to_string(),
        language: ".NET".to_string(),
        context_count: 3,
    }];

    // Act & Assert (should not panic)
    display_manifests(&manifests);
}

#[test]
fn test_display_manifests_multiple_manifests() {
    // Arrange
    let manifests = vec![
        ManifestInfo {
            path: PathBuf::from("./project1/manifest.yaml"),
            project_name: "Project1".to_string(),
            language: ".NET".to_string(),
            context_count: 2,
        },
        ManifestInfo {
            path: PathBuf::from("./project2/manifest.yaml"),
            project_name: "Project2".to_string(),
            language: "Java".to_string(),
            context_count: 5,
        },
    ];

    // Act & Assert (should not panic)
    display_manifests(&manifests);
}

// ═══════════════════════════════════════════════════════════════
//  Test Cases - Metadata Extraction
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_metadata_extraction_dotnet_project() {
    // Arrange
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    create_test_manifest(&temp_dir, "manifest.yaml", sample_manifest_yaml()).await;

    // Act
    let manifests = discover_manifests(Some(temp_dir.path().to_path_buf())).await;

    // Assert
    assert_eq!(manifests[0].project_name, "TestProject");
    assert_eq!(manifests[0].language, "net8.0");
    assert_eq!(manifests[0].context_count, 2);
}

#[tokio::test]
async fn test_metadata_extraction_context_count() {
    // Arrange
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let manifest_with_contexts = r#"
apiVersion: ntk/v1
kind: solution
meta:
  name: MultiContext
conventions:
  namespaceRoot: Test
  targetFramework: net8.0
solution:
  root: ./
  slnFile: Test.sln
contexts:
  - name: Context1
  - name: Context2
  - name: Context3
  - name: Context4
apply:
  mode: feature
  feature:
    contexts:
      - Context1
      - Context2
      - Context3
      - Context4
"#;
    create_test_manifest(&temp_dir, "manifest.yaml", manifest_with_contexts).await;

    // Act
    let manifests = discover_manifests(Some(temp_dir.path().to_path_buf())).await;

    // Assert
    assert_eq!(
        manifests[0].context_count, 4,
        "Should correctly count number of contexts"
    );
}

#[tokio::test]
async fn test_metadata_extraction_no_contexts() {
    // Arrange
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let manifest_no_contexts = r#"
apiVersion: ntk/v1
kind: solution
meta:
  name: NoContexts
conventions:
  namespaceRoot: Test
  targetFramework: net8.0
solution:
  root: ./
  slnFile: Test.sln
contexts: []
apply:
  mode: artifact
  artifact:
    kind: Entity
    name: TestEntity
"#;
    create_test_manifest(&temp_dir, "manifest.yaml", manifest_no_contexts).await;

    // Act
    let manifests = discover_manifests(Some(temp_dir.path().to_path_buf())).await;

    // Assert
    assert_eq!(manifests.len(), 1, "Should find the manifest file");
    assert_eq!(
        manifests[0].context_count, 0,
        "Should handle manifests with zero contexts"
    );
}
