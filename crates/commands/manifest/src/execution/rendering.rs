/// Template rendering utilities
///
/// This module integrates with the templating crate's TemplateEngine.
/// NO duplication - delegates all rendering to nettoolskit-templating.
use crate::core::error::{ManifestError, ManifestResult};
use crate::core::models::*;
use nettoolskit_templating::{TemplateEngine, TemplateResolver};
use serde_json::{Map, Value};
use std::path::Path;

/// Render a template with given data using the shared TemplateEngine
///
/// This function delegates to nettoolskit-templating to avoid duplication.
pub async fn render_template(
    templates_root: &Path,
    template_path: &str,
    data: &Value,
    insert_todo: bool,
) -> ManifestResult<String> {
    // Use shared TemplateEngine from templating crate
    let engine = TemplateEngine::new().with_todo_insertion(insert_todo);
    let resolver = TemplateResolver::new(templates_root);

    // Resolve template path
    let full_path =
        resolver
            .resolve(template_path)
            .await
            .map_err(|_| ManifestError::TemplateNotFound {
                path: template_path.to_string(),
            })?;

    // Render using shared engine (no duplication!)
    engine
        .render_from_file(&full_path, data)
        .await
        .map_err(|err| ManifestError::TemplateRenderError {
            template: template_path.to_string(),
            reason: err.to_string(),
        })
}

/// Build solution stub (minimal .sln file)
#[allow(dead_code)]
pub fn build_solution_stub(_name: &str) -> String {
    r#"
Microsoft Visual Studio Solution File, Format Version 12.00
# Visual Studio Version 17
VisualStudioVersion = 17.0.31903.59
MinimumVisualStudioVersion = 10.0.40219.1
Project("{2150E333-8FDC-42A3-9474-1A3956D46DE8}") = "Solution Items", "Solution Items", "{9A19103F-16F7-4668-BE54-9A1E7A4F7556}"
EndProject
Global
    GlobalSection(SolutionConfigurationPlatforms) = preSolution
        Debug|Any CPU = Debug|Any CPU
        Release|Any CPU = Release|Any CPU
    EndGlobalSection
    GlobalSection(ProjectConfigurationPlatforms) = postSolution
    EndGlobalSection
EndGlobal
"#
    .trim()
    .to_string()
}

/// Build project stub (minimal .csproj file)
#[allow(dead_code)]
pub fn build_project_stub(name: &str, target_framework: &str, author: &str) -> String {
    format!(
        r#"<Project Sdk="Microsoft.NET.Sdk">
  <PropertyGroup>
    <TargetFramework>{target_framework}</TargetFramework>
    <ImplicitUsings>enable</ImplicitUsings>
    <Nullable>enable</Nullable>
    <Authors>{author}</Authors>
    <Company>{author}</Company>
    <Product>{name}</Product>
  </PropertyGroup>
</Project>
"#,
        target_framework = target_framework,
        author = author,
        name = name
    )
}

/// Build project payload for rendering
#[allow(dead_code)]
pub fn build_project_payload(manifest: &ManifestDocument, project: &ManifestProject) -> Value {
    let mut map = Map::new();
    map.insert("name".to_string(), Value::String(project.name.clone()));
    map.insert(
        "namespace_root".to_string(),
        Value::String(manifest.conventions.namespace_root.clone()),
    );
    map.insert(
        "target_framework".to_string(),
        Value::String(manifest.conventions.target_framework.clone()),
    );
    if let Some(author) = &manifest.meta.author {
        map.insert("author".to_string(), Value::String(author.clone()));
    }
    Value::Object(map)
}

/// Normalize line endings (CRLF → LF) for comparison
pub fn normalize_line_endings(s: &str) -> String {
    s.replace("\r\n", "\n")
}
