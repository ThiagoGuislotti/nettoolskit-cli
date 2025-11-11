/// Tests for manifest domain models
use nettoolskit_manifest::{ExecutionSummary, ManifestKind, ManifestProjectKind};
use std::path::PathBuf;

// ManifestKind Tests

#[test]
fn test_manifest_kind_solution() {
    // Arrange
    let yaml = "solution";

    // Act
    let kind: ManifestKind = serde_yaml::from_str(yaml).unwrap();

    // Assert
    assert!(matches!(kind, ManifestKind::Solution));
}

// ManifestProjectKind Tests

#[test]
fn test_manifest_project_kind_default() {
    // Act
    let kind = ManifestProjectKind::default();

    // Assert
    assert!(matches!(kind, ManifestProjectKind::Unknown));
}

#[test]
fn test_manifest_project_kind_domain_template() {
    // Arrange
    let kind = ManifestProjectKind::Domain;

    // Act
    let template = kind.template_path();

    // Assert
    assert!(template.is_some());
    assert!(template.unwrap().contains("domain.csproj.hbs"));
}

#[test]
fn test_manifest_project_kind_unknown_no_template() {
    // Arrange
    let kind = ManifestProjectKind::Unknown;

    // Act
    let template = kind.template_path();

    // Assert
    assert!(template.is_none());
}

#[test]
fn test_manifest_project_kind_application() {
    // Arrange
    let yaml = "application";

    // Act
    let kind: ManifestProjectKind = serde_yaml::from_str(yaml).unwrap();

    // Assert
    assert!(matches!(kind, ManifestProjectKind::Application));
}

#[test]
fn test_manifest_project_kind_infrastructure() {
    // Arrange
    let yaml = "infrastructure";

    // Act
    let kind: ManifestProjectKind = serde_yaml::from_str(yaml).unwrap();

    // Assert
    assert!(matches!(kind, ManifestProjectKind::Infrastructure));
}

#[test]
fn test_manifest_project_kind_api() {
    // Arrange
    let yaml = "api";

    // Act
    let kind: ManifestProjectKind = serde_yaml::from_str(yaml).unwrap();

    // Assert
    assert!(matches!(kind, ManifestProjectKind::Api));
}

#[test]
fn test_manifest_project_kind_worker() {
    // Arrange
    let yaml = "worker";

    // Act
    let kind: ManifestProjectKind = serde_yaml::from_str(yaml).unwrap();

    // Assert
    assert!(matches!(kind, ManifestProjectKind::Worker));
}

// ExecutionSummary Tests

#[test]
fn test_execution_summary_default() {
    // Act
    let summary = ExecutionSummary::default();

    // Assert
    assert_eq!(summary.created.len(), 0);
    assert_eq!(summary.updated.len(), 0);
    assert_eq!(summary.skipped.len(), 0);
    assert_eq!(summary.notes.len(), 0);
}

#[test]
fn test_execution_summary_add_created() {
    // Arrange
    let mut summary = ExecutionSummary::default();

    // Act
    summary.created.push(PathBuf::from("src/Entity.cs"));
    summary.created.push(PathBuf::from("src/Repository.cs"));

    // Assert
    assert_eq!(summary.created.len(), 2);
    assert!(summary.created.contains(&PathBuf::from("src/Entity.cs")));
}

#[test]
fn test_execution_summary_add_updated() {
    // Arrange
    let mut summary = ExecutionSummary::default();

    // Act
    summary.updated.push(PathBuf::from("src/Existing.cs"));

    // Assert
    assert_eq!(summary.updated.len(), 1);
    assert_eq!(summary.updated[0], PathBuf::from("src/Existing.cs"));
}

#[test]
fn test_execution_summary_add_skipped() {
    // Arrange
    let mut summary = ExecutionSummary::default();

    // Act
    summary.skipped.push((
        PathBuf::from("src/Collision.cs"),
        "already exists".to_string(),
    ));

    // Assert
    assert_eq!(summary.skipped.len(), 1);
    assert_eq!(summary.skipped[0].0, PathBuf::from("src/Collision.cs"));
}

#[test]
fn test_execution_summary_add_notes() {
    // Arrange
    let mut summary = ExecutionSummary::default();

    // Act
    summary.notes.push("Processing context: Orders".to_string());
    summary.notes.push("Generated 5 files".to_string());

    // Assert
    assert_eq!(summary.notes.len(), 2);
    assert!(summary.notes[0].contains("Orders"));
}

#[test]
fn test_execution_summary_fields() {
    // Arrange
    let mut summary = ExecutionSummary::default();

    // Act
    summary.created.push(PathBuf::from("test.cs"));
    summary.notes.push("note".to_string());

    // Assert
    assert_eq!(summary.created.len(), 1);
    assert_eq!(summary.notes.len(), 1);
}

#[test]
fn test_execution_summary_debug() {
    // Arrange
    let summary = ExecutionSummary::default();

    // Act
    let debug_str = format!("{:?}", summary);

    // Assert
    assert!(debug_str.contains("ExecutionSummary"));
}
