//! Naming and code generation conventions

use serde::Deserialize;
use super::manifest_policy::ManifestPolicy;

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
