/// Individual artifact generation tasks
use crate::core::error::{ManifestError, ManifestResult};
use crate::core::models::{
    ArtifactKind, ManifestContext, ManifestConventions, RenderTask, TemplateMapping,
};
use serde_json::json;

/// Generate tasks for a specific artifact by name
pub fn append_artifact_tasks(
    tasks: &mut Vec<RenderTask>,
    contexts: &[&ManifestContext],
    conventions: &ManifestConventions,
    kind: &ArtifactKind,
    name: Option<&str>,
    mappings: &[&TemplateMapping],
) -> ManifestResult<()> {
    match kind {
        ArtifactKind::ValueObject => {
            append_value_object_artifact(tasks, contexts, conventions, name, mappings)
        }
        ArtifactKind::Entity => {
            append_entity_artifact(tasks, contexts, conventions, name, mappings)
        }
        ArtifactKind::UseCaseCommand => {
            append_use_case_artifact(tasks, contexts, conventions, name, mappings)
        }
        ArtifactKind::DomainEvent => {
            append_domain_event_artifact(tasks, contexts, conventions, name, mappings)
        }
        ArtifactKind::RepositoryInterface => {
            append_repository_artifact(tasks, contexts, conventions, name, mappings)
        }
        ArtifactKind::EnumType => {
            append_enum_artifact(tasks, contexts, conventions, name, mappings)
        }
        ArtifactKind::Endpoint => {
            append_endpoint_artifact(tasks, contexts, conventions, name, mappings)
        }
        ArtifactKind::Unknown(ref kind_label) => Err(ManifestError::Validation(format!(
            "artifact mode not implemented for unknown kind '{kind_label}'"
        ))),
    }
}

