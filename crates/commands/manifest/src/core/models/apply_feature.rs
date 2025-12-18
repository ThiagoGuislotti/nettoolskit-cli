//! Apply feature configuration

use serde::Deserialize;

/// Apply feature configuration
#[derive(Debug, Deserialize, Clone, Default)]
pub struct ApplyFeature {
    #[serde(default)]
    pub context: Option<String>,
    #[serde(default)]
    pub include: Vec<String>,
}
