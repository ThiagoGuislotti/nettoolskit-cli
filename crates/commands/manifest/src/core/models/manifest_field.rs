//! Field definition

use serde::Deserialize;

/// Field definition
#[derive(Debug, Deserialize, Clone, Default)]
pub struct ManifestField {
    /// Field name.
    pub name: String,
    /// Field data type.
    #[serde(rename = "type")]
    pub r#type: String,
    /// Whether this field is a primary key.
    #[serde(default)]
    pub key: bool,
    /// Whether this field allows null values.
    #[serde(default)]
    pub nullable: bool,
    /// Optional database column name override.
    #[serde(default, rename = "columnName")]
    pub column_name: Option<String>,
}
