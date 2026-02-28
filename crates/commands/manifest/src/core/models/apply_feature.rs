//! Apply feature configuration

use serde::Deserialize;

/// Apply feature configuration
#[derive(Debug, Deserialize, Clone, Default)]
pub struct ApplyFeature {
    /// Optional bounded-context scope.
    #[serde(default)]
    pub context: Option<String>,
    /// Feature names to include.
    #[serde(default)]
    pub include: Vec<String>,
}
