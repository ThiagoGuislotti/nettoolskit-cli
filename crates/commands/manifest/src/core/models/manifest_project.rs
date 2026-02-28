//! Project definition

use super::manifest_project_kind::ManifestProjectKind;
use serde::Deserialize;
use std::path::PathBuf;

/// Project definition
#[derive(Debug, Deserialize, Clone)]
pub struct ManifestProject {
    /// Project type/kind.
    #[serde(rename = "type")]
    #[serde(default)]
    pub kind: ManifestProjectKind,
    /// Project name.
    pub name: String,
    /// Filesystem path to the project.
    pub path: PathBuf,
}
