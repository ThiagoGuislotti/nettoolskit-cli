//! Guards for validation and safety checks

use serde::Deserialize;
use super::missing_project_action::MissingProjectAction;

/// Guards for validation and safety checks
#[derive(Debug, Deserialize, Clone, Default)]
pub struct ManifestGuards {
    #[serde(default, rename = "requireExistingProjects")]
    pub require_existing_projects: bool,
    #[serde(default, rename = "onMissingProject")]
    pub on_missing_project: Option<MissingProjectAction>,
}
