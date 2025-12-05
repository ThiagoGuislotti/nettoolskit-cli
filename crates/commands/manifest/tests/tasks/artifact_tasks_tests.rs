//! Tests for single-artifact task generation
//!
//! Validates append_artifact_tasks() for filtered generation of
//! specific artifacts by kind and optional name.

use nettoolskit_manifest::models::{ArtifactKind, RenderTask};
use super::test_helpers::{
    build_template_index, create_test_context_with_all_artifacts, create_test_conventions,
};

#[test]
fn test_append_artifact_tasks_with_value_object_filter() {
    // Arrange
    let conventions = create_test_conventions();
    let context = create_test_context_with_all_artifacts();
    let contexts = vec![&context];
    let template_index = build_template_index();
    let kind = ArtifactKind::ValueObject;
    let mappings = template_index
        .get(&kind)
        .map(|v| v.as_slice())
        .unwrap_or(&[]);
    let mut tasks: Vec<RenderTask> = Vec::new();

    // Act
    let result = nettoolskit_manifest::tasks::append_artifact_tasks(
        &mut tasks,
        &contexts,
        &conventions,
        &kind,
        Some("OrderId"),
        mappings,
    );

    // Assert
    assert!(result.is_ok());
    assert_eq!(tasks.len(), 1, "Should generate 1 filtered artifact");
    assert_eq!(tasks[0].kind, ArtifactKind::ValueObject);
}

#[test]
fn test_append_artifact_tasks_without_filter_generates_all_of_kind() {
    // Arrange
    let conventions = create_test_conventions();
    let context = create_test_context_with_all_artifacts();
    let contexts = vec![&context];
    let template_index = build_template_index();
    let kind = ArtifactKind::ValueObject;
    let mappings = template_index
        .get(&kind)
        .map(|v| v.as_slice())
        .unwrap_or(&[]);
    let mut tasks: Vec<RenderTask> = Vec::new();

    // Act - No name filter, should generate all ValueObjects
    let result = nettoolskit_manifest::tasks::append_artifact_tasks(
        &mut tasks,
        &contexts,
        &conventions,
        &kind,
        None,
        mappings,
    );

    // Assert
    assert!(result.is_ok());
    assert_eq!(
        tasks.len(),
        1,
        "Should generate all ValueObjects (1 in test data)"
    );
}