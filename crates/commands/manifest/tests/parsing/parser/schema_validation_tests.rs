use crate::parsing::test_helpers::create_temp_manifest;
use nettoolskit_manifest::ManifestParser;

#[test]
fn test_validate_empty_name() {
    // Arrange
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

    // Act
    let manifest = ManifestParser::from_file(&path).unwrap();
    let result = ManifestParser::validate(&manifest);

    // Assert
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("meta.name cannot be empty"));
}

#[test]
fn test_validate_empty_namespace_root() {
    // Arrange
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

    // Act
    let manifest = ManifestParser::from_file(&path).unwrap();
    let result = ManifestParser::validate(&manifest);

    // Assert
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("namespaceRoot cannot be empty"));
}

#[test]
fn test_validate_artifact_mode_requires_artifact_section() {
    // Arrange
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

    // Act
    let manifest = ManifestParser::from_file(&path).unwrap();
    let result = ManifestParser::validate(&manifest);

    // Assert
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("artifact section is required"));
}

#[test]
fn test_validate_successful() {
    // Arrange
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

    // Act
    let manifest = ManifestParser::from_file(&path).unwrap();
    let result = ManifestParser::validate(&manifest);

    // Assert
    assert!(result.is_ok());
}