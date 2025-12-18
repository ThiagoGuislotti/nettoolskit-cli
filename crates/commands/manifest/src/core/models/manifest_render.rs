//! Render rules configuration

use serde::Deserialize;
use super::render_rule::RenderRule;

/// Render rules configuration
#[derive(Debug, Deserialize, Clone, Default)]
pub struct ManifestRender {
    #[serde(default)]
    pub rules: Vec<RenderRule>,
}
