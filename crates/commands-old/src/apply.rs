use crate::{CommandError, ExitStatus, Result};
use clap::Parser;
use handlebars::Handlebars;
use once_cell::sync::Lazy;
use owo_colors::OwoColorize;
use regex::Regex;
use serde::Deserialize;
use serde_json::{Map, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use tokio::task;
use tracing::info;
use walkdir::WalkDir;

static PLACEHOLDER_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\{([A-Za-z0-9_]+)\}").expect("invalid placeholder regex"));

const DEFAULT_MANIFEST_NAME: &str = "ntk-manifest.yml";
const DEFAULT_OUTPUT_DIR: &str = "target/ntk-output";

#[derive(Debug, Parser, Clone)]
pub struct ApplyArgs {
    #[clap(value_name = "MANIFEST", default_value = DEFAULT_MANIFEST_NAME)]
    pub manifest: PathBuf,
    #[clap(short, long = "output", value_name = "DIR", default_value = DEFAULT_OUTPUT_DIR)]
    pub output: PathBuf,
    #[clap(short = 'n', long, action = clap::ArgAction::SetTrue)]
    pub dry_run: bool,
}

impl Default for ApplyArgs {
    fn default() -> Self {
        Self {
            manifest: PathBuf::from(DEFAULT_MANIFEST_NAME),
            output: PathBuf::from(DEFAULT_OUTPUT_DIR),
            dry_run: false,
        }
    }
}

pub async fn run(args: ApplyArgs) -> ExitStatus {
    println!("{}", "⚡ Applying manifest".bold().yellow());
    println!("Manifest: {}", args.manifest.display().to_string().cyan());
    println!("Output: {}", args.output.display().to_string().cyan());

    if args.dry_run {
        println!("{}", "🔍 Dry run mode - no changes will be made".yellow());
    }

    let dry_run = args.dry_run;
    match execute_apply(args).await {
        Ok(summary) => {
            println!();
            summary.print(dry_run);
            println!();
            println!("{}", "✅ Manifest processed successfully".green());
            ExitStatus::Success
        }
        Err(err) => {
            println!();
            println!("{} {}", "✖".red(), err.to_string().red());
            ExitStatus::Error
        }
    }
}

async fn execute_apply(args: ApplyArgs) -> Result<ApplySummary> {
    let manifest_path = resolve_manifest_path(&args.manifest)?;
    let output_root = resolve_output_root(&args.output)?;

    let config = ApplyConfig {
        manifest_path,
        output_root,
        dry_run: args.dry_run,
    };

    task::spawn_blocking(move || apply_sync(config))
        .await
        .map_err(|err| CommandError::Runtime(format!("task join error: {err}")))?
}

fn resolve_manifest_path(manifest: &Path) -> Result<PathBuf> {
    let candidate = if manifest.is_absolute() {
        manifest.to_path_buf()
    } else if manifest.as_os_str().is_empty() || manifest == Path::new(DEFAULT_MANIFEST_NAME) {
        let cwd = std::env::current_dir()?;
        let options = [
            cwd.join(DEFAULT_MANIFEST_NAME),
            cwd.join("ntk-manifest.yaml"),
            cwd.join(".docs").join(DEFAULT_MANIFEST_NAME),
            cwd.join(".docs").join("ntk-manifest-artifact.yml"),
            cwd.join(".docs").join("ntk-manifest-feature.yml"),
            cwd.join(".docs").join("ntk-manifest-layer.yml"),
        ];
        match options.iter().find(|path| path.exists()) {
            Some(path) => path.clone(),
            None => cwd.join(DEFAULT_MANIFEST_NAME),
        }
    } else {
        std::env::current_dir()?.join(manifest)
    };

    if candidate.exists() {
        Ok(candidate)
    } else {
        Err(CommandError::Other(format!(
            "manifest not found: {}",
            candidate.display()
        )))
    }
}

fn resolve_output_root(path: &PathBuf) -> Result<PathBuf> {
    if path.exists() {
        return Ok(path.canonicalize()?);
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::create_dir_all(path)?;
    Ok(path.canonicalize()?)
}

struct ApplyConfig {
    manifest_path: PathBuf,
    output_root: PathBuf,
    dry_run: bool,
}

#[cfg(test)]
fn apply_manifest_for_tests(
    manifest_path: &Path,
    output_root: &Path,
    dry_run: bool,
) -> Result<ApplySummary> {
    let config = ApplyConfig {
        manifest_path: manifest_path.to_path_buf(),
        output_root: output_root.to_path_buf(),
        dry_run,
    };

    apply_sync(config)
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum ArtifactKind {
    ValueObject,
    Entity,
    DomainEvent,
    RepositoryInterface,
    EnumType,
    UseCaseCommand,
    Endpoint,
    Unknown(String),
}

impl ArtifactKind {
    fn from(value: &str) -> Self {
        match value {
            "value-object" => Self::ValueObject,
            "entity" => Self::Entity,
            "domain-event" => Self::DomainEvent,
            "repository-interface" => Self::RepositoryInterface,
            "enum" => Self::EnumType,
            "usecase-command" => Self::UseCaseCommand,
            "endpoint" => Self::Endpoint,
            other => Self::Unknown(other.to_string()),
        }
    }

    fn label(&self) -> &str {
        match self {
            Self::ValueObject => "value-object",
            Self::Entity => "entity",
            Self::DomainEvent => "domain-event",
            Self::RepositoryInterface => "repository-interface",
            Self::EnumType => "enum",
            Self::UseCaseCommand => "usecase-command",
            Self::Endpoint => "endpoint",
            Self::Unknown(value) => value.as_str(),
        }
    }
}

#[derive(Debug)]
struct RenderTask {
    kind: ArtifactKind,
    template: String,
    destination: PathBuf,
    data: Value,
    note: Option<String>,
}

#[derive(Debug)]
enum FileChangeKind {
    Create,
    Update,
}

#[derive(Debug)]
struct FileChange {
    path: PathBuf,
    content: String,
    kind: FileChangeKind,
    note: Option<String>,
}

#[derive(Default)]
struct ApplySummary {
    created: Vec<PathBuf>,
    updated: Vec<PathBuf>,
    skipped: Vec<(PathBuf, String)>,
    notes: Vec<String>,
}

impl ApplySummary {
    fn print(&self, dry_run: bool) {
        if dry_run {
            println!("{}", "Plan summary (dry-run)".bold().yellow());
        } else {
            println!("{}", "Plan summary".bold().green());
        }

        if !self.created.is_empty() {
            println!("{}", "Created files:".bold());
            for path in &self.created {
                println!("  {}", path.display());
            }
        }

        if !self.updated.is_empty() {
            println!("{}", "Updated files:".bold());
            for path in &self.updated {
                println!("  {}", path.display());
            }
        }

        if !self.skipped.is_empty() {
            println!("{}", "Skipped items:".bold());
            for (path, reason) in &self.skipped {
                println!("  {} ({})", path.display(), reason);
            }
        }

        if !self.notes.is_empty() {
            println!("{}", "Notes:".bold());
            for note in &self.notes {
                println!("  {}", note);
            }
        }

        if self.created.is_empty()
            && self.updated.is_empty()
            && self.skipped.is_empty()
            && self.notes.is_empty()
        {
            println!("{}", "No operations were scheduled.".italic().blue());
        }
    }
}
#[derive(Debug, Deserialize)]
struct ManifestDocument {
    #[serde(rename = "apiVersion")]
    api_version: String,
    kind: ManifestKind,
    meta: ManifestMeta,
    conventions: ManifestConventions,
    solution: ManifestSolution,
    #[serde(default)]
    guards: ManifestGuards,
    #[serde(default)]
    projects: BTreeMap<String, ManifestProject>,
    #[serde(default)]
    contexts: Vec<ManifestContext>,
    #[serde(default)]
    templates: ManifestTemplates,
    #[serde(default)]
    #[allow(dead_code)]
    render: ManifestRender,
    apply: ManifestApply,
}

impl ManifestDocument {
    fn from_path(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        serde_yaml::from_str(&content)
            .map_err(|err| CommandError::Other(format!("failed to parse manifest: {err}")))
    }

    fn validate(&self) -> Result<()> {
        if self.api_version != "ntk/v1" {
            return Err(CommandError::Other(format!(
                "unsupported apiVersion: {}",
                self.api_version
            )));
        }

        match self.apply.mode {
            ApplyModeKind::Artifact => {
                if self.apply.artifact.is_none() {
                    return Err(CommandError::Other(
                        "apply.artifact section is required for artifact mode".to_string(),
                    ));
                }
            }
            ApplyModeKind::Feature => {
                if self.apply.feature.is_none() {
                    return Err(CommandError::Other(
                        "apply.feature section is required for feature mode".to_string(),
                    ));
                }
            }
            ApplyModeKind::Layer => {
                if self.apply.layer.is_none() {
                    return Err(CommandError::Other(
                        "apply.layer section is required for layer mode".to_string(),
                    ));
                }
            }
        }

        if let ManifestGuards {
            require_existing_projects: true,
            ..
        } = self.guards
        {
            info!("Guard requireExistingProjects=true detected: existing projects must be present");
        }

        Ok(())
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum ManifestKind {
    Solution,
}

#[derive(Debug, Deserialize)]
struct ManifestMeta {
    name: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    author: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ManifestConventions {
    #[serde(rename = "namespaceRoot")]
    namespace_root: String,
    #[serde(rename = "targetFramework")]
    target_framework: String,
    #[serde(default)]
    policy: ManifestPolicy,
}

#[derive(Debug, Deserialize)]
struct ManifestSolution {
    root: PathBuf,
    #[serde(rename = "slnFile")]
    sln_file: PathBuf,
}

#[derive(Debug, Deserialize, Default)]
struct ManifestGuards {
    #[serde(default)]
    #[serde(rename = "requireExistingProjects")]
    require_existing_projects: bool,
    #[serde(default)]
    #[serde(rename = "onMissingProject")]
    on_missing_project: Option<MissingProjectAction>,
}

#[derive(Debug, Deserialize)]
struct ManifestProject {
    #[serde(rename = "type")]
    #[serde(default)]
    kind: ManifestProjectKind,
    name: String,
    path: PathBuf,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
enum ManifestProjectKind {
    Domain,
    Application,
    Infrastructure,
    Api,
    Worker,
    #[default]
    Unknown,
}

impl ManifestProjectKind {
    fn template_path(&self) -> Option<&'static str> {
        match self {
            Self::Domain => Some("dotnet/src/domain/domain.csproj.hbs"),
            _ => None,
        }
    }
}

#[derive(Debug, Deserialize, Default)]
struct ManifestPolicy {
    #[serde(default)]
    collision: Option<ManifestCollisionPolicy>,
    #[serde(default, rename = "insertTodoWhenMissing")]
    insert_todo_when_missing: bool,
    #[serde(default)]
    #[allow(dead_code)]
    strict: bool,
}

#[derive(Debug, Deserialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
enum ManifestCollisionPolicy {
    Fail,
    Overwrite,
}

#[derive(Debug, Deserialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
enum MissingProjectAction {
    Fail,
    Skip,
}

#[derive(Debug, Deserialize, Default)]
struct ManifestContext {
    name: String,
    #[serde(default)]
    aggregates: Vec<ManifestAggregate>,
    #[serde(default, rename = "useCases")]
    use_cases: Vec<ManifestUseCase>,
}

#[derive(Debug, Deserialize, Default)]
struct ManifestAggregate {
    name: String,
    #[serde(default, rename = "valueObjects")]
    value_objects: Vec<ManifestValueObject>,
    #[serde(default)]
    entities: Vec<ManifestEntity>,
    #[serde(default, rename = "domainEvents")]
    domain_events: Vec<ManifestDomainEvent>,
    #[serde(default)]
    repository: Option<ManifestRepository>,
    #[serde(default)]
    enums: Vec<ManifestEnum>,
}

#[derive(Debug, Deserialize, Default)]
struct ManifestValueObject {
    name: String,
    #[serde(default)]
    fields: Vec<ManifestField>,
}

#[derive(Debug, Deserialize, Default)]
struct ManifestEntity {
    name: String,
    #[serde(default)]
    fields: Vec<ManifestField>,
}

#[derive(Debug, Deserialize, Default)]
struct ManifestDomainEvent {
    name: String,
}

#[derive(Debug, Deserialize, Default)]
struct ManifestRepository {
    name: String,
    #[serde(default)]
    methods: Vec<ManifestRepositoryMethod>,
}

#[derive(Debug, Deserialize, Default)]
struct ManifestRepositoryMethod {
    name: String,
    #[serde(default)]
    args: Vec<ManifestMethodArgument>,
    #[serde(default)]
    returns: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
struct ManifestMethodArgument {
    name: String,
    #[serde(rename = "type")]
    r#type: String,
}

#[derive(Debug, Deserialize, Default)]
struct ManifestField {
    name: String,
    #[serde(rename = "type")]
    r#type: String,
    #[serde(default)]
    key: bool,
    #[serde(default)]
    nullable: bool,
    #[serde(default, rename = "columnName")]
    column_name: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
struct ManifestEnum {
    name: String,
    #[serde(default)]
    values: Vec<ManifestEnumValue>,
}

#[derive(Debug, Deserialize, Default)]
struct ManifestEnumValue {
    name: String,
    value: i32,
}

#[derive(Debug, Deserialize, Default)]
struct ManifestUseCase {
    name: String,
    #[serde(rename = "type")]
    use_case_type: String,
    #[serde(default)]
    input: Vec<ManifestField>,
    #[serde(default)]
    output: Vec<ManifestField>,
}

#[derive(Debug, Deserialize, Default)]
struct ManifestTemplates {
    #[serde(default)]
    mapping: Vec<TemplateMapping>,
}

impl ManifestTemplates {
    fn index_by_artifact(&self) -> BTreeMap<ArtifactKind, Vec<&TemplateMapping>> {
        let mut map: BTreeMap<ArtifactKind, Vec<&TemplateMapping>> = BTreeMap::new();
        for mapping in &self.mapping {
            let kind = ArtifactKind::from(mapping.artifact.as_str());
            map.entry(kind).or_default().push(mapping);
        }
        map
    }
}

#[derive(Debug, Deserialize)]
struct TemplateMapping {
    artifact: String,
    template: String,
    dst: String,
}

#[derive(Debug, Deserialize, Default)]
struct ManifestRender {
    #[serde(default)]
    #[allow(dead_code)]
    rules: Vec<RenderRule>,
}

#[derive(Debug, Deserialize)]
struct RenderRule {
    #[allow(dead_code)]
    expand: String,
    #[serde(rename = "as")]
    #[allow(dead_code)]
    alias: String,
}

#[derive(Debug, Deserialize)]
struct ManifestApply {
    mode: ApplyModeKind,
    #[serde(default)]
    artifact: Option<ApplyArtifact>,
    #[serde(default)]
    feature: Option<ApplyFeature>,
    #[serde(default)]
    layer: Option<ApplyLayer>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum ApplyModeKind {
    Artifact,
    Feature,
    Layer,
}

#[derive(Debug, Deserialize, Default)]
struct ApplyArtifact {
    kind: String,
    #[serde(default)]
    context: Option<String>,
    #[serde(default)]
    name: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
struct ApplyFeature {
    #[serde(default)]
    context: Option<String>,
    #[serde(default)]
    include: Vec<String>,
}

#[derive(Debug, Deserialize, Default)]
struct ApplyLayer {
    #[serde(default)]
    include: Vec<String>,
}

fn apply_sync(config: ApplyConfig) -> Result<ApplySummary> {
    let manifest = ManifestDocument::from_path(&config.manifest_path)?;
    manifest.validate()?;

    let templates_root = locate_templates_root(&config.manifest_path)?;
    let solution_root = config.output_root.join(&manifest.solution.root);
    let solution_file = solution_root.join(&manifest.solution.sln_file);

    let mut summary = ApplySummary::default();
    summary
        .notes
        .push(format!("Applying manifest kind {:?}", manifest.kind));
    summary.notes.push(format!(
        "Namespace root: {}",
        manifest.conventions.namespace_root
    ));
    summary
        .notes
        .push(format!("Solution root: {}", solution_root.display()));

    if let Some(description) = &manifest.meta.description {
        summary
            .notes
            .push(format!("Manifest description: {}", description));
    }

    let insert_todo = manifest.conventions.policy.insert_todo_when_missing;

    if !solution_root.exists() {
        if manifest.guards.require_existing_projects {
            return Err(CommandError::Other(format!(
                "solution root not found: {}",
                solution_root.display()
            )));
        }
        ensure_directory(
            &solution_root,
            config.dry_run,
            &mut summary,
            "solution root",
        )?;
    }

    let mut changes: Vec<FileChange> = Vec::new();

    if solution_file.exists() {
        summary
            .skipped
            .push((solution_file.clone(), "solution already exists".to_string()));
    } else {
        if manifest.guards.require_existing_projects {
            return Err(CommandError::Other(format!(
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
                    return Err(CommandError::Other(format!(
                        "project missing: {}",
                        csproj_path.display()
                    )));
                }
                _ => {}
            }
        }

        if !project_dir.exists() {
            ensure_directory(&project_dir, config.dry_run, &mut summary, "project root")?;
        }

        if csproj_path.exists() {
            summary
                .skipped
                .push((csproj_path, "project already exists".to_string()));
        } else {
            let content = if let Some(template_path) = project.kind.template_path() {
                let payload = build_project_payload(&manifest, project);
                render_template(&templates_root, template_path, &payload, insert_todo)?
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

    let policy = manifest
        .conventions
        .policy
        .collision
        .unwrap_or(ManifestCollisionPolicy::Fail);
    let tasks = collect_render_tasks(&manifest)?;
    for task in tasks {
        let rendered = render_template(&templates_root, &task.template, &task.data, insert_todo)?;
        let absolute_path = config.output_root.join(&task.destination);

        if absolute_path.exists() {
            match policy {
                ManifestCollisionPolicy::Fail => {
                    return Err(CommandError::Other(format!(
                        "collision detected for {}; adjust policy.collision to overwrite to proceed",
                        absolute_path.display()
                    )));
                }
                ManifestCollisionPolicy::Overwrite => {
                    let existing = fs::read_to_string(&absolute_path).unwrap_or_default();
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

    execute_plan(changes, config.dry_run, &mut summary)?;
    Ok(summary)
}

fn locate_templates_root(manifest_path: &Path) -> Result<PathBuf> {
    let mut current = manifest_path
        .parent()
        .ok_or_else(|| CommandError::Other("manifest path has no parent".to_string()))?;

    for _ in 0..8 {
        let candidate = current.join("templates");
        if candidate.exists() && candidate.is_dir() {
            return Ok(candidate);
        }

        if let Some(parent) = current.parent() {
            current = parent;
        } else {
            break;
        }
    }

    Err(CommandError::Other(format!(
        "unable to locate templates directory relative to {}",
        manifest_path.display()
    )))
}

fn ensure_directory(
    path: &Path,
    dry_run: bool,
    summary: &mut ApplySummary,
    note: &str,
) -> Result<()> {
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
    summary
        .notes
        .push(format!("Created directory: {} ({})", path.display(), note));
    Ok(())
}

fn collect_render_tasks(manifest: &ManifestDocument) -> Result<Vec<RenderTask>> {
    let mut tasks = Vec::new();
    let template_index = manifest.templates.index_by_artifact();

    match manifest.apply.mode {
        ApplyModeKind::Artifact => {
            let artifact_cfg =
                manifest.apply.artifact.as_ref().ok_or_else(|| {
                    CommandError::Other("apply.artifact section missing".to_string())
                })?;
            let kind = ArtifactKind::from(artifact_cfg.kind.as_str());
            let mappings = template_index.get(&kind).ok_or_else(|| {
                CommandError::Other(format!(
                    "no template mapping found for artifact '{}'",
                    kind.label()
                ))
            })?;

            match kind {
                ArtifactKind::EnumType => {
                    let locator = find_enum(
                        &manifest.contexts,
                        artifact_cfg.context.as_deref(),
                        artifact_cfg.name.as_deref(),
                    )?;
                    for mapping in mappings {
                        tasks.push(build_enum_task(
                            mapping,
                            &manifest.conventions,
                            locator.context,
                            locator.aggregate,
                            locator.enm,
                        )?);
                    }
                }
                ArtifactKind::ValueObject => {
                    let locator = find_value_object(
                        &manifest.contexts,
                        artifact_cfg.context.as_deref(),
                        artifact_cfg.name.as_deref(),
                    )?;
                    for mapping in mappings {
                        tasks.push(build_value_object_task(
                            mapping,
                            &manifest.conventions,
                            locator.context,
                            locator.aggregate,
                            locator.value_object,
                        )?);
                    }
                }
                ArtifactKind::Entity => {
                    let locator = find_entity(
                        &manifest.contexts,
                        artifact_cfg.context.as_deref(),
                        artifact_cfg.name.as_deref(),
                    )?;
                    for mapping in mappings {
                        tasks.push(build_entity_task(
                            mapping,
                            &manifest.conventions,
                            locator.context,
                            locator.aggregate,
                            locator.entity,
                        )?);
                    }
                }
                ArtifactKind::DomainEvent => {
                    let locator = find_domain_event(
                        &manifest.contexts,
                        artifact_cfg.context.as_deref(),
                        artifact_cfg.name.as_deref(),
                    )?;
                    for mapping in mappings {
                        tasks.push(build_domain_event_task(
                            mapping,
                            &manifest.conventions,
                            locator.context,
                            locator.aggregate,
                            locator.event,
                        )?);
                    }
                }
                ArtifactKind::RepositoryInterface => {
                    let locator =
                        find_repository(&manifest.contexts, artifact_cfg.context.as_deref())?;
                    for mapping in mappings {
                        tasks.push(build_repository_task(
                            mapping,
                            &manifest.conventions,
                            locator.context,
                            locator.aggregate,
                            locator.repository,
                        )?);
                    }
                }
                ArtifactKind::UseCaseCommand => {
                    let locator = find_use_case(
                        &manifest.contexts,
                        artifact_cfg.context.as_deref(),
                        artifact_cfg.name.as_deref(),
                    )?;
                    for mapping in mappings {
                        tasks.push(build_use_case_task(
                            mapping,
                            &manifest.conventions,
                            locator.context,
                            locator.aggregate,
                            locator.use_case,
                        )?);
                    }
                }
                ArtifactKind::Endpoint => {
                    let locator = find_use_case(
                        &manifest.contexts,
                        artifact_cfg.context.as_deref(),
                        artifact_cfg.name.as_deref(),
                    )?;
                    for mapping in mappings {
                        tasks.push(build_endpoint_task(
                            mapping,
                            &manifest.conventions,
                            locator.context,
                            locator.aggregate,
                            locator.use_case,
                        )?);
                    }
                }
                ArtifactKind::Unknown(name) => {
                    return Err(CommandError::Other(format!(
                        "unsupported artifact kind: {}",
                        name
                    )));
                }
            }
        }
        ApplyModeKind::Feature => {
            let feature_cfg =
                manifest.apply.feature.as_ref().ok_or_else(|| {
                    CommandError::Other("apply.feature section missing".to_string())
                })?;

            let contexts = select_contexts(&manifest.contexts, feature_cfg.context.as_deref());
            if contexts.is_empty() {
                return Err(CommandError::Other(
                    "no matching contexts found for feature apply mode".to_string(),
                ));
            }

            let includes: BTreeSet<String> = feature_cfg
                .include
                .iter()
                .map(|value| value.trim().to_lowercase())
                .collect();

            if includes.contains("domain") {
                append_domain_tasks(
                    &mut tasks,
                    &contexts,
                    &manifest.conventions,
                    &template_index,
                )?;
            }

            if includes.contains("application") {
                append_application_tasks(
                    &mut tasks,
                    &contexts,
                    &manifest.conventions,
                    &template_index,
                )?;
            }

            if includes.contains("api") {
                append_api_tasks(
                    &mut tasks,
                    &contexts,
                    &manifest.conventions,
                    &template_index,
                )?;
            }
        }
        ApplyModeKind::Layer => {
            let layer_cfg =
                manifest.apply.layer.as_ref().ok_or_else(|| {
                    CommandError::Other("apply.layer section missing".to_string())
                })?;

            let contexts: Vec<&ManifestContext> = manifest.contexts.iter().collect();
            let includes: BTreeSet<String> = layer_cfg
                .include
                .iter()
                .map(|value| value.trim().to_lowercase())
                .collect();

            if includes.contains("domain") {
                append_domain_tasks(
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

fn select_contexts<'a>(
    contexts: &'a [ManifestContext],
    name: Option<&str>,
) -> Vec<&'a ManifestContext> {
    match name {
        Some(target) => contexts
            .iter()
            .filter(|ctx| ctx.name.eq_ignore_ascii_case(target))
            .collect(),
        None => contexts.iter().collect(),
    }
}
fn append_domain_tasks(
    tasks: &mut Vec<RenderTask>,
    contexts: &[&ManifestContext],
    conventions: &ManifestConventions,
    template_index: &BTreeMap<ArtifactKind, Vec<&TemplateMapping>>,
) -> Result<()> {
    if let Some(mappings) = template_index.get(&ArtifactKind::EnumType) {
        for context in contexts {
            for aggregate in &context.aggregates {
                for enm in &aggregate.enums {
                    for mapping in mappings {
                        tasks.push(build_enum_task(
                            mapping,
                            conventions,
                            context,
                            aggregate,
                            enm,
                        )?);
                    }
                }
            }
        }
    }

    if let Some(mappings) = template_index.get(&ArtifactKind::ValueObject) {
        for context in contexts {
            for aggregate in &context.aggregates {
                for value_object in &aggregate.value_objects {
                    for mapping in mappings {
                        tasks.push(build_value_object_task(
                            mapping,
                            conventions,
                            context,
                            aggregate,
                            value_object,
                        )?);
                    }
                }
            }
        }
    } else if contexts.iter().any(|ctx| {
        ctx.aggregates
            .iter()
            .any(|agg| !agg.value_objects.is_empty())
    }) {
        return Err(CommandError::Other(
            "value-object definitions found but no template mapping configured".to_string(),
        ));
    }

    if let Some(mappings) = template_index.get(&ArtifactKind::Entity) {
        for context in contexts {
            for aggregate in &context.aggregates {
                for entity in &aggregate.entities {
                    for mapping in mappings {
                        tasks.push(build_entity_task(
                            mapping,
                            conventions,
                            context,
                            aggregate,
                            entity,
                        )?);
                    }
                }
            }
        }
    } else if contexts
        .iter()
        .any(|ctx| ctx.aggregates.iter().any(|agg| !agg.entities.is_empty()))
    {
        return Err(CommandError::Other(
            "entity definitions found but no template mapping configured".to_string(),
        ));
    }

    if let Some(mappings) = template_index.get(&ArtifactKind::DomainEvent) {
        for context in contexts {
            for aggregate in &context.aggregates {
                for event in &aggregate.domain_events {
                    for mapping in mappings {
                        tasks.push(build_domain_event_task(
                            mapping,
                            conventions,
                            context,
                            aggregate,
                            event,
                        )?);
                    }
                }
            }
        }
    }

    if let Some(mappings) = template_index.get(&ArtifactKind::RepositoryInterface) {
        for context in contexts {
            for aggregate in &context.aggregates {
                if let Some(repository) = &aggregate.repository {
                    for mapping in mappings {
                        tasks.push(build_repository_task(
                            mapping,
                            conventions,
                            context,
                            aggregate,
                            repository,
                        )?);
                    }
                }
            }
        }
    }

    Ok(())
}

fn append_application_tasks(
    tasks: &mut Vec<RenderTask>,
    contexts: &[&ManifestContext],
    conventions: &ManifestConventions,
    template_index: &BTreeMap<ArtifactKind, Vec<&TemplateMapping>>,
) -> Result<()> {
    let Some(mappings) = template_index.get(&ArtifactKind::UseCaseCommand) else {
        if contexts.iter().any(|ctx| !ctx.use_cases.is_empty()) {
            return Err(CommandError::Other(
                "use case commands defined but no template mapping configured".to_string(),
            ));
        }
        return Ok(());
    };

    for context in contexts {
        for use_case in &context.use_cases {
            if !use_case.use_case_type.eq_ignore_ascii_case("command") {
                continue;
            }

            for mapping in mappings {
                tasks.push(build_use_case_task(
                    mapping,
                    conventions,
                    context,
                    None,
                    use_case,
                )?);
            }
        }
    }

    Ok(())
}

fn append_api_tasks(
    tasks: &mut Vec<RenderTask>,
    contexts: &[&ManifestContext],
    conventions: &ManifestConventions,
    template_index: &BTreeMap<ArtifactKind, Vec<&TemplateMapping>>,
) -> Result<()> {
    let Some(mappings) = template_index.get(&ArtifactKind::Endpoint) else {
        if contexts.iter().any(|ctx| !ctx.use_cases.is_empty()) {
            return Err(CommandError::Other(
                "API endpoints requested but no endpoint template mapping configured".to_string(),
            ));
        }
        return Ok(());
    };

    for context in contexts {
        for use_case in &context.use_cases {
            if !use_case.use_case_type.eq_ignore_ascii_case("command") {
                continue;
            }

            for mapping in mappings {
                tasks.push(build_endpoint_task(
                    mapping,
                    conventions,
                    context,
                    None,
                    use_case,
                )?);
            }
        }
    }

    Ok(())
}

struct ValueObjectLocator<'a> {
    context: &'a ManifestContext,
    aggregate: &'a ManifestAggregate,
    value_object: &'a ManifestValueObject,
}

struct EntityLocator<'a> {
    context: &'a ManifestContext,
    aggregate: &'a ManifestAggregate,
    entity: &'a ManifestEntity,
}

struct DomainEventLocator<'a> {
    context: &'a ManifestContext,
    aggregate: &'a ManifestAggregate,
    event: &'a ManifestDomainEvent,
}

struct RepositoryLocator<'a> {
    context: &'a ManifestContext,
    aggregate: &'a ManifestAggregate,
    repository: &'a ManifestRepository,
}

struct UseCaseLocator<'a> {
    context: &'a ManifestContext,
    aggregate: Option<&'a ManifestAggregate>,
    use_case: &'a ManifestUseCase,
}

struct EnumLocator<'a> {
    context: &'a ManifestContext,
    aggregate: &'a ManifestAggregate,
    enm: &'a ManifestEnum,
}

fn find_value_object<'a>(
    contexts: &'a [ManifestContext],
    context_name: Option<&str>,
    object_name: Option<&str>,
) -> Result<ValueObjectLocator<'a>> {
    for context in contexts {
        if let Some(name) = context_name {
            if !context.name.eq_ignore_ascii_case(name) {
                continue;
            }
        }

        for aggregate in &context.aggregates {
            for value_object in &aggregate.value_objects {
                if let Some(name) = object_name {
                    if !value_object.name.eq_ignore_ascii_case(name) {
                        continue;
                    }
                }

                return Ok(ValueObjectLocator {
                    context,
                    aggregate,
                    value_object,
                });
            }
        }
    }

    Err(CommandError::Other(
        "value-object not found in manifest contexts".to_string(),
    ))
}

fn find_entity<'a>(
    contexts: &'a [ManifestContext],
    context_name: Option<&str>,
    entity_name: Option<&str>,
) -> Result<EntityLocator<'a>> {
    for context in contexts {
        if let Some(name) = context_name {
            if !context.name.eq_ignore_ascii_case(name) {
                continue;
            }
        }

        for aggregate in &context.aggregates {
            for entity in &aggregate.entities {
                if let Some(name) = entity_name {
                    if !entity.name.eq_ignore_ascii_case(name) {
                        continue;
                    }
                }

                return Ok(EntityLocator {
                    context,
                    aggregate,
                    entity,
                });
            }
        }
    }

    Err(CommandError::Other(
        "entity not found in manifest contexts".to_string(),
    ))
}

fn find_domain_event<'a>(
    contexts: &'a [ManifestContext],
    context_name: Option<&str>,
    event_name: Option<&str>,
) -> Result<DomainEventLocator<'a>> {
    for context in contexts {
        if let Some(name) = context_name {
            if !context.name.eq_ignore_ascii_case(name) {
                continue;
            }
        }

        for aggregate in &context.aggregates {
            for event in &aggregate.domain_events {
                if let Some(name) = event_name {
                    if !event.name.eq_ignore_ascii_case(name) {
                        continue;
                    }
                }

                return Ok(DomainEventLocator {
                    context,
                    aggregate,
                    event,
                });
            }
        }
    }

    Err(CommandError::Other(
        "domain-event not found in manifest contexts".to_string(),
    ))
}

fn find_repository<'a>(
    contexts: &'a [ManifestContext],
    context_name: Option<&str>,
) -> Result<RepositoryLocator<'a>> {
    for context in contexts {
        if let Some(name) = context_name {
            if !context.name.eq_ignore_ascii_case(name) {
                continue;
            }
        }

        for aggregate in &context.aggregates {
            if let Some(repository) = &aggregate.repository {
                return Ok(RepositoryLocator {
                    context,
                    aggregate,
                    repository,
                });
            }
        }
    }

    Err(CommandError::Other(
        "repository not found in manifest contexts".to_string(),
    ))
}

fn find_use_case<'a>(
    contexts: &'a [ManifestContext],
    context_name: Option<&str>,
    use_case_name: Option<&str>,
) -> Result<UseCaseLocator<'a>> {
    for context in contexts {
        if let Some(name) = context_name {
            if !context.name.eq_ignore_ascii_case(name) {
                continue;
            }
        }

        for use_case in &context.use_cases {
            if let Some(name) = use_case_name {
                if !use_case.name.eq_ignore_ascii_case(name) {
                    continue;
                }
            }

            return Ok(UseCaseLocator {
                context,
                aggregate: None,
                use_case,
            });
        }
    }

    Err(CommandError::Other(
        "use-case not found in manifest contexts".to_string(),
    ))
}

fn find_enum<'a>(
    contexts: &'a [ManifestContext],
    context_name: Option<&str>,
    enum_name: Option<&str>,
) -> Result<EnumLocator<'a>> {
    for context in contexts {
        if let Some(name) = context_name {
            if !context.name.eq_ignore_ascii_case(name) {
                continue;
            }
        }

        for aggregate in &context.aggregates {
            for enm in &aggregate.enums {
                if let Some(name) = enum_name {
                    if !enm.name.eq_ignore_ascii_case(name) {
                        continue;
                    }
                }

                return Ok(EnumLocator {
                    context,
                    aggregate,
                    enm,
                });
            }
        }
    }

    Err(CommandError::Other(
        "enum not found in manifest contexts".to_string(),
    ))
}

fn serialize_field(field: &ManifestField) -> Value {
    let mut map = Map::new();
    map.insert("name".to_string(), Value::String(field.name.clone()));
    map.insert("Name".to_string(), Value::String(field.name.clone()));
    map.insert("type".to_string(), Value::String(field.r#type.clone()));
    map.insert(
        "jsonName".to_string(),
        Value::String(to_lower_camel(&field.name)),
    );
    map.insert(
        "columnName".to_string(),
        Value::String(
            field
                .column_name
                .clone()
                .unwrap_or_else(|| field.name.clone()),
        ),
    );
    map.insert("required".to_string(), Value::Bool(!field.nullable));
    map.insert("key".to_string(), Value::Bool(field.key));
    Value::Object(map)
}

fn serialize_repository_method(method: &ManifestRepositoryMethod) -> Value {
    let mut map = Map::new();
    map.insert("name".to_string(), Value::String(method.name.clone()));
    if let Some(ret) = &method.returns {
        map.insert("returns".to_string(), Value::String(ret.clone()));
    }
    let args: Vec<Value> = method
        .args
        .iter()
        .map(|arg| {
            let mut arg_map = Map::new();
            arg_map.insert("name".to_string(), Value::String(arg.name.clone()));
            arg_map.insert("type".to_string(), Value::String(arg.r#type.clone()));
            Value::Object(arg_map)
        })
        .collect();
    map.insert("args".to_string(), Value::Array(args));
    Value::Object(map)
}

fn build_enum_task(
    mapping: &TemplateMapping,
    conventions: &ManifestConventions,
    context: &ManifestContext,
    aggregate: &ManifestAggregate,
    enm: &ManifestEnum,
) -> Result<RenderTask> {
    let mut payload = Map::new();
    payload.insert(
        "namespaceRoot".to_string(),
        Value::String(conventions.namespace_root.clone()),
    );
    payload.insert("context".to_string(), Value::String(context.name.clone()));
    payload.insert(
        "aggregate".to_string(),
        Value::String(aggregate.name.clone()),
    );
    payload.insert("name".to_string(), Value::String(enm.name.clone()));
    payload.insert("Name".to_string(), Value::String(enm.name.clone()));

    let values: Vec<Value> = enm
        .values
        .iter()
        .map(|entry| {
            let mut value_map = Map::new();
            value_map.insert("name".to_string(), Value::String(entry.name.clone()));
            value_map.insert("value".to_string(), Value::from(entry.value));
            Value::Object(value_map)
        })
        .collect();
    payload.insert("values".to_string(), Value::Array(values));

    let destination = resolve_destination(&mapping.dst, &payload)?;
    Ok(RenderTask {
        kind: ArtifactKind::EnumType,
        template: mapping.template.clone(),
        destination,
        data: Value::Object(payload),
        note: Some("enum".to_string()),
    })
}

fn build_value_object_task(
    mapping: &TemplateMapping,
    conventions: &ManifestConventions,
    context: &ManifestContext,
    aggregate: &ManifestAggregate,
    value_object: &ManifestValueObject,
) -> Result<RenderTask> {
    let mut payload = Map::new();
    payload.insert(
        "namespaceRoot".to_string(),
        Value::String(conventions.namespace_root.clone()),
    );
    payload.insert("context".to_string(), Value::String(context.name.clone()));
    payload.insert(
        "aggregate".to_string(),
        Value::String(aggregate.name.clone()),
    );
    payload.insert("name".to_string(), Value::String(value_object.name.clone()));
    payload.insert("Name".to_string(), Value::String(value_object.name.clone()));

    let fields: Vec<Value> = value_object.fields.iter().map(serialize_field).collect();
    payload.insert("fields".to_string(), Value::Array(fields));

    let destination = resolve_destination(&mapping.dst, &payload)?;
    Ok(RenderTask {
        kind: ArtifactKind::ValueObject,
        template: mapping.template.clone(),
        destination,
        data: Value::Object(payload),
        note: Some("value-object".to_string()),
    })
}

fn build_entity_task(
    mapping: &TemplateMapping,
    conventions: &ManifestConventions,
    context: &ManifestContext,
    aggregate: &ManifestAggregate,
    entity: &ManifestEntity,
) -> Result<RenderTask> {
    let mut payload = Map::new();
    payload.insert(
        "namespaceRoot".to_string(),
        Value::String(conventions.namespace_root.clone()),
    );
    payload.insert("context".to_string(), Value::String(context.name.clone()));
    payload.insert(
        "aggregate".to_string(),
        Value::String(aggregate.name.clone()),
    );
    payload.insert("name".to_string(), Value::String(entity.name.clone()));
    payload.insert("Name".to_string(), Value::String(entity.name.clone()));
    payload.insert(
        "tableName".to_string(),
        Value::String(format!("{}s", entity.name)),
    );

    let properties: Vec<Value> = entity.fields.iter().map(serialize_field).collect();
    payload.insert("properties".to_string(), Value::Array(properties));

    let destination = resolve_destination(&mapping.dst, &payload)?;
    Ok(RenderTask {
        kind: ArtifactKind::Entity,
        template: mapping.template.clone(),
        destination,
        data: Value::Object(payload),
        note: Some("entity".to_string()),
    })
}

fn build_domain_event_task(
    mapping: &TemplateMapping,
    conventions: &ManifestConventions,
    context: &ManifestContext,
    aggregate: &ManifestAggregate,
    event: &ManifestDomainEvent,
) -> Result<RenderTask> {
    let mut payload = Map::new();
    payload.insert(
        "namespaceRoot".to_string(),
        Value::String(conventions.namespace_root.clone()),
    );
    payload.insert("context".to_string(), Value::String(context.name.clone()));
    payload.insert(
        "aggregate".to_string(),
        Value::String(aggregate.name.clone()),
    );
    payload.insert("name".to_string(), Value::String(event.name.clone()));
    payload.insert("Name".to_string(), Value::String(event.name.clone()));

    let destination = resolve_destination(&mapping.dst, &payload)?;
    Ok(RenderTask {
        kind: ArtifactKind::DomainEvent,
        template: mapping.template.clone(),
        destination,
        data: Value::Object(payload),
        note: Some("domain-event".to_string()),
    })
}

fn build_repository_task(
    mapping: &TemplateMapping,
    conventions: &ManifestConventions,
    context: &ManifestContext,
    aggregate: &ManifestAggregate,
    repository: &ManifestRepository,
) -> Result<RenderTask> {
    let mut payload = Map::new();
    payload.insert(
        "namespaceRoot".to_string(),
        Value::String(conventions.namespace_root.clone()),
    );
    payload.insert("context".to_string(), Value::String(context.name.clone()));
    payload.insert(
        "aggregate".to_string(),
        Value::String(aggregate.name.clone()),
    );
    payload.insert("name".to_string(), Value::String(repository.name.clone()));
    payload.insert("Name".to_string(), Value::String(repository.name.clone()));

    let methods: Vec<Value> = repository
        .methods
        .iter()
        .map(serialize_repository_method)
        .collect();
    payload.insert("methods".to_string(), Value::Array(methods));

    let destination = resolve_destination(&mapping.dst, &payload)?;
    Ok(RenderTask {
        kind: ArtifactKind::RepositoryInterface,
        template: mapping.template.clone(),
        destination,
        data: Value::Object(payload),
        note: Some("repository-interface".to_string()),
    })
}

fn build_use_case_task(
    mapping: &TemplateMapping,
    conventions: &ManifestConventions,
    context: &ManifestContext,
    aggregate: Option<&ManifestAggregate>,
    use_case: &ManifestUseCase,
) -> Result<RenderTask> {
    let mut payload = Map::new();
    payload.insert(
        "namespaceRoot".to_string(),
        Value::String(conventions.namespace_root.clone()),
    );
    payload.insert("context".to_string(), Value::String(context.name.clone()));
    payload.insert(
        "aggregate".to_string(),
        Value::String(
            aggregate
                .map(|agg| agg.name.clone())
                .unwrap_or_else(String::new),
        ),
    );
    payload.insert("name".to_string(), Value::String(use_case.name.clone()));
    payload.insert("Name".to_string(), Value::String(use_case.name.clone()));
    payload.insert("useCase".to_string(), Value::String(use_case.name.clone()));
    payload.insert("UseCase".to_string(), Value::String(use_case.name.clone()));

    let properties: Vec<Value> = use_case.input.iter().map(serialize_field).collect();
    payload.insert("properties".to_string(), Value::Array(properties));
    payload.insert(
        "response".to_string(),
        Value::Bool(!use_case.output.is_empty()),
    );

    let destination = resolve_destination(&mapping.dst, &payload)?;
    Ok(RenderTask {
        kind: ArtifactKind::UseCaseCommand,
        template: mapping.template.clone(),
        destination,
        data: Value::Object(payload),
        note: Some("usecase-command".to_string()),
    })
}

fn build_endpoint_task(
    mapping: &TemplateMapping,
    conventions: &ManifestConventions,
    context: &ManifestContext,
    aggregate: Option<&ManifestAggregate>,
    use_case: &ManifestUseCase,
) -> Result<RenderTask> {
    let mut payload = Map::new();
    payload.insert(
        "namespaceRoot".to_string(),
        Value::String(conventions.namespace_root.clone()),
    );
    payload.insert("context".to_string(), Value::String(context.name.clone()));
    payload.insert(
        "aggregate".to_string(),
        Value::String(
            aggregate
                .map(|agg| agg.name.clone())
                .unwrap_or_else(String::new),
        ),
    );
    payload.insert("useCase".to_string(), Value::String(use_case.name.clone()));
    payload.insert("UseCase".to_string(), Value::String(use_case.name.clone()));
    payload.insert(
        "response".to_string(),
        Value::Bool(!use_case.output.is_empty()),
    );

    let destination = resolve_destination(&mapping.dst, &payload)?;
    Ok(RenderTask {
        kind: ArtifactKind::Endpoint,
        template: mapping.template.clone(),
        destination,
        data: Value::Object(payload),
        note: Some("endpoint".to_string()),
    })
}

fn render_template(
    templates_root: &Path,
    template: &str,
    data: &Value,
    insert_todo_when_missing: bool,
) -> Result<String> {
    let template_path = locate_template_file(templates_root, template)?;

    let source = fs::read_to_string(&template_path).map_err(|err| {
        CommandError::TemplateError(format!(
            "failed to read template {}: {err}",
            template_path.display()
        ))
    })?;

    let mut handlebars = Handlebars::new();
    handlebars.set_strict_mode(false);
    handlebars
        .register_template_string("tpl", source)
        .map_err(|err| {
            CommandError::TemplateError(format!(
                "failed to register template {}: {err}",
                template_path.display()
            ))
        })?;

    let mut content = handlebars.render("tpl", data).map_err(|err| {
        CommandError::TemplateError(format!(
            "failed to render template {}: {err}",
            template_path.display()
        ))
    })?;

    if insert_todo_when_missing && !content.contains("TODO") {
        content.push_str("\n// TODO: Review generated content\n");
    }

    if !content.ends_with('\n') {
        content.push('\n');
    }

    Ok(content)
}

fn locate_template_file(templates_root: &Path, template: &str) -> Result<PathBuf> {
    let direct = templates_root.join(template);
    if direct.exists() {
        return Ok(direct);
    }

    let mut parts: Vec<&str> = template.split('/').collect();
    if parts.len() > 1 && parts[0] == "dotnet" && parts[1] != "src" && parts[1] != "tests" {
        parts.insert(1, "src");
        let alt = templates_root.join(parts.join("/"));
        if alt.exists() {
            return Ok(alt);
        }
    }

    let file_name = Path::new(template)
        .file_name()
        .ok_or_else(|| CommandError::TemplateNotFound(template.to_string()))?;

    for entry in WalkDir::new(templates_root)
        .into_iter()
        .filter_map(|entry| entry.ok())
    {
        if entry.file_type().is_file() && entry.file_name() == file_name {
            return Ok(entry.path().to_path_buf());
        }
    }

    Err(CommandError::TemplateNotFound(template.to_string()))
}

fn resolve_destination(dst: &str, payload: &Map<String, Value>) -> Result<PathBuf> {
    let mut result = String::new();
    let mut last_index = 0;

    for captures in PLACEHOLDER_RE.captures_iter(dst) {
        let m = captures.get(0).unwrap();
        result.push_str(&dst[last_index..m.start()]);
        let key = captures.get(1).unwrap().as_str();
        let replacement = lookup_placeholder(payload, key).ok_or_else(|| {
            CommandError::Other(format!(
                "missing placeholder value '{}' for destination {}",
                key, dst
            ))
        })?;
        result.push_str(&replacement);
        last_index = m.end();
    }

    result.push_str(&dst[last_index..]);
    Ok(PathBuf::from(result))
}

fn lookup_placeholder(payload: &Map<String, Value>, key: &str) -> Option<String> {
    if let Some(Value::String(value)) = payload.get(key) {
        return Some(value.clone());
    }

    if key.len() > 1 {
        let mut chars = key.chars();
        let first = chars.next()?.to_ascii_lowercase();
        let mut alt = String::from(first);
        alt.extend(chars);
        if let Some(Value::String(value)) = payload.get(&alt) {
            return Some(value.clone());
        }
    }

    None
}

fn normalize_line_endings(input: &str) -> String {
    input.replace("\r\n", "\n")
}

fn to_lower_camel(input: &str) -> String {
    if input.is_empty() {
        return String::new();
    }

    let mut chars = input.chars();
    let first = chars.next().unwrap().to_ascii_lowercase();
    let mut result = String::new();
    result.push(first);
    result.extend(chars);
    result
}

fn execute_plan(changes: Vec<FileChange>, dry_run: bool, summary: &mut ApplySummary) -> Result<()> {
    for change in changes {
        let FileChange {
            path,
            content,
            kind,
            note,
        } = change;

        match kind {
            FileChangeKind::Create => {
                if dry_run {
                    summary.created.push(path.clone());
                } else {
                    if let Some(parent) = path.parent() {
                        fs::create_dir_all(parent)?;
                    }
                    fs::write(&path, content.as_bytes())?;
                    summary.created.push(path.clone());
                }
            }
            FileChangeKind::Update => {
                if dry_run {
                    summary.updated.push(path.clone());
                } else {
                    if let Some(parent) = path.parent() {
                        fs::create_dir_all(parent)?;
                    }
                    fs::write(&path, content.as_bytes())?;
                    summary.updated.push(path.clone());
                }
            }
        }

        if let Some(note) = note {
            summary.notes.push(format!("{} ({})", path.display(), note));
        }
    }

    Ok(())
}

fn build_solution_stub(solution_name: &str) -> String {
    format!(
        "Microsoft Visual Studio Solution File, Format Version 12.00
# NetToolsKit CLI generated - TODO: register projects for {solution_name}
Global
    GlobalSection(SolutionConfigurationPlatforms) = preSolution
        Debug|Any CPU = Debug|Any CPU
        Release|Any CPU = Release|Any CPU
    EndGlobalSection
EndGlobal
"
    )
}

fn build_project_stub(name: &str, target_framework: &str, author: &str) -> String {
    format!(
        "<Project Sdk=\"Microsoft.NET.Sdk\">
  <PropertyGroup>
    <Authors>{author}</Authors>
    <IsPackable>false</IsPackable>
    <TargetFramework>{target_framework}</TargetFramework>
  </PropertyGroup>
  <!-- TODO: add references and package dependencies for {name} -->
</Project>
"
    )
}

fn build_project_payload(manifest: &ManifestDocument, project: &ManifestProject) -> Value {
    let mut payload = Map::new();
    payload.insert(
        "author".to_string(),
        Value::String(
            manifest
                .meta
                .author
                .clone()
                .unwrap_or_else(|| manifest.meta.name.clone()),
        ),
    );
    payload.insert(
        "targetFramework".to_string(),
        Value::String(manifest.conventions.target_framework.clone()),
    );
    payload.insert(
        "projectName".to_string(),
        Value::String(manifest.meta.name.clone()),
    );
    payload.insert(
        "projectFullName".to_string(),
        Value::String(project.name.clone()),
    );
    Value::Object(payload)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::{Path, PathBuf};
    use tempfile::TempDir;

    fn manifest_path() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("data")
            .join("ntk-manifest-domain.yml")
    }

    #[test]
    fn dry_run_artifact_manifest_generates_plan() -> Result<()> {
        let temp_dir = TempDir::new().expect("temp dir");
        let manifest = manifest_path();

        let summary = apply_manifest_for_tests(manifest.as_path(), temp_dir.path(), true)?;

        let expected_solution = temp_dir.path().join("samples/src/Rent.Service.sln");
        let expected_project = temp_dir
            .path()
            .join("samples/src/Rent.Service.Domain/Rent.Service.Domain.csproj");
        let expected_entity = temp_dir
            .path()
            .join("samples/src/Rent.Service.Domain/Entities/Rental.cs");

        assert!(
            summary
                .created
                .iter()
                .any(|path| path == &expected_solution),
            "expected solution path {} in created set",
            expected_solution.display()
        );
        assert!(
            summary.created.iter().any(|path| path == &expected_project),
            "expected project path {} in created set",
            expected_project.display()
        );
        assert!(
            summary.created.iter().any(|path| path == &expected_entity),
            "expected entity path {} in created set",
            expected_entity.display()
        );
        assert!(
            !expected_solution.exists(),
            "solution file should not exist in dry run"
        );

        Ok(())
    }

    #[test]
    fn apply_artifact_manifest_writes_files() -> Result<()> {
        let temp_dir = TempDir::new().expect("temp dir");
        let manifest = manifest_path();

        let summary = apply_manifest_for_tests(manifest.as_path(), temp_dir.path(), false)?;

        let solution = temp_dir.path().join("samples/src/Rent.Service.sln");
        let csproj = temp_dir
            .path()
            .join("samples/src/Rent.Service.Domain/Rent.Service.Domain.csproj");
        let entity = temp_dir
            .path()
            .join("samples/src/Rent.Service.Domain/Entities/Rental.cs");

        assert!(
            solution.exists(),
            "expected solution file {} to exist",
            solution.display()
        );
        assert!(
            csproj.exists(),
            "expected csproj file {} to exist",
            csproj.display()
        );
        assert!(
            entity.exists(),
            "expected entity file {} to exist",
            entity.display()
        );

        let entity_content = fs::read_to_string(&entity)?;
        assert!(
            entity_content.contains("public record Rental"),
            "expected generated entity to contain record declaration"
        );

        let csproj_content = fs::read_to_string(&csproj)?;
        assert!(
            csproj_content.contains("<TargetFramework>net9.0</TargetFramework>"),
            "domain csproj should target net9.0"
        );

        assert!(
            summary.updated.is_empty(),
            "unexpected updated entries in summary"
        );

        Ok(())
    }

    #[test]
    fn acceptance_manifest_generates_cross_layer_assets() -> Result<()> {
        let temp_dir = TempDir::new().expect("temp dir");

        let manifest = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("crates dir")
            .parent()
            .expect("cli root")
            .join(".docs")
            .join("ntk-manifest-acceptance.yml");

        let summary = apply_manifest_for_tests(manifest.as_path(), temp_dir.path(), false)?;
        let target_samples = temp_dir.path().join("samples").join("src");

        let domain_vo = target_samples.join("Rent.Service.Domain/ValueObjects/Money.cs");
        let domain_enum = target_samples.join("Rent.Service.Domain/Enums/RentalStatus.cs");
        let domain_entity = target_samples.join("Rent.Service.Domain/Entities/Rental.cs");
        let repository_interface =
            target_samples.join("Rent.Service.Application/Repositories/IRentalRepository.cs");

        assert!(domain_vo.exists(), "domain value object should exist");
        assert!(domain_enum.exists(), "domain enum should exist");
        assert!(domain_entity.exists(), "domain entity should exist");
        assert!(
            repository_interface.exists(),
            "application repository interface should exist"
        );

        assert!(
            summary
                .created
                .iter()
                .any(|path| path.ends_with("Rent.Service.Domain/Enums/RentalStatus.cs")),
            "summary should report created enum"
        );

        Ok(())
    }

    #[test]
    #[ignore]
    fn acceptance_manifest_exports_to_target() -> Result<()> {
        let target_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("target")
            .join("acceptance-domain");

        if target_dir.exists() {
            fs::remove_dir_all(&target_dir)?;
        }

        fs::create_dir_all(&target_dir)?;

        let manifest = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("crates dir")
            .parent()
            .expect("cli root")
            .join(".docs")
            .join("ntk-manifest-acceptance.yml");

        let summary = apply_manifest_for_tests(manifest.as_path(), &target_dir, false)?;

        let output_entity = target_dir.join("samples/src/Rent.Service.Domain/Entities/Rental.cs");
        let output_enum = target_dir.join("samples/src/Rent.Service.Domain/Enums/RentalStatus.cs");
        let output_repository = target_dir
            .join("samples/src/Rent.Service.Application/Repositories/IRentalRepository.cs");
        assert!(
            output_entity.exists(),
            "expected generated entity at {}",
            output_entity.display()
        );
        assert!(
            output_enum.exists(),
            "expected generated enum at {}",
            output_enum.display()
        );
        assert!(
            output_repository.exists(),
            "expected repository interface at {}",
            output_repository.display()
        );

        println!(
            "Acceptance export completed to {}. Created {} files.",
            target_dir.display(),
            summary.created.len()
        );
        Ok(())
    }
}
