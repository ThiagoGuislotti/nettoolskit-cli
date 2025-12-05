//! Tests for API layer task generation
//!
//! Validates append_api_tasks() generates correct RenderTasks
//! for controllers, endpoints, and API infrastructure.

use nettoolskit_manifest::models::{ArtifactKind, RenderTask};
use super::test_helpers::{
    build_template_index, create_test_context_with_all_artifacts, create_test_conventions,
};

#[test]
fn test_append_api_tasks_generates_controllers() {
    // Arrange
    let conventions = create_test_conventions();
    let context = create_test_context_with_all_artifacts();
    let contexts = vec![&context];
    let template_index = build_template_index();
    let mut tasks: Vec<RenderTask> = Vec::new();

    // Act
    let result = nettoolskit_manifest::tasks::append_api_tasks(
        &mut tasks,
        &contexts,
        &conventions,
        &template_index,
    );

    // Assert
    assert!(result.is_ok());
    assert_eq!(tasks.len(), 1, "Should generate 1 controller");
    assert_eq!(tasks[0].kind, ArtifactKind::Endpoint);
    assert_eq!(tasks[0].template, "API/Controller.cs.hbs");
}