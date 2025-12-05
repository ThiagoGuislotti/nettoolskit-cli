use crate::parsing::test_helpers::create_temp_manifest;
use nettoolskit_manifest::ManifestParser;
use std::path::PathBuf;

#[test]
fn test_parser_invalid_api_version() {
    // Arrange
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

    // Act
    let result = ManifestParser::from_file(&path);

    // Assert
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("unsupported apiVersion"));
}

#[test]
fn test_parser_missing_file() {
    // Arrange
    let path = PathBuf::from("nonexistent-manifest-file.yml");

    // Act
    let result = ManifestParser::from_file(&path);

    // Assert
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("failed to read manifest"));
}

#[test]
fn test_parser_invalid_yaml() {
    // Arrange
    let invalid_yaml = r#"
apiVersion: ntk/v1
kind: solution
meta:
  name: TestManifest
  invalid_indent
conventions:
"#;
    let (_temp, path) = create_temp_manifest(invalid_yaml);

    // Act
    let result = ManifestParser::from_file(&path);

    // Assert
    assert!(result.is_err());
}