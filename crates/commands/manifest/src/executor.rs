/// Manifest execution orchestrator (thin layer)
use crate::error::{ManifestError, ManifestResult};
use crate::models::{ApplyModeKind, ExecutionSummary, ManifestDocument};
use crate::parser::ManifestParser;
use std::path::{Path, PathBuf};

/// Configuration for manifest execution
#[derive(Debug, Clone)]
pub struct ExecutionConfig {
    pub manifest_path: PathBuf,
    pub output_root: PathBuf,
    pub dry_run: bool,
}

/// Executor for manifest operations (orchestrator)
pub struct ManifestExecutor;

impl ManifestExecutor {
    /// Create new executor
    pub fn new() -> Self {
        Self
    }

    /// Execute manifest (async entry point)
    pub async fn execute(&self, config: ExecutionConfig) -> ManifestResult<ExecutionSummary> {
        // Parse and validate manifest
        let manifest = ManifestParser::from_file(&config.manifest_path)?;
        ManifestParser::validate(&manifest)?;

        // Execute manifest application
        Self::execute_async(manifest, config).await
    }

    /// Locate templates directory relative to manifest
    fn locate_templates_root(manifest_path: &Path) -> ManifestResult<PathBuf> {
        let manifest_dir = manifest_path
            .parent()
            .ok_or_else(|| ManifestError::Other("manifest has no parent directory".to_string()))?;

        // Search for templates directory
        let candidates = vec![
            Some(manifest_dir.join("templates")),
            Some(manifest_dir.join(".templates")),
            manifest_dir.parent().map(|p| p.join("templates")),
        ];

        for candidate in candidates.into_iter().flatten() {
            if candidate.exists() && candidate.is_dir() {
                return Ok(candidate);
            }
        }

        Err(ManifestError::TemplateNotFound {
            path: "templates directory not found near manifest".to_string(),
        })
    }

    /// Main async execution logic - orchestrates specialized modules
    async fn execute_async(
        manifest: ManifestDocument,
        config: ExecutionConfig,
    ) -> ManifestResult<ExecutionSummary> {
        use crate::models::{FileChange, FileChangeKind, ManifestCollisionPolicy};
        use crate::rendering::{normalize_line_endings, render_template};
        use std::fs;

        let mut summary = ExecutionSummary::default();

        // Add metadata to summary
        summary
            .notes
            .push(format!("Applying manifest kind {:?}", manifest.kind));
        summary.notes.push(format!(
            "Namespace root: {}",
            manifest.conventions.namespace_root
        ));
        if let Some(description) = &manifest.meta.description {
            summary
                .notes
                .push(format!("Manifest description: {}", description));
        }

        // Locate templates
        let templates_root = Self::locate_templates_root(&config.manifest_path)?;

        // Setup paths
        let solution_root = config.output_root.join(&manifest.solution.root);
        summary
            .notes
            .push(format!("Solution root: {}", solution_root.display()));

        // Validate solution root
        if !solution_root.exists() {
            if manifest.guards.require_existing_projects {
                return Err(ManifestError::Validation(format!(
                    "solution root not found: {}",
                    solution_root.display()
                )));
            }
            crate::files::ensure_directory(
                &solution_root,
                config.dry_run,
                &mut summary,
                "solution root",
            )?;
        }

        // Collect render tasks (delegated to tasks module)
        let tasks = Self::collect_render_tasks(&manifest)?;

        summary
            .notes
            .push(format!("Collected {} render task(s)", tasks.len()));

        // Render and build file changes
        let policy = manifest
            .conventions
            .policy
            .collision
            .unwrap_or(ManifestCollisionPolicy::Fail);
        let insert_todo = manifest.conventions.policy.insert_todo_when_missing;
        let mut changes = Vec::new();

        for task in tasks {
            let rendered =
                render_template(&templates_root, &task.template, &task.data, insert_todo).await?;
            let absolute_path = config.output_root.join(&task.destination);

            if absolute_path.exists() {
                match policy {
                    ManifestCollisionPolicy::Fail => {
                        return Err(ManifestError::Validation(format!(
                            "collision detected for {}; adjust policy.collision to overwrite to proceed",
                            absolute_path.display()
                        )));
                    }
                    ManifestCollisionPolicy::Overwrite => {
                        let existing = fs::read_to_string(&absolute_path)?;

                        if normalize_line_endings(&existing) == normalize_line_endings(&rendered) {
                            summary.skipped.push((
                                absolute_path.clone(),
                                format!("unchanged {}", task.kind.label()),
                            ));
                            continue;
                        }

                        changes.push(FileChange {
                            path: absolute_path,
                            content: rendered,
                            kind: FileChangeKind::Update,
                            note: task.note.clone(),
                        });
                    }
                }
            } else {
                changes.push(FileChange {
                    path: absolute_path,
                    content: rendered,
                    kind: FileChangeKind::Create,
                    note: task.note.clone(),
                });
            }
        }

        // Execute file operations (delegated to files module)
        crate::files::execute_plan(changes, config.dry_run, &mut summary)?;

        Ok(summary)
    }

