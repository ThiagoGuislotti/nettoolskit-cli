///! Manifest metadata

use serde::Deserialize;

/// Manifest metadata
#[derive(Debug, Deserialize, Clone)]
pub struct ManifestMeta {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub author: Option<String>,
}
