//! Apply configuration

use super::apply_artifact::ApplyArtifact;
use super::apply_feature::ApplyFeature;
use super::apply_layer::ApplyLayer;
use super::apply_mode_kind::ApplyModeKind;
use serde::Deserialize;

/// Apply configuration
#[derive(Debug, Deserialize, Clone)]
pub struct ManifestApply {
    /// Apply mode (artifact, feature, or layer).
    pub mode: ApplyModeKind,
    /// Artifact-specific configuration.
    #[serde(default)]
    pub artifact: Option<ApplyArtifact>,
    /// Feature-specific configuration.
    #[serde(default)]
    pub feature: Option<ApplyFeature>,
    /// Layer-specific configuration.
    #[serde(default)]
    pub layer: Option<ApplyLayer>,
}
