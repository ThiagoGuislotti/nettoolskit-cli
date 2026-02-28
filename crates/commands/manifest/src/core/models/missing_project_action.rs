//! Action to take when project is missing

use serde::Deserialize;

/// Action to take when project is missing
#[derive(Debug, Deserialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum MissingProjectAction {
    /// Abort execution with an error.
    Fail,
    /// Skip the missing project and continue.
    Skip,
}
