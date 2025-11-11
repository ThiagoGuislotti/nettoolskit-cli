/// Domain models for manifest documents
///
/// These types represent the structure of NetToolsKit manifest files (YAML).
use owo_colors::OwoColorize;
use regex::Regex;
use once_cell::sync::Lazy;
use serde::Deserialize;
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::PathBuf;

/// Regex for placeholder detection in templates
#[allow(dead_code)]
pub static PLACEHOLDER_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\{([A-Za-z0-9_]+)\}").expect("invalid placeholder regex"));

/// Manifest document root
#[derive(Debug, Deserialize)]
pub struct ManifestDocument {
    #[serde(rename = "apiVersion")]
    pub api_version: String,
    pub kind: ManifestKind,
    pub meta: ManifestMeta,
    pub conventions: ManifestConventions,
    pub solution: ManifestSolution,
    #[serde(default)]
    pub guards: ManifestGuards,
    #[serde(default)]
    pub projects: BTreeMap<String, ManifestProject>,
    #[serde(default)]
    pub contexts: Vec<ManifestContext>,
    #[serde(default)]
    pub templates: ManifestTemplates,
    #[serde(default)]
    pub render: ManifestRender,
    pub apply: ManifestApply,
}

/// Manifest kind (currently only Solution supported)
#[derive(Debug, Deserialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum ManifestKind {
    Solution,
}

/// Manifest metadata
#[derive(Debug, Deserialize, Clone)]
pub struct ManifestMeta {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub author: Option<String>,
}

/// Naming and code generation conventions
#[derive(Debug, Deserialize, Clone)]
pub struct ManifestConventions {
    #[serde(rename = "namespaceRoot")]
    pub namespace_root: String,
    #[serde(rename = "targetFramework")]
    pub target_framework: String,
    #[serde(default)]
    pub policy: ManifestPolicy,
}

/// Solution configuration
#[derive(Debug, Deserialize, Clone)]
pub struct ManifestSolution {
    pub root: PathBuf,
    #[serde(rename = "slnFile")]
    pub sln_file: PathBuf,
}

/// Guards for validation and safety checks
#[derive(Debug, Deserialize, Clone, Default)]
pub struct ManifestGuards {
    #[serde(default, rename = "requireExistingProjects")]
    pub require_existing_projects: bool,
    #[serde(default, rename = "onMissingProject")]
    pub on_missing_project: Option<MissingProjectAction>,
}

/// Action to take when project is missing
#[derive(Debug, Deserialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum MissingProjectAction {
    Fail,
    Skip,
}

/// Project definition
#[derive(Debug, Deserialize, Clone)]
pub struct ManifestProject {
    #[serde(rename = "type")]
    #[serde(default)]
    pub kind: ManifestProjectKind,
    pub name: String,
    pub path: PathBuf,
}

/// Project kind/type
#[derive(Debug, Deserialize, Clone, Copy, Default)]
#[serde(rename_all = "lowercase")]
pub enum ManifestProjectKind {
    Domain,
    Application,
    Infrastructure,
    Api,
    Worker,
    #[default]
    Unknown,
}

impl ManifestProjectKind {
    pub fn template_path(&self) -> Option<&'static str> {
        match self {
            Self::Domain => Some("dotnet/src/domain/domain.csproj.hbs"),
            _ => None,
        }
    }
}

/// Code generation policies
#[derive(Debug, Deserialize, Clone, Default)]
pub struct ManifestPolicy {
    #[serde(default)]
    pub collision: Option<ManifestCollisionPolicy>,
    #[serde(default, rename = "insertTodoWhenMissing")]
    pub insert_todo_when_missing: bool,
    #[serde(default)]
    pub strict: bool,
}

/// File collision handling policy
#[derive(Debug, Deserialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum ManifestCollisionPolicy {
    Fail,
    Overwrite,
}

/// DDD bounded context
#[derive(Debug, Deserialize, Clone, Default)]
pub struct ManifestContext {
    pub name: String,
    #[serde(default)]
    pub aggregates: Vec<ManifestAggregate>,
    #[serde(default, rename = "useCases")]
    pub use_cases: Vec<ManifestUseCase>,
}

/// DDD aggregate root
#[derive(Debug, Deserialize, Clone, Default)]
pub struct ManifestAggregate {
    pub name: String,
    #[serde(default, rename = "valueObjects")]
    pub value_objects: Vec<ManifestValueObject>,
    #[serde(default)]
    pub entities: Vec<ManifestEntity>,
    #[serde(default, rename = "domainEvents")]
    pub domain_events: Vec<ManifestDomainEvent>,
    #[serde(default)]
    pub repository: Option<ManifestRepository>,
    #[serde(default)]
    pub enums: Vec<ManifestEnum>,
}

/// DDD value object
#[derive(Debug, Deserialize, Clone, Default)]
pub struct ManifestValueObject {
    pub name: String,
    #[serde(default)]
    pub fields: Vec<ManifestField>,
}

/// DDD entity
#[derive(Debug, Deserialize, Clone, Default)]
pub struct ManifestEntity {
    pub name: String,
    #[serde(default)]
    pub fields: Vec<ManifestField>,
}

/// DDD domain event
#[derive(Debug, Deserialize, Clone, Default)]
pub struct ManifestDomainEvent {
    pub name: String,
}

