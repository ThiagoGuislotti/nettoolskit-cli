//! Method argument

use serde::Deserialize;

/// Method argument
#[derive(Debug, Deserialize, Clone, Default)]
pub struct ManifestMethodArgument {
    pub name: String,
    #[serde(rename = "type")]
    pub r#type: String,
}
