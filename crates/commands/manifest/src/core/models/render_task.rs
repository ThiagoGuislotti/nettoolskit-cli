///! Template rendering task

use serde_json::Value;
use std::path::PathBuf;
use super::artifact_kind::ArtifactKind;

/// Template rendering task
#[derive(Debug)]
pub struct RenderTask {
    pub kind: ArtifactKind,
    pub template: String,
    pub destination: PathBuf,
    pub data: Value,
    pub note: Option<String>,
}
