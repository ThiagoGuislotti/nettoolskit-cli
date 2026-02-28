//! Render rules configuration

use super::render_rule::RenderRule;
use serde::Deserialize;

/// Render rules configuration
#[derive(Debug, Deserialize, Clone, Default)]
pub struct ManifestRender {
    /// Collection of render rules.
    #[serde(default)]
    pub rules: Vec<RenderRule>,
}
