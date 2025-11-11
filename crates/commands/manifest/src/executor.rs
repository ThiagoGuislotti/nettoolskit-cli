/// Manifest execution logic
use crate::error::{ManifestError, ManifestResult};
use crate::models::{ExecutionSummary, ManifestDocument};
use crate::parser::ManifestParser;
use std::path::PathBuf;

/// Configuration for manifest execution
#[derive(Debug, Clone)]
pub struct ExecutionConfig {
    pub manifest_path: PathBuf,
    pub output_root: PathBuf,
    pub dry_run: bool,
}

/// Executor for manifest operations
pub struct ManifestExecutor;

impl ManifestExecutor {
    /// Create new executor
    pub fn new() -> Self {
        Self
    }

    /// Execute manifest (async)
    pub async fn execute(&self, config: ExecutionConfig) -> ManifestResult<ExecutionSummary> {
        // Parse manifest
        let manifest = ManifestParser::from_file(&config.manifest_path)?;
        ManifestParser::validate(&manifest)?;

        // Execute manifest application
        Self::execute_async(manifest, config).await
    }

    /// Locate templates directory relative to manifest
    fn locate_templates_root(manifest_path: &PathBuf) -> ManifestResult<PathBuf> {
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

    /// Asynchronous execution - full manifest application logic
    async fn execute_async(
        manifest: ManifestDocument,
        config: ExecutionConfig,
    ) -> ManifestResult<ExecutionSummary> {
        use crate::models::{FileChange, FileChangeKind, ManifestCollisionPolicy, MissingProjectAction};
        use crate::rendering::{build_project_payload, build_project_stub, build_solution_stub, normalize_line_endings, render_template};
        use std::fs;

        let mut summary = ExecutionSummary::default();

        // Add initial notes
        summary.notes.push(format!("Applying manifest kind {:?}", manifest.kind));
        summary.notes.push(format!(
            "Namespace root: {}",
            manifest.conventions.namespace_root
        ));

        if let Some(description) = &manifest.meta.description {
            summary.notes.push(format!("Manifest description: {}", description));
        }

        // Locate templates root
        let templates_root = Self::locate_templates_root(&config.manifest_path)?;

        // Setup solution paths
        let solution_root = config.output_root.join(&manifest.solution.root);
        let solution_file = solution_root.join(&manifest.solution.sln_file);

        summary.notes.push(format!("Solution root: {}", solution_root.display()));

        let insert_todo = manifest.conventions.policy.insert_todo_when_missing;

        // Validate solution root existence
        if !solution_root.exists() {
            if manifest.guards.require_existing_projects {
                return Err(ManifestError::Validation(format!(
                    "solution root not found: {}",
                    solution_root.display()
                )));
            }
            Self::ensure_directory(
                &solution_root,
                config.dry_run,
                &mut summary,
                "solution root",
            )?;
        }

        let mut changes: Vec<FileChange> = Vec::new();

        // Handle solution file
        if solution_file.exists() {
            summary.skipped.push((
                solution_file.clone(),
                "solution already exists".to_string(),
            ));
        } else {
            if manifest.guards.require_existing_projects {
                return Err(ManifestError::Validation(format!(
                    "solution file missing: {}",
                    solution_file.display()
                )));
            }

            changes.push(FileChange {
                path: solution_file.clone(),
                content: build_solution_stub(&manifest.meta.name),
                kind: FileChangeKind::Create,
                note: Some("solution scaffold".to_string()),
            });
        }

        // Process projects
        for project in manifest.projects.values() {
            let project_dir = solution_root.join(&project.path);
            let csproj_path = project_dir.join(format!("{}.csproj", project.name));

            if !project_dir.exists() || !csproj_path.exists() {
                match (
                    manifest.guards.require_existing_projects,
                    manifest.guards.on_missing_project,
                ) {
                    (true, Some(MissingProjectAction::Skip)) => {
                        summary.skipped.push((
                            csproj_path.clone(),
                            "project missing (skipped due to guard configuration)".to_string(),
                        ));
                        continue;
                    }
                    (true, _) => {
                        return Err(ManifestError::Validation(format!(
                            "project missing: {}",
                            csproj_path.display()
                        )));
                    }
                    _ => {}
                }
            }

            if !project_dir.exists() {
                Self::ensure_directory(&project_dir, config.dry_run, &mut summary, "project root")?;
            }

            if csproj_path.exists() {
                summary.skipped.push((csproj_path, "project already exists".to_string()));
            } else {
                let content = if let Some(template_path) = project.kind.template_path() {
                    let payload = build_project_payload(&manifest, project);
                    render_template(&templates_root, template_path, &payload, insert_todo).await?
                } else {
                    build_project_stub(
                        &project.name,
                        &manifest.conventions.target_framework,
                        manifest
                            .meta
                            .author
                            .as_deref()
                            .unwrap_or(&manifest.meta.name),
                    )
                };

                changes.push(FileChange {
                    path: csproj_path,
                    content,
                    kind: FileChangeKind::Create,
                    note: Some(format!("project scaffold ({:?})", project.kind)),
                });
            }
        }

        // Collect and process render tasks
        let policy = manifest
            .conventions
            .policy
            .collision
            .unwrap_or(ManifestCollisionPolicy::Fail);

        let tasks = Self::collect_render_tasks(&manifest)?;

        for task in tasks {
            let rendered = render_template(&templates_root, &task.template, &task.data, insert_todo).await?;
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

        // Execute the plan
        Self::execute_plan(changes, config.dry_run, &mut summary)?;

        Ok(summary)
    }

    /// Ensure directory exists
    fn ensure_directory(
        path: &PathBuf,
        dry_run: bool,
        summary: &mut ExecutionSummary,
        note: &str,
    ) -> ManifestResult<()> {
        use std::fs;

        if path.exists() {
            return Ok(());
        }

        if dry_run {
            summary.notes.push(format!(
                "create directory (dry-run): {} ({})",
                path.display(),
                note
            ));
            return Ok(());
        }

        fs::create_dir_all(path)?;
        summary.notes.push(format!("Created directory: {} ({})", path.display(), note));
        Ok(())
    }

    /// Execute file change plan
    fn execute_plan(
        changes: Vec<crate::models::FileChange>,
        dry_run: bool,
        summary: &mut ExecutionSummary,
    ) -> ManifestResult<()> {
        use crate::models::FileChangeKind;
        use std::fs;

        for change in changes {
            if dry_run {
                match change.kind {
                    FileChangeKind::Create => {
                        summary.notes.push(format!(
                            "would create: {} ({})",
                            change.path.display(),
                            change.note.as_deref().unwrap_or("no note")
                        ));
                    }
                    FileChangeKind::Update => {
                        summary.notes.push(format!(
                            "would update: {} ({})",
                            change.path.display(),
                            change.note.as_deref().unwrap_or("no note")
                        ));
                    }
                }
                continue;
            }

            // Ensure parent directory exists
            if let Some(parent) = change.path.parent() {
                fs::create_dir_all(parent)?;
            }

            // Write file
            fs::write(&change.path, &change.content)?;

            match change.kind {
                FileChangeKind::Create => {
                    summary.created.push(change.path.clone());
                }
                FileChangeKind::Update => {
                    summary.updated.push(change.path.clone());
                }
            }
        }

        Ok(())
    }

    /// Collect render tasks based on manifest mode
    #[allow(unused_variables)]
    fn collect_render_tasks(manifest: &ManifestDocument) -> ManifestResult<Vec<crate::models::RenderTask>> {
        use crate::models::{ArtifactKind, ApplyModeKind};
        use std::collections::BTreeSet;

        let mut tasks = Vec::new();
        let template_index = manifest.templates.index_by_artifact();

        match manifest.apply.mode {
            ApplyModeKind::Artifact => {
                let artifact_cfg = manifest.apply.artifact.as_ref().ok_or_else(|| {
                    ManifestError::Validation("apply.artifact section missing".to_string())
                })?;

                let kind = ArtifactKind::from_str(&artifact_cfg.kind);
                let mappings = template_index.get(&kind).ok_or_else(|| {
                    ManifestError::Validation(format!(
                        "no template mapping found for artifact '{}'",
                        kind.label()
                    ))
                })?;

                // TODO: Implement artifact-specific task building
                // This will require porting find_* and build_*_task functions
            }
            ApplyModeKind::Feature => {
                let feature_cfg = manifest.apply.feature.as_ref().ok_or_else(|| {
                    ManifestError::Validation("apply.feature section missing".to_string())
                })?;

                let contexts = Self::select_contexts(&manifest.contexts, feature_cfg.context.as_deref());
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

                if includes.contains("domain") {
                    Self::append_domain_tasks(&mut tasks, &contexts, &manifest.conventions, &template_index)?;
                }

                if includes.contains("application") {
                    Self::append_application_tasks(&mut tasks, &contexts, &manifest.conventions, &template_index)?;
                }

                if includes.contains("api") {
                    Self::append_api_tasks(&mut tasks, &contexts, &manifest.conventions, &template_index)?;
                }
            }
            ApplyModeKind::Layer => {
                let layer_cfg = manifest.apply.layer.as_ref().ok_or_else(|| {
                    ManifestError::Validation("apply.layer section missing".to_string())
                })?;

                let contexts: Vec<&crate::models::ManifestContext> = manifest.contexts.iter().collect();
                let includes: BTreeSet<String> = layer_cfg
                    .include
                    .iter()
                    .map(|value| value.trim().to_lowercase())
                    .collect();

                if includes.contains("domain") {
                    Self::append_domain_tasks(&mut tasks, &contexts, &manifest.conventions, &template_index)?;
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

    /// Append domain tasks (placeholder)
    fn append_domain_tasks(
        _tasks: &mut Vec<crate::models::RenderTask>,
        _contexts: &[&crate::models::ManifestContext],
        _conventions: &crate::models::ManifestConventions,
        _template_index: &std::collections::BTreeMap<crate::models::ArtifactKind, Vec<&crate::models::TemplateMapping>>,
    ) -> ManifestResult<()> {
        // TODO: Port full domain task logic from backup
        Ok(())
    }

    /// Append application tasks (placeholder)
    fn append_application_tasks(
        _tasks: &mut Vec<crate::models::RenderTask>,
        _contexts: &[&crate::models::ManifestContext],
        _conventions: &crate::models::ManifestConventions,
        _template_index: &std::collections::BTreeMap<crate::models::ArtifactKind, Vec<&crate::models::TemplateMapping>>,
    ) -> ManifestResult<()> {
        // TODO: Port full application task logic from backup
        Ok(())
    }

    /// Append API tasks (placeholder)
    fn append_api_tasks(
        _tasks: &mut Vec<crate::models::RenderTask>,
        _contexts: &[&crate::models::ManifestContext],
        _conventions: &crate::models::ManifestConventions,
        _template_index: &std::collections::BTreeMap<crate::models::ArtifactKind, Vec<&crate::models::TemplateMapping>>,
    ) -> ManifestResult<()> {
        // TODO: Port full API task logic from backup
        Ok(())
    }
}

impl Default for ManifestExecutor {
    fn default() -> Self {
        Self::new()
    }
}