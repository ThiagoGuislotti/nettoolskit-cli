//! Tests for single-artifact task generation
//!
//! Validates append_artifact_tasks() for filtered generation of
//! specific artifacts by kind and optional name.

use super::test_helpers::{
    build_template_index, create_test_context_with_all_artifacts, create_test_conventions,
};
use nettoolskit_manifest::models::{ArtifactKind, RenderTask};

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

// --- Entity artifact tests ---

#[test]
fn test_append_entity_artifact_with_name_filter() {
    let conventions = create_test_conventions();
    let context = create_test_context_with_all_artifacts();
    let contexts = vec![&context];
    let template_index = build_template_index();
    let kind = ArtifactKind::Entity;
    let mappings = template_index
        .get(&kind)
        .map(|v| v.as_slice())
        .unwrap_or(&[]);
    let mut tasks: Vec<RenderTask> = Vec::new();

    let result = nettoolskit_manifest::tasks::append_artifact_tasks(
        &mut tasks,
        &contexts,
        &conventions,
        &kind,
        Some("OrderItem"),
        mappings,
    );

    assert!(result.is_ok());
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].kind, ArtifactKind::Entity);
    assert!(tasks[0]
        .note
        .as_ref()
        .is_some_and(|n| n.contains("OrderItem")));
}

#[test]
fn test_append_entity_artifact_name_filter_no_match() {
    let conventions = create_test_conventions();
    let context = create_test_context_with_all_artifacts();
    let contexts = vec![&context];
    let template_index = build_template_index();
    let kind = ArtifactKind::Entity;
    let mappings = template_index
        .get(&kind)
        .map(|v| v.as_slice())
        .unwrap_or(&[]);
    let mut tasks: Vec<RenderTask> = Vec::new();

    let result = nettoolskit_manifest::tasks::append_artifact_tasks(
        &mut tasks,
        &contexts,
        &conventions,
        &kind,
        Some("NonExistent"),
        mappings,
    );

    assert!(result.is_ok());
    assert!(tasks.is_empty(), "No entities should match 'NonExistent'");
}

// --- DomainEvent artifact tests ---

#[test]
fn test_append_domain_event_artifact_unfiltered() {
    let conventions = create_test_conventions();
    let context = create_test_context_with_all_artifacts();
    let contexts = vec![&context];
    let template_index = build_template_index();
    let kind = ArtifactKind::DomainEvent;
    let mappings = template_index
        .get(&kind)
        .map(|v| v.as_slice())
        .unwrap_or(&[]);
    let mut tasks: Vec<RenderTask> = Vec::new();

    let result = nettoolskit_manifest::tasks::append_artifact_tasks(
        &mut tasks,
        &contexts,
        &conventions,
        &kind,
        None,
        mappings,
    );

    assert!(result.is_ok());
    assert_eq!(
        tasks.len(),
        1,
        "Should generate 1 DomainEvent (OrderCreated)"
    );
    assert_eq!(tasks[0].kind, ArtifactKind::DomainEvent);
    assert!(tasks[0]
        .note
        .as_ref()
        .is_some_and(|n| n.contains("OrderCreated")));
}

#[test]
fn test_append_domain_event_artifact_with_name_filter() {
    let conventions = create_test_conventions();
    let context = create_test_context_with_all_artifacts();
    let contexts = vec![&context];
    let template_index = build_template_index();
    let kind = ArtifactKind::DomainEvent;
    let mappings = template_index
        .get(&kind)
        .map(|v| v.as_slice())
        .unwrap_or(&[]);
    let mut tasks: Vec<RenderTask> = Vec::new();

    let result = nettoolskit_manifest::tasks::append_artifact_tasks(
        &mut tasks,
        &contexts,
        &conventions,
        &kind,
        Some("OrderCreated"),
        mappings,
    );

    assert!(result.is_ok());
    assert_eq!(tasks.len(), 1);
    assert!(tasks[0]
        .destination
        .to_string_lossy()
        .contains("OrderCreated"));
}

