/// Tasks for Application layer (use cases, services, commands, queries)
use crate::core::error::ManifestResult;
use crate::core::models::{
    ArtifactKind, ManifestContext, ManifestConventions, RenderTask, TemplateMapping,
};
use serde_json::json;
use std::collections::BTreeMap;
use std::path::PathBuf;

/// Generate application layer tasks (UseCases, Commands)
pub fn append_application_tasks(
    tasks: &mut Vec<RenderTask>,
    contexts: &[&ManifestContext],
    conventions: &ManifestConventions,
    template_index: &BTreeMap<ArtifactKind, Vec<&TemplateMapping>>,
) -> ManifestResult<()> {
    if let Some(mappings) = template_index.get(&ArtifactKind::UseCaseCommand) {
        for context in contexts {
            for use_case in &context.use_cases {
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

                    let destination = PathBuf::from(
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
    }
    Ok(())
}
