/// Artifact-specific task generation
use crate::error::{ManifestError, ManifestResult};
use crate::models::{
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
        _ => Err(ManifestError::Validation(format!(
            "artifact mode not implemented for kind '{}'",
            kind.label()
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