#[test]
fn test_append_domain_event_artifact_filter_no_match() {
    let conventions = create_test_conventions();
    let context = create_test_context_with_all_artifacts();
    let contexts = vec![&context];
    let template_index = build_template_index();
    let kind = ArtifactKind::DomainEvent;
    let mappings = template_index
        .get(&kind)
        .map(|v| v.as_slice())
        .unwrap_or(&[]);
    let mut tasks: Vec<RenderTask> = Vec::new();

    let result = nettoolskit_manifest::tasks::append_artifact_tasks(
        &mut tasks,
        &contexts,
        &conventions,
        &kind,
        Some("NoSuchEvent"),
        mappings,
    );

    assert!(result.is_ok());
    assert!(tasks.is_empty());
}

// --- RepositoryInterface artifact tests ---

#[test]
fn test_append_repository_artifact_unfiltered() {
    let conventions = create_test_conventions();
    let context = create_test_context_with_all_artifacts();
    let contexts = vec![&context];
    let template_index = build_template_index();
    let kind = ArtifactKind::RepositoryInterface;
    let mappings = template_index
        .get(&kind)
        .map(|v| v.as_slice())
        .unwrap_or(&[]);
    let mut tasks: Vec<RenderTask> = Vec::new();

    let result = nettoolskit_manifest::tasks::append_artifact_tasks(
        &mut tasks,
        &contexts,
        &conventions,
        &kind,
        None,
        mappings,
    );

    assert!(result.is_ok());
    assert_eq!(tasks.len(), 1, "Should generate 1 Repository (Order)");
    assert_eq!(tasks[0].kind, ArtifactKind::RepositoryInterface);
    assert!(tasks[0]
        .note
        .as_ref()
        .is_some_and(|n| n.contains("Repository")));
}

#[test]
fn test_append_repository_artifact_with_name_filter() {
    let conventions = create_test_conventions();
    let context = create_test_context_with_all_artifacts();
    let contexts = vec![&context];
    let template_index = build_template_index();
    let kind = ArtifactKind::RepositoryInterface;
    let mappings = template_index
        .get(&kind)
        .map(|v| v.as_slice())
        .unwrap_or(&[]);
    let mut tasks: Vec<RenderTask> = Vec::new();

    let result = nettoolskit_manifest::tasks::append_artifact_tasks(
        &mut tasks,
        &contexts,
        &conventions,
        &kind,
        Some("Order"),
        mappings,
    );

    assert!(result.is_ok());
    assert_eq!(tasks.len(), 1);
    assert!(tasks[0].destination.to_string_lossy().contains("Order"));
}

#[test]
fn test_append_repository_artifact_filter_no_match() {
    let conventions = create_test_conventions();
    let context = create_test_context_with_all_artifacts();
    let contexts = vec![&context];
    let template_index = build_template_index();
    let kind = ArtifactKind::RepositoryInterface;
    let mappings = template_index
        .get(&kind)
        .map(|v| v.as_slice())
        .unwrap_or(&[]);
    let mut tasks: Vec<RenderTask> = Vec::new();

    let result = nettoolskit_manifest::tasks::append_artifact_tasks(
        &mut tasks,
        &contexts,
        &conventions,
        &kind,
        Some("Product"),
        mappings,
    );

    assert!(result.is_ok());
    assert!(tasks.is_empty());
}

// --- EnumType artifact tests ---

#[test]
fn test_append_enum_artifact_unfiltered() {
    let conventions = create_test_conventions();
    let context = create_test_context_with_all_artifacts();
    let contexts = vec![&context];
    let template_index = build_template_index();
    let kind = ArtifactKind::EnumType;
    let mappings = template_index
        .get(&kind)
        .map(|v| v.as_slice())
        .unwrap_or(&[]);
    let mut tasks: Vec<RenderTask> = Vec::new();

    let result = nettoolskit_manifest::tasks::append_artifact_tasks(
        &mut tasks,
        &contexts,
        &conventions,
        &kind,
        None,
        mappings,
    );

    assert!(result.is_ok());
    assert_eq!(tasks.len(), 1, "Should generate 1 Enum (OrderStatus)");
    assert_eq!(tasks[0].kind, ArtifactKind::EnumType);
    assert!(tasks[0]
        .note
        .as_ref()
        .is_some_and(|n| n.contains("OrderStatus")));
}