/// Repository definition
#[derive(Debug, Deserialize, Clone, Default)]
pub struct ManifestRepository {
    pub name: String,
    #[serde(default)]
    pub methods: Vec<ManifestRepositoryMethod>,
}

/// Repository method
#[derive(Debug, Deserialize, Clone, Default)]
pub struct ManifestRepositoryMethod {
    pub name: String,
    #[serde(default)]
    pub args: Vec<ManifestMethodArgument>,
    #[serde(default)]
    pub returns: Option<String>,
}

/// Method argument
#[derive(Debug, Deserialize, Clone, Default)]
pub struct ManifestMethodArgument {
    pub name: String,
    #[serde(rename = "type")]
    pub r#type: String,
}

/// Field definition
#[derive(Debug, Deserialize, Clone, Default)]
pub struct ManifestField {
    pub name: String,
    #[serde(rename = "type")]
    pub r#type: String,
    #[serde(default)]
    pub key: bool,
    #[serde(default)]
    pub nullable: bool,
    #[serde(default, rename = "columnName")]
    pub column_name: Option<String>,
}

/// Enum definition
#[derive(Debug, Deserialize, Clone, Default)]
pub struct ManifestEnum {
    pub name: String,
    #[serde(default)]
    pub values: Vec<ManifestEnumValue>,
}

/// Enum value
#[derive(Debug, Deserialize, Clone, Default)]
pub struct ManifestEnumValue {
    pub name: String,
    pub value: i32,
}

/// Use case definition
#[derive(Debug, Deserialize, Clone, Default)]
pub struct ManifestUseCase {
    pub name: String,
    #[serde(rename = "type")]
    pub use_case_type: String,
    #[serde(default)]
    pub input: Vec<ManifestField>,
    #[serde(default)]
    pub output: Vec<ManifestField>,
}

/// Artifact kinds for code generation
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ArtifactKind {
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
    pub fn from_str(value: &str) -> Self {
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

    pub fn label(&self) -> &str {
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

/// Template rendering task
#[derive(Debug)]
pub struct RenderTask {
    pub kind: ArtifactKind,
    pub template: String,
    pub destination: PathBuf,
    pub data: Value,
    pub note: Option<String>,
}

/// Kind of file operation
#[derive(Debug, Clone, Copy)]
pub enum FileChangeKind {
    Create,
    Update,
}

/// File change operation
#[derive(Debug)]
pub struct FileChange {
    pub path: PathBuf,
    pub content: String,
    pub kind: FileChangeKind,
    pub note: Option<String>,
}

/// Summary of manifest execution
#[derive(Default, Debug)]
pub struct ExecutionSummary {
    pub created: Vec<PathBuf>,
    pub updated: Vec<PathBuf>,
    pub skipped: Vec<(PathBuf, String)>,
    pub notes: Vec<String>,
}

impl ExecutionSummary {
    /// Print summary to console
    pub fn print(&self, dry_run: bool) {
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

/// Template mappings configuration
#[derive(Debug, Deserialize, Clone, Default)]
pub struct ManifestTemplates {
    #[serde(default)]
    pub mapping: Vec<TemplateMapping>,
}

impl ManifestTemplates {
    /// Index templates by artifact kind
    pub fn index_by_artifact(&self) -> BTreeMap<ArtifactKind, Vec<&TemplateMapping>> {
        let mut map: BTreeMap<ArtifactKind, Vec<&TemplateMapping>> = BTreeMap::new();
        for mapping in &self.mapping {
            let kind = ArtifactKind::from_str(&mapping.artifact);
            map.entry(kind).or_default().push(mapping);
        }
        map
    }
}

/// Template mapping definition
#[derive(Debug, Deserialize, Clone)]
pub struct TemplateMapping {
    pub artifact: String,
    pub template: String,
    pub dst: String,
}

/// Render rules configuration
#[derive(Debug, Deserialize, Clone, Default)]
pub struct ManifestRender {
    #[serde(default)]
    pub rules: Vec<RenderRule>,
}

/// Render rule definition
#[derive(Debug, Deserialize, Clone)]
pub struct RenderRule {
    pub expand: String,
    #[serde(rename = "as")]
    pub alias: String,
}

/// Apply configuration
#[derive(Debug, Deserialize, Clone)]
pub struct ManifestApply {
    pub mode: ApplyModeKind,
    #[serde(default)]
    pub artifact: Option<ApplyArtifact>,
    #[serde(default)]
    pub feature: Option<ApplyFeature>,
    #[serde(default)]
    pub layer: Option<ApplyLayer>,
}

/// Apply mode
#[derive(Debug, Deserialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum ApplyModeKind {
    Artifact,
    Feature,
    Layer,
}

/// Apply artifact configuration
#[derive(Debug, Deserialize, Clone, Default)]
pub struct ApplyArtifact {
    pub kind: String,
    #[serde(default)]
    pub context: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
}

/// Apply feature configuration
#[derive(Debug, Deserialize, Clone, Default)]
pub struct ApplyFeature {
    #[serde(default)]
    pub context: Option<String>,
    #[serde(default)]
    pub include: Vec<String>,
}

/// Apply layer configuration
#[derive(Debug, Deserialize, Clone, Default)]
pub struct ApplyLayer {
    #[serde(default)]
    pub include: Vec<String>,
}