fn append_value_object_artifact(
    tasks: &mut Vec<RenderTask>,
    contexts: &[&ManifestContext],
    conventions: &ManifestConventions,
    name: Option<&str>,
    mappings: &[&TemplateMapping],
) -> ManifestResult<()> {
    for context in contexts {
        for aggregate in &context.aggregates {
            for value_object in &aggregate.value_objects {
                if let Some(target_name) = name {
                    if !value_object.name.eq_ignore_ascii_case(target_name) {
                        continue;
                    }
                }

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

                    let destination = std::path::PathBuf::from(
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
    }
    Ok(())
}

fn append_entity_artifact(
    tasks: &mut Vec<RenderTask>,
    contexts: &[&ManifestContext],
    conventions: &ManifestConventions,
    name: Option<&str>,
    mappings: &[&TemplateMapping],
) -> ManifestResult<()> {
    for context in contexts {
        for aggregate in &context.aggregates {
            for entity in &aggregate.entities {
                if let Some(target_name) = name {
                    if !entity.name.eq_ignore_ascii_case(target_name) {
                        continue;
                    }
                }

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

                    let destination = std::path::PathBuf::from(
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
    }
    Ok(())
}
fn append_domain_event_artifact(
    tasks: &mut Vec<RenderTask>,
    contexts: &[&ManifestContext],
    conventions: &ManifestConventions,
    name: Option<&str>,
    mappings: &[&TemplateMapping],
) -> ManifestResult<()> {
    for context in contexts {
        for aggregate in &context.aggregates {
            for event in &aggregate.domain_events {
                if let Some(target_name) = name {
                    if !event.name.eq_ignore_ascii_case(target_name) {
                        continue;
                    }
                }

                for mapping in mappings {
                    let data = json!({
                        "namespace": format!("{}.Domain.Events", conventions.namespace_root),
                        "contextName": context.name,
                        "aggregateName": aggregate.name,
                        "name": event.name,
                    });

                    let destination = std::path::PathBuf::from(
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
    }
    Ok(())
}

fn append_repository_artifact(
    tasks: &mut Vec<RenderTask>,
    contexts: &[&ManifestContext],
    conventions: &ManifestConventions,
    name: Option<&str>,
    mappings: &[&TemplateMapping],
) -> ManifestResult<()> {
    for context in contexts {
        for aggregate in &context.aggregates {
            if let Some(repository) = &aggregate.repository {
                if let Some(target_name) = name {
                    if !repository.name.eq_ignore_ascii_case(target_name) {
                        continue;
                    }
                }

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

                    let destination = std::path::PathBuf::from(
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
    }
    Ok(())
}

fn append_enum_artifact(
    tasks: &mut Vec<RenderTask>,
    contexts: &[&ManifestContext],
    conventions: &ManifestConventions,
    name: Option<&str>,
    mappings: &[&TemplateMapping],
) -> ManifestResult<()> {
    for context in contexts {
        for aggregate in &context.aggregates {
            for enum_def in &aggregate.enums {
                if let Some(target_name) = name {
                    if !enum_def.name.eq_ignore_ascii_case(target_name) {
                        continue;
                    }
                }

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

                    let destination = std::path::PathBuf::from(
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
    }
    Ok(())
}

fn append_endpoint_artifact(
    tasks: &mut Vec<RenderTask>,
    contexts: &[&ManifestContext],
    conventions: &ManifestConventions,
    name: Option<&str>,
    mappings: &[&TemplateMapping],
) -> ManifestResult<()> {
    for context in contexts {
        for use_case in &context.use_cases {
            if let Some(target_name) = name {
                if !use_case.name.eq_ignore_ascii_case(target_name) {
                    continue;
                }
            }

            for mapping in mappings {
                let controller_name = format!("{}Controller", use_case.name);
                let data = json!({
                    "namespace": format!("{}.Api.Controllers", conventions.namespace_root),
                    "contextName": context.name,
                    "useCaseName": use_case.name,
                    "name": controller_name,
                    "input": use_case.input.iter().map(|f| json!({
                        "name": f.name,
                        "type": f.r#type,
                        "nullable": f.nullable,
                    })).collect::<Vec<_>>(),
                    "output": use_case.output.iter().map(|f| json!({
                        "name": f.name,
                        "type": f.r#type,
                        "nullable": f.nullable,
                    })).collect::<Vec<_>>(),
                });

                let destination = std::path::PathBuf::from(
                    mapping
                        .dst
                        .replace("{context}", &context.name)
                        .replace("{name}", &controller_name),
                );

                tasks.push(RenderTask {
                    kind: ArtifactKind::Endpoint,
                    template: mapping.template.clone(),
                    destination,
                    data,
                    note: Some(format!("Controller: {controller_name}")),
                });
            }
        }
    }
    Ok(())
}
fn append_use_case_artifact(
    tasks: &mut Vec<RenderTask>,
    contexts: &[&ManifestContext],
    conventions: &ManifestConventions,
    name: Option<&str>,
    mappings: &[&TemplateMapping],
) -> ManifestResult<()> {
    for context in contexts {
        for use_case in &context.use_cases {
            if let Some(target_name) = name {
                if !use_case.name.eq_ignore_ascii_case(target_name) {
                    continue;
                }
            }

            for mapping in mappings {
                let data = json!({
                    "namespace": format!("{}.Application.UseCases", conventions.namespace_root),
                    "contextName": context.name,
                    "name": use_case.name,
                    "type": use_case.use_case_type,
                    "input": use_case.input.iter().map(|f| json!({
                        "name": f.name,
                        "type": f.r#type,
                        "nullable": f.nullable,
                    })).collect::<Vec<_>>(),
                    "output": use_case.output.iter().map(|f| json!({
                        "name": f.name,
                        "type": f.r#type,
                        "nullable": f.nullable,
                    })).collect::<Vec<_>>(),
                });

                let destination = std::path::PathBuf::from(
                    mapping
                        .dst
                        .replace("{context}", &context.name)
                        .replace("{name}", &use_case.name),
                );

                tasks.push(RenderTask {
                    kind: ArtifactKind::UseCaseCommand,
                    template: mapping.template.clone(),
                    destination,
                    data,
                    note: Some(format!("UseCase: {}", use_case.name)),
                });
            }
        }
    }
    Ok(())
}