#[test]
fn test_append_enum_artifact_with_name_filter() {
    let conventions = create_test_conventions();
    let context = create_test_context_with_all_artifacts();
    let contexts = vec![&context];
    let template_index = build_template_index();
    let kind = ArtifactKind::EnumType;
    let mappings = template_index
        .get(&kind)
        .map(|v| v.as_slice())
        .unwrap_or(&[]);
    let mut tasks: Vec<RenderTask> = Vec::new();

    let result = nettoolskit_manifest::tasks::append_artifact_tasks(
        &mut tasks,
        &contexts,
        &conventions,
        &kind,
        Some("OrderStatus"),
        mappings,
    );

    assert!(result.is_ok());
    assert_eq!(tasks.len(), 1);
    let data = &tasks[0].data;
    assert_eq!(data["values"].as_array().unwrap().len(), 2);
}

#[test]
fn test_append_enum_artifact_filter_no_match() {
    let conventions = create_test_conventions();
    let context = create_test_context_with_all_artifacts();
    let contexts = vec![&context];
    let template_index = build_template_index();
    let kind = ArtifactKind::EnumType;
    let mappings = template_index
        .get(&kind)
        .map(|v| v.as_slice())
        .unwrap_or(&[]);
    let mut tasks: Vec<RenderTask> = Vec::new();

    let result = nettoolskit_manifest::tasks::append_artifact_tasks(
        &mut tasks,
        &contexts,
        &conventions,
        &kind,
        Some("PaymentMethod"),
        mappings,
    );

    assert!(result.is_ok());
    assert!(tasks.is_empty());
}

// --- Endpoint artifact tests ---

#[test]
fn test_append_endpoint_artifact_unfiltered() {
    let conventions = create_test_conventions();
    let context = create_test_context_with_all_artifacts();
    let contexts = vec![&context];
    let template_index = build_template_index();
    let kind = ArtifactKind::Endpoint;
    let mappings = template_index
        .get(&kind)
        .map(|v| v.as_slice())
        .unwrap_or(&[]);
    let mut tasks: Vec<RenderTask> = Vec::new();

    let result = nettoolskit_manifest::tasks::append_artifact_tasks(
        &mut tasks,
        &contexts,
        &conventions,
        &kind,
        None,
        mappings,
    );

    assert!(result.is_ok());
    assert_eq!(
        tasks.len(),
        1,
        "Should generate 1 Endpoint (CreateOrderController)"
    );
    assert_eq!(tasks[0].kind, ArtifactKind::Endpoint);
    assert!(tasks[0]
        .note
        .as_ref()
        .is_some_and(|n| n.contains("Controller")));
}

#[test]
fn test_append_endpoint_artifact_with_name_filter() {
    let conventions = create_test_conventions();
    let context = create_test_context_with_all_artifacts();
    let contexts = vec![&context];
    let template_index = build_template_index();
    let kind = ArtifactKind::Endpoint;
    let mappings = template_index
        .get(&kind)
        .map(|v| v.as_slice())
        .unwrap_or(&[]);
    let mut tasks: Vec<RenderTask> = Vec::new();

    let result = nettoolskit_manifest::tasks::append_artifact_tasks(
        &mut tasks,
        &contexts,
        &conventions,
        &kind,
        Some("CreateOrder"),
        mappings,
    );

    assert!(result.is_ok());
    assert_eq!(tasks.len(), 1);
    assert!(tasks[0]
        .destination
        .to_string_lossy()
        .contains("CreateOrderController"));
}

#[test]
fn test_append_endpoint_artifact_filter_no_match() {
    let conventions = create_test_conventions();
    let context = create_test_context_with_all_artifacts();
    let contexts = vec![&context];
    let template_index = build_template_index();
    let kind = ArtifactKind::Endpoint;
    let mappings = template_index
        .get(&kind)
        .map(|v| v.as_slice())
        .unwrap_or(&[]);
    let mut tasks: Vec<RenderTask> = Vec::new();

    let result = nettoolskit_manifest::tasks::append_artifact_tasks(
        &mut tasks,
        &contexts,
        &conventions,
        &kind,
        Some("DeleteOrder"),
        mappings,
    );

    assert!(result.is_ok());
    assert!(tasks.is_empty());
}

// --- Unknown kind test ---

