/// Tests for task generation modules (domain, application, api, artifact)
///
/// Note: These tests validate the PUBLIC API and structure of task generation.
/// Full integration tests with actual rendering are in integration tests.
use nettoolskit_manifest::models::{
    ArtifactKind, ManifestAggregate, ManifestContext, ManifestConventions, ManifestDomainEvent,
    ManifestEntity, ManifestEnum, ManifestEnumValue, ManifestField, ManifestPolicy,
    ManifestRepository, ManifestUseCase, ManifestValueObject, RenderTask, TemplateMapping,
};
use std::collections::BTreeMap;

// Test Helpers

fn create_test_conventions() -> ManifestConventions {
    ManifestConventions {
        namespace_root: "TestApp".to_string(),
        target_framework: "net8.0".to_string(),
        policy: ManifestPolicy {
            collision: None,
            insert_todo_when_missing: false,
            strict: false,
        },
    }
}

fn create_test_context_with_all_artifacts() -> ManifestContext {
    ManifestContext {
        name: "Orders".to_string(),
        aggregates: vec![ManifestAggregate {
            name: "Order".to_string(),
            value_objects: vec![ManifestValueObject {
                name: "OrderId".to_string(),
                fields: vec![ManifestField {
                    name: "value".to_string(),
                    r#type: "Guid".to_string(),
                    nullable: false,
                    key: false,
                    column_name: None,
                }],
            }],
            entities: vec![ManifestEntity {
                name: "OrderItem".to_string(),
                fields: vec![ManifestField {
                    name: "quantity".to_string(),
                    r#type: "int".to_string(),
                    nullable: false,
                    key: false,
                    column_name: None,
                }],
            }],
            domain_events: vec![ManifestDomainEvent {
                name: "OrderCreated".to_string(),
            }],
            repository: Some(ManifestRepository {
                name: "Order".to_string(),
                methods: vec![],
            }),
            enums: vec![ManifestEnum {
                name: "OrderStatus".to_string(),
                values: vec![
                    ManifestEnumValue {
                        name: "Pending".to_string(),
                        value: 0,
                    },
                    ManifestEnumValue {
                        name: "Confirmed".to_string(),
                        value: 1,
                    },
                ],
            }],
        }],
        use_cases: vec![ManifestUseCase {
            name: "CreateOrder".to_string(),
            use_case_type: "command".to_string(),
            input: vec![ManifestField {
                name: "customerId".to_string(),
                r#type: "Guid".to_string(),
                nullable: false,
                key: false,
                column_name: None,
            }],
            output: vec![ManifestField {
                name: "orderId".to_string(),
                r#type: "Guid".to_string(),
                nullable: false,
                key: false,
                column_name: None,
            }],
        }],
    }
}

fn build_template_index() -> BTreeMap<ArtifactKind, Vec<&'static TemplateMapping>> {
    // Using Box::leak to create static references for testing
    let mappings: Vec<TemplateMapping> = vec![
        TemplateMapping {
            artifact: "value-object".to_string(),
            template: "Domain/ValueObject.cs.hbs".to_string(),
            dst: "{context}/Domain/ValueObjects/{name}.cs".to_string(),
        },
        TemplateMapping {
            artifact: "entity".to_string(),
            template: "Domain/Entity.cs.hbs".to_string(),
            dst: "{context}/Domain/Entities/{name}.cs".to_string(),
        },
        TemplateMapping {
            artifact: "domain-event".to_string(),
            template: "Domain/DomainEvent.cs.hbs".to_string(),
            dst: "{context}/Domain/Events/{name}.cs".to_string(),
        },
        TemplateMapping {
            artifact: "repository-interface".to_string(),
            template: "Domain/IRepository.cs.hbs".to_string(),
            dst: "{context}/Domain/Repositories/I{name}Repository.cs".to_string(),
        },
        TemplateMapping {
            artifact: "enum".to_string(),
            template: "Domain/Enum.cs.hbs".to_string(),
            dst: "{context}/Domain/Enums/{name}.cs".to_string(),
        },
        TemplateMapping {
            artifact: "usecase-command".to_string(),
            template: "Application/UseCase.cs.hbs".to_string(),
            dst: "{context}/Application/UseCases/{name}.cs".to_string(),
        },
        TemplateMapping {
            artifact: "endpoint".to_string(),
            template: "API/Controller.cs.hbs".to_string(),
            dst: "{context}/API/Controllers/{name}Controller.cs".to_string(),
        },
    ];

    let leaked: &'static [TemplateMapping] = Box::leak(mappings.into_boxed_slice());

    let mut index: BTreeMap<ArtifactKind, Vec<&'static TemplateMapping>> = BTreeMap::new();
    for mapping in leaked {
        let kind = ArtifactKind::from_str(&mapping.artifact);
        index.entry(kind).or_default().push(mapping);
    }
    index
}

// Domain Tasks Tests

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
    assert!(tasks.iter().any(|t| t.kind == ArtifactKind::RepositoryInterface));
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

    let value_object_task = tasks.iter().find(|t| t.kind == ArtifactKind::ValueObject).unwrap();
    assert_eq!(value_object_task.template, "Domain/ValueObject.cs.hbs");

    let entity_task = tasks.iter().find(|t| t.kind == ArtifactKind::Entity).unwrap();
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

// Application Tasks Tests

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

// API Tasks Tests

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

// Artifact Tasks Tests (Single Artifact Mode)

#[test]
fn test_append_artifact_tasks_with_value_object_filter() {
    // Arrange
    let conventions = create_test_conventions();
    let context = create_test_context_with_all_artifacts();
    let contexts = vec![&context];
    let template_index = build_template_index();
    let kind = ArtifactKind::ValueObject;
    let mappings = template_index.get(&kind).map(|v| v.as_slice()).unwrap_or(&[]);
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
    let mappings = template_index.get(&kind).map(|v| v.as_slice()).unwrap_or(&[]);
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
    assert_eq!(tasks.len(), 1, "Should generate all ValueObjects (1 in test data)");
}
