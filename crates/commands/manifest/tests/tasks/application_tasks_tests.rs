//! Tests for application layer task generation
//!
//! Validates append_application_tasks() generates correct RenderTasks
//! for use cases (commands, queries, etc).

use nettoolskit_manifest::models::{ArtifactKind, ManifestContext, RenderTask};
use super::test_helpers::{
    build_template_index, create_test_context_with_all_artifacts, create_test_conventions,
};

#[test]
fn test_append_application_tasks_generates_use_cases() {
    // Arrange
    let conventions = create_test_conventions();
    let context = create_test_context_with_all_artifacts();
    let contexts = vec![&context];
    let template_index = build_template_index();
    let mut tasks: Vec<RenderTask> = Vec::new();

    // Act
    let result = nettoolskit_manifest::tasks::append_application_tasks(
        &mut tasks,
        &contexts,
        &conventions,
        &template_index,
    );

    // Assert
    assert!(result.is_ok());
    assert_eq!(tasks.len(), 1, "Should generate 1 use case");
    assert_eq!(tasks[0].kind, ArtifactKind::UseCaseCommand);
    assert_eq!(tasks[0].template, "Application/UseCase.cs.hbs");
}

#[test]
fn test_append_application_tasks_with_no_use_cases() {
    // Arrange
    let conventions = create_test_conventions();
    let context = ManifestContext {
        name: "NoUseCases".to_string(),
        aggregates: vec![],
        use_cases: vec![],
    };
    let contexts = vec![&context];
    let template_index = build_template_index();
    let mut tasks: Vec<RenderTask> = Vec::new();

    // Act
    let result = nettoolskit_manifest::tasks::append_application_tasks(
        &mut tasks,
        &contexts,
        &conventions,
        &template_index,
    );

    // Assert
    assert!(result.is_ok());
    assert_eq!(tasks.len(), 0);
}