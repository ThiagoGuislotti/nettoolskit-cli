//! Project definition

use serde::Deserialize;
use std::path::PathBuf;
use super::manifest_project_kind::ManifestProjectKind;

/// Project definition
#[derive(Debug, Deserialize, Clone)]
pub struct ManifestProject {
    #[serde(rename = "type")]
    #[serde(default)]
    pub kind: ManifestProjectKind,
    pub name: String,
    pub path: PathBuf,
}