    /// Collect render tasks based on manifest mode (delegates to tasks module)
    fn collect_render_tasks(
        manifest: &ManifestDocument,
    ) -> ManifestResult<Vec<crate::models::RenderTask>> {
        use std::collections::BTreeSet;

        let mut tasks = Vec::new();
        let template_index = manifest.templates.index_by_artifact();

        match manifest.apply.mode {
            ApplyModeKind::Artifact => {
                let artifact_cfg = manifest.apply.artifact.as_ref().ok_or_else(|| {
                    ManifestError::Validation("apply.artifact section missing".to_string())
                })?;

                let kind = crate::models::ArtifactKind::parse_kind(&artifact_cfg.kind);
                let mappings = template_index.get(&kind).ok_or_else(|| {
                    ManifestError::Validation(format!(
                        "no template mapping found for artifact '{}'",
                        kind.label()
                    ))
                })?;

                let contexts =
                    Self::select_contexts(&manifest.contexts, artifact_cfg.context.as_deref());
                if contexts.is_empty() {
                    return Err(ManifestError::Validation(
                        "no matching contexts found for artifact apply mode".to_string(),
                    ));
                }

                // Delegate to tasks::artifact module
                crate::tasks::append_artifact_tasks(
                    &mut tasks,
                    &contexts,
                    &manifest.conventions,
                    &kind,
                    artifact_cfg.name.as_deref(),
                    mappings,
                )?;
            }
            ApplyModeKind::Feature => {
                let feature_cfg = manifest.apply.feature.as_ref().ok_or_else(|| {
                    ManifestError::Validation("apply.feature section missing".to_string())
                })?;

                let contexts =
                    Self::select_contexts(&manifest.contexts, feature_cfg.context.as_deref());
                if contexts.is_empty() {
                    return Err(ManifestError::Validation(
                        "no matching contexts found for feature apply mode".to_string(),
                    ));
                }

                let includes: BTreeSet<String> = feature_cfg
                    .include
                    .iter()
                    .map(|value| value.trim().to_lowercase())
                    .collect();

                // Delegate to specialized task modules
                if includes.contains("domain") {
                    crate::tasks::append_domain_tasks(
                        &mut tasks,
                        &contexts,
                        &manifest.conventions,
                        &template_index,
                    )?;
                }

                if includes.contains("application") {
                    crate::tasks::append_application_tasks(
                        &mut tasks,
                        &contexts,
                        &manifest.conventions,
                        &template_index,
                    )?;
                }

                if includes.contains("api") {
                    crate::tasks::append_api_tasks(
                        &mut tasks,
                        &contexts,
                        &manifest.conventions,
                        &template_index,
                    )?;
                }
            }
            ApplyModeKind::Layer => {
                let layer_cfg = manifest.apply.layer.as_ref().ok_or_else(|| {
                    ManifestError::Validation("apply.layer section missing".to_string())
                })?;

                let contexts: Vec<&crate::models::ManifestContext> =
                    manifest.contexts.iter().collect();
                let includes: BTreeSet<String> = layer_cfg
                    .include
                    .iter()
                    .map(|value| value.trim().to_lowercase())
                    .collect();

                // Delegate to specialized task modules
                if includes.contains("domain") {
                    crate::tasks::append_domain_tasks(
                        &mut tasks,
                        &contexts,
                        &manifest.conventions,
                        &template_index,
                    )?;
                }
            }
        }

        Ok(tasks)
    }

    /// Select contexts by name
    fn select_contexts<'a>(
        contexts: &'a [crate::models::ManifestContext],
        name: Option<&str>,
    ) -> Vec<&'a crate::models::ManifestContext> {
        match name {
            Some(target) => contexts
                .iter()
                .filter(|ctx| ctx.name.eq_ignore_ascii_case(target))
                .collect(),
            None => contexts.iter().collect(),
        }
    }
}

impl Default for ManifestExecutor {
    fn default() -> Self {
        Self::new()
    }
}
