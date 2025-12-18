//! Code generation policies

use serde::Deserialize;
use super::manifest_collision_policy::ManifestCollisionPolicy;

/// Code generation policies
#[derive(Debug, Deserialize, Clone, Default)]
pub struct ManifestPolicy {
    #[serde(default)]
    pub collision: Option<ManifestCollisionPolicy>,
    #[serde(default, rename = "insertTodoWhenMissing")]
    pub insert_todo_when_missing: bool,
    #[serde(default)]
    pub strict: bool,
}
