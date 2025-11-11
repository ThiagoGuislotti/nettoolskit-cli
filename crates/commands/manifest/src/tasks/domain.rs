/// Domain layer task generation
use crate::error::ManifestResult;
use crate::models::{
    ArtifactKind, ManifestContext, ManifestConventions, RenderTask, TemplateMapping,
};
use serde_json::json;
use std::collections::BTreeMap;
use std::path::PathBuf;

/// Generate domain layer tasks (ValueObjects, Entities, DomainEvents, Repositories, Enums)
pub fn append_domain_tasks(
    tasks: &mut Vec<RenderTask>,
    contexts: &[&ManifestContext],
    conventions: &ManifestConventions,
    template_index: &BTreeMap<ArtifactKind, Vec<&TemplateMapping>>,
) -> ManifestResult<()> {
    for context in contexts {
        for aggregate in &context.aggregates {
            append_value_objects(tasks, context, aggregate, conventions, template_index)?;
            append_entities(tasks, context, aggregate, conventions, template_index)?;
            append_domain_events(tasks, context, aggregate, conventions, template_index)?;
            append_repository_interfaces(tasks, context, aggregate, conventions, template_index)?;
            append_enums(tasks, context, aggregate, conventions, template_index)?;
        }
    }
    Ok(())
}

fn append_value_objects(
    tasks: &mut Vec<RenderTask>,
    context: &ManifestContext,
    aggregate: &crate::models::ManifestAggregate,
    conventions: &ManifestConventions,
    template_index: &BTreeMap<ArtifactKind, Vec<&TemplateMapping>>,
) -> ManifestResult<()> {
    if let Some(mappings) = template_index.get(&ArtifactKind::ValueObject) {
        for value_object in &aggregate.value_objects {
            for mapping in mappings {
                let data = json!({
                    "namespace": format!("{}.Domain.ValueObjects", conventions.namespace_root),
                    "contextName": context.name,
                    "name": value_object.name,
                    "fields": value_object.fields.iter().map(|f| json!({
                        "name": f.name,
                        "type": f.r#type,
                        "nullable": f.nullable,
                    })).collect::<Vec<_>>(),
                });

                let destination = PathBuf::from(
                    mapping
                        .dst
                        .replace("{context}", &context.name)
                        .replace("{name}", &value_object.name),
                );

                tasks.push(RenderTask {
                    kind: ArtifactKind::ValueObject,
                    template: mapping.template.clone(),
                    destination,
                    data,
                    note: Some(format!("ValueObject: {}", value_object.name)),
                });
            }
        }
    }
    Ok(())
}

fn append_entities(
    tasks: &mut Vec<RenderTask>,
    context: &ManifestContext,
    aggregate: &crate::models::ManifestAggregate,
    conventions: &ManifestConventions,
    template_index: &BTreeMap<ArtifactKind, Vec<&TemplateMapping>>,
) -> ManifestResult<()> {
    if let Some(mappings) = template_index.get(&ArtifactKind::Entity) {
        for entity in &aggregate.entities {
            for mapping in mappings {
                let data = json!({
                    "namespace": format!("{}.Domain.Entities", conventions.namespace_root),
                    "contextName": context.name,
                    "aggregateName": aggregate.name,
                    "name": entity.name,
                    "fields": entity.fields.iter().map(|f| json!({
                        "name": f.name,
                        "type": f.r#type,
                        "nullable": f.nullable,
                        "key": f.key,
                    })).collect::<Vec<_>>(),
                });

                let destination = PathBuf::from(
                    mapping
                        .dst
                        .replace("{context}", &context.name)
                        .replace("{aggregate}", &aggregate.name)
                        .replace("{name}", &entity.name),
                );

                tasks.push(RenderTask {
                    kind: ArtifactKind::Entity,
                    template: mapping.template.clone(),
                    destination,
                    data,
                    note: Some(format!("Entity: {}", entity.name)),
                });
            }
        }
    }
    Ok(())
}

