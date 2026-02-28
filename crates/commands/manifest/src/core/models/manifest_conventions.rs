//! Naming and code generation conventions

use super::manifest_policy::ManifestPolicy;
use serde::Deserialize;

/// Naming and code generation conventions
#[derive(Debug, Deserialize, Clone)]
pub struct ManifestConventions {
    /// Root namespace for generated code.
    #[serde(rename = "namespaceRoot")]
    pub namespace_root: String,
    /// Target framework moniker (e.g. `net8.0`).
    #[serde(rename = "targetFramework")]
    pub target_framework: String,
    /// Code-generation policies.
    #[serde(default)]
    pub policy: ManifestPolicy,
}
