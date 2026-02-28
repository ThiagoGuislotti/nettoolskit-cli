//! Template rendering task

use super::artifact_kind::ArtifactKind;
use serde_json::Value;
use std::path::PathBuf;

/// Template rendering task
#[derive(Debug)]
pub struct RenderTask {
    /// Artifact kind for this render task.
    pub kind: ArtifactKind,
    /// Handlebars template name.
    pub template: String,
    /// Output file destination path.
    pub destination: PathBuf,
    /// Template context data.
    pub data: Value,
    /// Optional note or comment for the generated file.
    pub note: Option<String>,
}
