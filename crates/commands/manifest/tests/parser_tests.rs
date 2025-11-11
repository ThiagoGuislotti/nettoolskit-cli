/// Tests for ManifestParser
use nettoolskit_manifest::{ManifestParser};
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

// Helper to create a temp file with manifest content
fn create_temp_manifest(content: &str) -> (TempDir, PathBuf) {
    let temp_dir = TempDir::new().unwrap();
    let manifest_path = temp_dir.path().join("test-manifest.yml");
    fs::write(&manifest_path, content).unwrap();
    (temp_dir, manifest_path)
}

#[test]
fn test_parser_minimal_valid_manifest() {
    let manifest_yaml = r#"
apiVersion: ntk/v1
kind: solution
meta:
  name: TestManifest
conventions:
  namespaceRoot: TestApp
  targetFramework: net9.0
solution:
  root: ./
  slnFile: TestApp.sln
apply:
  mode: artifact
  artifact:
    kind: entity
    project: Domain
    namespace: Core.Entities
    context: Orders
"#;

    let (_temp, path) = create_temp_manifest(manifest_yaml);
    let result = ManifestParser::from_file(&path);

    assert!(result.is_ok());
    let manifest = result.unwrap();
    assert_eq!(manifest.meta.name, "TestManifest");
    assert_eq!(manifest.conventions.namespace_root, "TestApp");
}

#[test]
fn test_parser_invalid_api_version() {
    let manifest_yaml = r#"
apiVersion: ntk/v99
kind: solution
meta:
  name: TestManifest
conventions:
  namespaceRoot: TestApp
  targetFramework: net9.0
solution:
  root: ./
  slnFile: TestApp.sln
apply:
  mode: artifact
  artifact:
    kind: entity
    project: Domain
    namespace: Core
    context: Test
"#;

    let (_temp, path) = create_temp_manifest(manifest_yaml);
    let result = ManifestParser::from_file(&path);

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("unsupported apiVersion"));
}

#[test]
fn test_parser_missing_file() {
    let path = PathBuf::from("nonexistent-manifest-file.yml");
    let result = ManifestParser::from_file(&path);

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("failed to read manifest"));
}

#[test]
fn test_parser_invalid_yaml() {
    let invalid_yaml = r#"
apiVersion: ntk/v1
kind: solution
meta:
  name: TestManifest
  invalid_indent
conventions:
"#;

    let (_temp, path) = create_temp_manifest(invalid_yaml);
    let result = ManifestParser::from_file(&path);

    assert!(result.is_err());
}

#[test]
fn test_validate_empty_name() {
    let manifest_yaml = r#"
apiVersion: ntk/v1
kind: solution
meta:
  name: ""
conventions:
  namespaceRoot: TestApp
  targetFramework: net9.0
solution:
  root: ./
  slnFile: Test.sln
apply:
  mode: artifact
  artifact:
    kind: entity
    project: Domain
    namespace: Core
    context: Test
"#;

    let (_temp, path) = create_temp_manifest(manifest_yaml);
    let manifest = ManifestParser::from_file(&path).unwrap();
    let result = ManifestParser::validate(&manifest);

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("meta.name cannot be empty"));
}

#[test]
fn test_validate_empty_namespace_root() {
    let manifest_yaml = r#"
apiVersion: ntk/v1
kind: solution
meta:
  name: TestManifest
conventions:
  namespaceRoot: ""
  targetFramework: net9.0
solution:
  root: ./
  slnFile: Test.sln
apply:
  mode: artifact
  artifact:
    kind: entity
    project: Domain
    namespace: Core
    context: Test
"#;

    let (_temp, path) = create_temp_manifest(manifest_yaml);
    let manifest = ManifestParser::from_file(&path).unwrap();
    let result = ManifestParser::validate(&manifest);

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("namespaceRoot cannot be empty"));
}

#[test]
fn test_validate_artifact_mode_requires_artifact_section() {
    let manifest_yaml = r#"
apiVersion: ntk/v1
kind: solution
meta:
  name: TestManifest
conventions:
  namespaceRoot: TestApp
  targetFramework: net9.0
solution:
  root: ./
  slnFile: Test.sln
apply:
  mode: artifact
"#;

    let (_temp, path) = create_temp_manifest(manifest_yaml);

    // Parse succeeds but validation should fail
    let manifest = ManifestParser::from_file(&path).unwrap();
    let result = ManifestParser::validate(&manifest);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("artifact section is required"));
}

#[test]
fn test_parser_with_contexts() {
    let manifest_yaml = r#"
apiVersion: ntk/v1
kind: solution
meta:
  name: TestManifest
conventions:
  namespaceRoot: TestApp
  targetFramework: net9.0
solution:
  root: ./
  slnFile: Test.sln
contexts:
  - name: Orders
    aggregates:
      - name: Order
        entities:
          - name: OrderItem
            fields:
              - name: ProductId
                type: Guid
apply:
  mode: artifact
  artifact:
    kind: entity
    project: Domain
    namespace: Core
    context: Orders
"#;

    let (_temp, path) = create_temp_manifest(manifest_yaml);
    let result = ManifestParser::from_file(&path);

    assert!(result.is_ok());
    let manifest = result.unwrap();
    assert_eq!(manifest.contexts.len(), 1);
    assert_eq!(manifest.contexts[0].name, "Orders");
}

#[test]
fn test_parser_with_projects() {
    let manifest_yaml = r#"
apiVersion: ntk/v1
kind: solution
meta:
  name: TestManifest
conventions:
  namespaceRoot: TestApp
  targetFramework: net9.0
solution:
  root: ./
  slnFile: Test.sln
projects:
  Domain:
    type: domain
    name: TestApp.Domain
    path: src/TestApp.Domain
apply:
  mode: artifact
  artifact:
    kind: entity
    project: Domain
    namespace: Core
    context: Test
"#;

    let (_temp, path) = create_temp_manifest(manifest_yaml);
    let result = ManifestParser::from_file(&path);

    assert!(result.is_ok());
    let manifest = result.unwrap();
    assert_eq!(manifest.projects.len(), 1);
    assert!(manifest.projects.contains_key("Domain"));
}

#[test]
fn test_validate_successful() {
    let manifest_yaml = r#"
apiVersion: ntk/v1
kind: solution
meta:
  name: ValidManifest
conventions:
  namespaceRoot: MyApp
  targetFramework: net9.0
solution:
  root: ./
  slnFile: MyApp.sln
apply:
  mode: artifact
  artifact:
    kind: entity
    project: Domain
    namespace: Core.Entities
    context: Test
"#;

    let (_temp, path) = create_temp_manifest(manifest_yaml);
    let manifest = ManifestParser::from_file(&path).unwrap();
    let result = ManifestParser::validate(&manifest);

    assert!(result.is_ok());
}