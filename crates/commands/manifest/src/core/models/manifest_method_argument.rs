//! Method argument

use serde::Deserialize;

/// Method argument
#[derive(Debug, Deserialize, Clone, Default)]
pub struct ManifestMethodArgument {
    /// Argument name.
    pub name: String,
    /// Argument data type.
    #[serde(rename = "type")]
    pub r#type: String,
}
