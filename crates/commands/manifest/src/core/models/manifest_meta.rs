//! Manifest metadata

use serde::Deserialize;

/// Manifest metadata
#[derive(Debug, Deserialize, Clone)]
pub struct ManifestMeta {
    /// Manifest name.
    pub name: String,
    /// Optional description.
    #[serde(default)]
    pub description: Option<String>,
    /// Optional author.
    #[serde(default)]
    pub author: Option<String>,
}
