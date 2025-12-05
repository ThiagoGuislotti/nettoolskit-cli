///! Apply mode enumeration

use serde::Deserialize;

/// Apply mode
#[derive(Debug, Deserialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum ApplyModeKind {
    Artifact,
    Feature,
    Layer,
}
