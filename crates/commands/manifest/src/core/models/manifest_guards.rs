//! Guards for validation and safety checks

use super::missing_project_action::MissingProjectAction;
use serde::Deserialize;

/// Guards for validation and safety checks
#[derive(Debug, Deserialize, Clone, Default)]
pub struct ManifestGuards {
    /// Whether all declared projects must exist on disk.
    #[serde(default, rename = "requireExistingProjects")]
    pub require_existing_projects: bool,
    /// Action to take when a declared project is missing.
    #[serde(default, rename = "onMissingProject")]
    pub on_missing_project: Option<MissingProjectAction>,
}
