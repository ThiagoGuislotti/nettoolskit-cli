//! Apply mode enumeration

use serde::Deserialize;

/// Apply mode
#[derive(Debug, Deserialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum ApplyModeKind {
    /// Apply a single artifact.
    Artifact,
    /// Apply a named feature set.
    Feature,
    /// Apply an architectural layer.
    Layer,
}
