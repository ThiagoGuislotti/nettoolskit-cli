//! Code generation policies

use super::manifest_collision_policy::ManifestCollisionPolicy;
use serde::Deserialize;

/// Code generation policies
#[derive(Debug, Deserialize, Clone, Default)]
pub struct ManifestPolicy {
    /// File collision resolution policy.
    #[serde(default)]
    pub collision: Option<ManifestCollisionPolicy>,
    /// Insert TODO markers for missing code sections.
    #[serde(default, rename = "insertTodoWhenMissing")]
    pub insert_todo_when_missing: bool,
    /// Enable strict validation mode.
    #[serde(default)]
    pub strict: bool,
}
