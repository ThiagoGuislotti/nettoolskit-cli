//! Render rule definition

use serde::Deserialize;

/// Render rule definition
#[derive(Debug, Deserialize, Clone)]
pub struct RenderRule {
    /// Expression to expand during rendering.
    pub expand: String,
    /// Alias used in templates for the expanded value.
    #[serde(rename = "as")]
    pub alias: String,
}
