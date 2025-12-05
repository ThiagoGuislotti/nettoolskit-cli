///! Apply artifact configuration

use serde::Deserialize;

/// Apply artifact configuration
#[derive(Debug, Deserialize, Clone, Default)]
pub struct ApplyArtifact {
    pub kind: String,
    #[serde(default)]
    pub context: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
}
