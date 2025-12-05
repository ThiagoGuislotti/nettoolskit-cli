use crate::parsing::test_helpers::create_temp_manifest;
use nettoolskit_manifest::ManifestParser;

#[test]
fn test_parser_minimal_valid_manifest() {
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

    // Act
    let result = ManifestParser::from_file(&path);

    // Assert
    assert!(result.is_ok());
    let manifest = result.unwrap();
    assert_eq!(manifest.meta.name, "TestManifest");
    assert_eq!(manifest.conventions.namespace_root, "TestApp");
}

#[test]
fn test_parser_with_contexts() {
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

    // Act
    let result = ManifestParser::from_file(&path);

    // Assert
    assert!(result.is_ok());
    let manifest = result.unwrap();
    assert_eq!(manifest.contexts.len(), 1);
    assert_eq!(manifest.contexts[0].name, "Orders");
}

#[test]
fn test_parser_with_projects() {
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

    // Act
    let result = ManifestParser::from_file(&path);

    // Assert
    assert!(result.is_ok());
    let manifest = result.unwrap();
    assert_eq!(manifest.projects.len(), 1);
    assert!(manifest.projects.contains_key("Domain"));
}