/// Tests for manifest domain models
use nettoolskit_manifest::{
    ManifestKind, ManifestProjectKind, ExecutionSummary,
};
use std::path::PathBuf;

#[test]
fn test_manifest_kind_solution() {
    // ManifestKind::Solution should be deserializable
    let yaml = "solution";
    let kind: ManifestKind = serde_yaml::from_str(yaml).unwrap();

    assert!(matches!(kind, ManifestKind::Solution));
}

#[test]
fn test_manifest_project_kind_default() {
    let kind = ManifestProjectKind::default();

    assert!(matches!(kind, ManifestProjectKind::Unknown));
}

#[test]
fn test_manifest_project_kind_domain_template() {
    let kind = ManifestProjectKind::Domain;
    let template = kind.template_path();

    assert!(template.is_some());
    assert!(template.unwrap().contains("domain.csproj.hbs"));
}

#[test]
fn test_manifest_project_kind_unknown_no_template() {
    let kind = ManifestProjectKind::Unknown;
    let template = kind.template_path();

    assert!(template.is_none());
}

#[test]
fn test_manifest_project_kind_application() {
    let yaml = "application";
    let kind: ManifestProjectKind = serde_yaml::from_str(yaml).unwrap();

    assert!(matches!(kind, ManifestProjectKind::Application));
}

#[test]
fn test_manifest_project_kind_infrastructure() {
    let yaml = "infrastructure";
    let kind: ManifestProjectKind = serde_yaml::from_str(yaml).unwrap();

    assert!(matches!(kind, ManifestProjectKind::Infrastructure));
}

#[test]
fn test_manifest_project_kind_api() {
    let yaml = "api";
    let kind: ManifestProjectKind = serde_yaml::from_str(yaml).unwrap();

    assert!(matches!(kind, ManifestProjectKind::Api));
}

#[test]
fn test_manifest_project_kind_worker() {
    let yaml = "worker";
    let kind: ManifestProjectKind = serde_yaml::from_str(yaml).unwrap();

    assert!(matches!(kind, ManifestProjectKind::Worker));
}

#[test]
fn test_execution_summary_default() {
    let summary = ExecutionSummary::default();

    assert_eq!(summary.created.len(), 0);
    assert_eq!(summary.updated.len(), 0);
    assert_eq!(summary.skipped.len(), 0);
    assert_eq!(summary.notes.len(), 0);
}

#[test]
fn test_execution_summary_add_created() {
    let mut summary = ExecutionSummary::default();

    summary.created.push(PathBuf::from("src/Entity.cs"));
    summary.created.push(PathBuf::from("src/Repository.cs"));

    assert_eq!(summary.created.len(), 2);
    assert!(summary.created.contains(&PathBuf::from("src/Entity.cs")));
}

#[test]
fn test_execution_summary_add_updated() {
    let mut summary = ExecutionSummary::default();

    summary.updated.push(PathBuf::from("src/Existing.cs"));

    assert_eq!(summary.updated.len(), 1);
    assert_eq!(summary.updated[0], PathBuf::from("src/Existing.cs"));
}

#[test]
fn test_execution_summary_add_skipped() {
    let mut summary = ExecutionSummary::default();

    summary.skipped.push((PathBuf::from("src/Collision.cs"), "already exists".to_string()));

    assert_eq!(summary.skipped.len(), 1);
    assert_eq!(summary.skipped[0].0, PathBuf::from("src/Collision.cs"));
}

#[test]
fn test_execution_summary_add_notes() {
    let mut summary = ExecutionSummary::default();

    summary.notes.push("Processing context: Orders".to_string());
    summary.notes.push("Generated 5 files".to_string());

    assert_eq!(summary.notes.len(), 2);
    assert!(summary.notes[0].contains("Orders"));
}

#[test]
fn test_execution_summary_fields() {
    let mut summary = ExecutionSummary::default();
    summary.created.push(PathBuf::from("test.cs"));
    summary.notes.push("note".to_string());

    assert_eq!(summary.created.len(), 1);
    assert_eq!(summary.notes.len(), 1);
}

#[test]
fn test_execution_summary_debug() {
    let summary = ExecutionSummary::default();
    let debug_str = format!("{:?}", summary);

    assert!(debug_str.contains("ExecutionSummary"));
}