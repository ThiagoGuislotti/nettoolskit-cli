//! Apply configuration

use serde::Deserialize;
use super::apply_artifact::ApplyArtifact;
use super::apply_feature::ApplyFeature;
use super::apply_layer::ApplyLayer;
use super::apply_mode_kind::ApplyModeKind;

/// Apply configuration
#[derive(Debug, Deserialize, Clone)]
pub struct ManifestApply {
    pub mode: ApplyModeKind,
    #[serde(default)]
    pub artifact: Option<ApplyArtifact>,
    #[serde(default)]
    pub feature: Option<ApplyFeature>,
    #[serde(default)]
    pub layer: Option<ApplyLayer>,
}
