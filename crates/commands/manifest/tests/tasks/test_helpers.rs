//! Shared test helpers and fixtures for task generation tests

use nettoolskit_manifest::models::{
    ArtifactKind, ManifestAggregate, ManifestContext, ManifestConventions, ManifestDomainEvent,
    ManifestEntity, ManifestEnum, ManifestEnumValue, ManifestField, ManifestPolicy,
    ManifestRepository, ManifestUseCase, ManifestValueObject, TemplateMapping,
};
use std::collections::BTreeMap;

pub fn create_test_conventions() -> ManifestConventions {
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

pub fn create_test_context_with_all_artifacts() -> ManifestContext {
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

pub fn build_template_index() -> BTreeMap<ArtifactKind, Vec<&'static TemplateMapping>> {
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
        let kind = ArtifactKind::parse_kind(&mapping.artifact);
        index.entry(kind).or_default().push(mapping);
    }
    index
}