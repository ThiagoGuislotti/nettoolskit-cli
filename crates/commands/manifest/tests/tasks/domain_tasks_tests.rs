//! Tests for domain artifact task generation
//!
//! Validates append_domain_tasks() generates correct RenderTasks for
//! value objects, entities, domain events, repositories, and enums.

use nettoolskit_manifest::models::{ArtifactKind, ManifestContext, RenderTask};
use super::test_helpers::{
    build_template_index, create_test_context_with_all_artifacts, create_test_conventions,
};

#[test]
fn test_append_domain_tasks_generates_all_artifact_types() {
    // Arrange
    let conventions = create_test_conventions();
    let context = create_test_context_with_all_artifacts();
    let contexts = vec![&context];
    let template_index = build_template_index();
    let mut tasks: Vec<RenderTask> = Vec::new();

    // Act
    let result = nettoolskit_manifest::tasks::append_domain_tasks(
        &mut tasks,
        &contexts,
        &conventions,
        &template_index,
    );

    // Assert
    assert!(result.is_ok(), "append_domain_tasks should succeed");

    // Should generate: 1 ValueObject + 1 Entity + 1 DomainEvent + 1 Repository + 1 Enum = 5 tasks
    assert_eq!(tasks.len(), 5, "Should generate 5 domain artifacts");

    // Verify each artifact type is present
    assert!(tasks.iter().any(|t| t.kind == ArtifactKind::ValueObject));
    assert!(tasks.iter().any(|t| t.kind == ArtifactKind::Entity));
    assert!(tasks.iter().any(|t| t.kind == ArtifactKind::DomainEvent));
    assert!(tasks
        .iter()
        .any(|t| t.kind == ArtifactKind::RepositoryInterface));
    assert!(tasks.iter().any(|t| t.kind == ArtifactKind::EnumType));
}

#[test]
fn test_append_domain_tasks_uses_correct_templates() {
    // Arrange
    let conventions = create_test_conventions();
    let context = create_test_context_with_all_artifacts();
    let contexts = vec![&context];
    let template_index = build_template_index();
    let mut tasks: Vec<RenderTask> = Vec::new();

    // Act
    let result = nettoolskit_manifest::tasks::append_domain_tasks(
        &mut tasks,
        &contexts,
        &conventions,
        &template_index,
    );

    // Assert
    assert!(result.is_ok());

    let value_object_task = tasks
        .iter()
        .find(|t| t.kind == ArtifactKind::ValueObject)
        .unwrap();
    assert_eq!(value_object_task.template, "Domain/ValueObject.cs.hbs");

    let entity_task = tasks
        .iter()
        .find(|t| t.kind == ArtifactKind::Entity)
        .unwrap();
    assert_eq!(entity_task.template, "Domain/Entity.cs.hbs");
}

#[test]
fn test_append_domain_tasks_with_empty_context() {
    // Arrange
    let conventions = create_test_conventions();
    let empty_context = ManifestContext {
        name: "Empty".to_string(),
        aggregates: vec![],
        use_cases: vec![],
    };
    let contexts = vec![&empty_context];
    let template_index = build_template_index();
    let mut tasks: Vec<RenderTask> = Vec::new();

    // Act
    let result = nettoolskit_manifest::tasks::append_domain_tasks(
        &mut tasks,
        &contexts,
        &conventions,
        &template_index,
    );

    // Assert
    assert!(result.is_ok());
    assert_eq!(tasks.len(), 0, "Empty context should generate no tasks");
}