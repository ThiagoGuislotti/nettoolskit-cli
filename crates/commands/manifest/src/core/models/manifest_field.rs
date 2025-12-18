//! Field definition

use serde::Deserialize;

/// Field definition
#[derive(Debug, Deserialize, Clone, Default)]
pub struct ManifestField {
    pub name: String,
    #[serde(rename = "type")]
    pub r#type: String,
    #[serde(default)]
    pub key: bool,
    #[serde(default)]
    pub nullable: bool,
    #[serde(default, rename = "columnName")]
    pub column_name: Option<String>,
}
