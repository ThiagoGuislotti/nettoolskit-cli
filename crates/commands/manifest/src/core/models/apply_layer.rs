//! Apply layer configuration

use serde::Deserialize;

/// Apply layer configuration
#[derive(Debug, Deserialize, Clone, Default)]
pub struct ApplyLayer {
    /// Layer names to include.
    #[serde(default)]
    pub include: Vec<String>,
}
