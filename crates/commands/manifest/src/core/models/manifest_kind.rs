///! Manifest document kind enumeration

use serde::Deserialize;

/// Manifest kind (currently only Solution supported)
#[derive(Debug, Deserialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum ManifestKind {
    Solution,
}
