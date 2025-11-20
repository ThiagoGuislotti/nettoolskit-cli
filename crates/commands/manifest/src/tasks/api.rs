/// Tasks for API layer (REST, gRPC, GraphQL, etc.)
use crate::core::error::ManifestResult;
use crate::core::models::{
    ArtifactKind, ManifestContext, ManifestConventions, RenderTask, TemplateMapping,
};
use serde_json::json;
use std::collections::BTreeMap;
use std::path::PathBuf;

/// Generate API layer tasks (Controllers, Endpoints)
pub fn append_api_tasks(
    tasks: &mut Vec<RenderTask>,
    contexts: &[&ManifestContext],
    conventions: &ManifestConventions,
    template_index: &BTreeMap<ArtifactKind, Vec<&TemplateMapping>>,
) -> ManifestResult<()> {
    if let Some(mappings) = template_index.get(&ArtifactKind::Endpoint) {
        for context in contexts {
            for use_case in &context.use_cases {
                for mapping in mappings {
                    let data = json!({
                        "namespace": format!("{}.Api.Controllers", conventions.namespace_root),
                        "contextName": context.name,
                        "useCaseName": use_case.name,
                        "name": format!("{}Controller", use_case.name),
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
                            .replace("{name}", &format!("{}Controller", use_case.name)),
                    );

                    tasks.push(RenderTask {
                        kind: ArtifactKind::Endpoint,
                        template: mapping.template.clone(),
                        destination,
                        data,
                        note: Some(format!("Controller: {}Controller", use_case.name)),
                    });
                }
            }
        }
    }
    Ok(())
}