fn append_domain_events(
    tasks: &mut Vec<RenderTask>,
    context: &ManifestContext,
    aggregate: &crate::models::ManifestAggregate,
    conventions: &ManifestConventions,
    template_index: &BTreeMap<ArtifactKind, Vec<&TemplateMapping>>,
) -> ManifestResult<()> {
    if let Some(mappings) = template_index.get(&ArtifactKind::DomainEvent) {
        for event in &aggregate.domain_events {
            for mapping in mappings {
                let data = json!({
                    "namespace": format!("{}.Domain.Events", conventions.namespace_root),
                    "contextName": context.name,
                    "aggregateName": aggregate.name,
                    "name": event.name,
                });

                let destination = PathBuf::from(
                    mapping
                        .dst
                        .replace("{context}", &context.name)
                        .replace("{aggregate}", &aggregate.name)
                        .replace("{name}", &event.name),
                );

                tasks.push(RenderTask {
                    kind: ArtifactKind::DomainEvent,
                    template: mapping.template.clone(),
                    destination,
                    data,
                    note: Some(format!("DomainEvent: {}", event.name)),
                });
            }
        }
    }
    Ok(())
}

fn append_repository_interfaces(
    tasks: &mut Vec<RenderTask>,
    context: &ManifestContext,
    aggregate: &crate::models::ManifestAggregate,
    conventions: &ManifestConventions,
    template_index: &BTreeMap<ArtifactKind, Vec<&TemplateMapping>>,
) -> ManifestResult<()> {
    if let Some(repository) = &aggregate.repository {
        if let Some(mappings) = template_index.get(&ArtifactKind::RepositoryInterface) {
            for mapping in mappings {
                let data = json!({
                    "namespace": format!("{}.Domain.Repositories", conventions.namespace_root),
                    "contextName": context.name,
                    "aggregateName": aggregate.name,
                    "name": repository.name,
                    "methods": repository.methods.iter().map(|m| json!({
                        "name": m.name,
                        "returns": m.returns,
                        "args": m.args.iter().map(|a| json!({
                            "name": a.name,
                            "type": a.r#type,
                        })).collect::<Vec<_>>(),
                    })).collect::<Vec<_>>(),
                });

                let destination = PathBuf::from(
                    mapping
                        .dst
                        .replace("{context}", &context.name)
                        .replace("{aggregate}", &aggregate.name)
                        .replace("{name}", &repository.name),
                );

                tasks.push(RenderTask {
                    kind: ArtifactKind::RepositoryInterface,
                    template: mapping.template.clone(),
                    destination,
                    data,
                    note: Some(format!("Repository: {}", repository.name)),
                });
            }
        }
    }
    Ok(())
}

fn append_enums(
    tasks: &mut Vec<RenderTask>,
    context: &ManifestContext,
    aggregate: &crate::models::ManifestAggregate,
    conventions: &ManifestConventions,
    template_index: &BTreeMap<ArtifactKind, Vec<&TemplateMapping>>,
) -> ManifestResult<()> {
    if let Some(mappings) = template_index.get(&ArtifactKind::EnumType) {
        for enum_def in &aggregate.enums {
            for mapping in mappings {
                let data = json!({
                    "namespace": format!("{}.Domain.Enums", conventions.namespace_root),
                    "contextName": context.name,
                    "aggregateName": aggregate.name,
                    "name": enum_def.name,
                    "values": enum_def.values.iter().map(|v| json!({
                        "name": v.name,
                        "value": v.value,
                    })).collect::<Vec<_>>(),
                });

                let destination = PathBuf::from(
                    mapping
                        .dst
                        .replace("{context}", &context.name)
                        .replace("{aggregate}", &aggregate.name)
                        .replace("{name}", &enum_def.name),
                );

                tasks.push(RenderTask {
                    kind: ArtifactKind::EnumType,
                    template: mapping.template.clone(),
                    destination,
                    data,
                    note: Some(format!("Enum: {}", enum_def.name)),
                });
            }
        }
    }
    Ok(())
}
