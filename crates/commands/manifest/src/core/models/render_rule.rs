///! Render rule definition

use serde::Deserialize;

/// Render rule definition
#[derive(Debug, Deserialize, Clone)]
pub struct RenderRule {
    pub expand: String,
    #[serde(rename = "as")]
    pub alias: String,
}
