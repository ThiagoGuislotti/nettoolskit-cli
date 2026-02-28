//! Apply artifact configuration

use serde::Deserialize;

/// Apply artifact configuration
#[derive(Debug, Deserialize, Clone, Default)]
pub struct ApplyArtifact {
    /// Artifact kind identifier.
    pub kind: String,
    /// Optional bounded-context scope.
    #[serde(default)]
    pub context: Option<String>,
    /// Optional artifact name override.
    #[serde(default)]
    pub name: Option<String>,
}