#[test]
fn test_append_artifact_tasks_unknown_kind_returns_error() {
    let conventions = create_test_conventions();
    let context = create_test_context_with_all_artifacts();
    let contexts = vec![&context];
    let kind = ArtifactKind::Unknown("widget".to_string());
    let mut tasks: Vec<RenderTask> = Vec::new();

    let result = nettoolskit_manifest::tasks::append_artifact_tasks(
        &mut tasks,
        &contexts,
        &conventions,
        &kind,
        None,
        &[],
    );

    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("widget"),
        "Error should mention the unknown kind"
    );
}

// --- Data shape validation tests ---

#[test]
fn test_domain_event_data_shape() {
    let conventions = create_test_conventions();
    let context = create_test_context_with_all_artifacts();
    let contexts = vec![&context];
    let template_index = build_template_index();
    let kind = ArtifactKind::DomainEvent;
    let mappings = template_index
        .get(&kind)
        .map(|v| v.as_slice())
        .unwrap_or(&[]);
    let mut tasks: Vec<RenderTask> = Vec::new();

    nettoolskit_manifest::tasks::append_artifact_tasks(
        &mut tasks,
        &contexts,
        &conventions,
        &kind,
        None,
        mappings,
    )
    .unwrap();

    let data = &tasks[0].data;
    assert_eq!(data["namespace"], "TestApp.Domain.Events");
    assert_eq!(data["contextName"], "Orders");
    assert_eq!(data["aggregateName"], "Order");
    assert_eq!(data["name"], "OrderCreated");
}

#[test]
fn test_repository_data_shape() {
    let conventions = create_test_conventions();
    let context = create_test_context_with_all_artifacts();
    let contexts = vec![&context];
    let template_index = build_template_index();
    let kind = ArtifactKind::RepositoryInterface;
    let mappings = template_index
        .get(&kind)
        .map(|v| v.as_slice())
        .unwrap_or(&[]);
    let mut tasks: Vec<RenderTask> = Vec::new();

    nettoolskit_manifest::tasks::append_artifact_tasks(
        &mut tasks,
        &contexts,
        &conventions,
        &kind,
        None,
        mappings,
    )
    .unwrap();

    let data = &tasks[0].data;
    assert_eq!(data["namespace"], "TestApp.Domain.Repositories");
    assert_eq!(data["name"], "Order");
    assert!(data["methods"].is_array());
}

#[test]
fn test_enum_data_shape() {
    let conventions = create_test_conventions();
    let context = create_test_context_with_all_artifacts();
    let contexts = vec![&context];
    let template_index = build_template_index();
    let kind = ArtifactKind::EnumType;
    let mappings = template_index
        .get(&kind)
        .map(|v| v.as_slice())
        .unwrap_or(&[]);
    let mut tasks: Vec<RenderTask> = Vec::new();

    nettoolskit_manifest::tasks::append_artifact_tasks(
        &mut tasks,
        &contexts,
        &conventions,
        &kind,
        None,
        mappings,
    )
    .unwrap();

    let data = &tasks[0].data;
    assert_eq!(data["namespace"], "TestApp.Domain.Enums");
    assert_eq!(data["name"], "OrderStatus");
    let values = data["values"].as_array().unwrap();
    assert_eq!(values[0]["name"], "Pending");
    assert_eq!(values[0]["value"], 0);
    assert_eq!(values[1]["name"], "Confirmed");
    assert_eq!(values[1]["value"], 1);
}

#[test]
fn test_endpoint_data_shape() {
    let conventions = create_test_conventions();
    let context = create_test_context_with_all_artifacts();
    let contexts = vec![&context];
    let template_index = build_template_index();
    let kind = ArtifactKind::Endpoint;
    let mappings = template_index
        .get(&kind)
        .map(|v| v.as_slice())
        .unwrap_or(&[]);
    let mut tasks: Vec<RenderTask> = Vec::new();

    nettoolskit_manifest::tasks::append_artifact_tasks(
        &mut tasks,
        &contexts,
        &conventions,
        &kind,
        None,
        mappings,
    )
    .unwrap();

    let data = &tasks[0].data;
    assert_eq!(data["namespace"], "TestApp.Api.Controllers");
    assert_eq!(data["name"], "CreateOrderController");
    assert_eq!(data["useCaseName"], "CreateOrder");
    assert!(data["input"].is_array());
    assert!(data["output"].is_array());
}
