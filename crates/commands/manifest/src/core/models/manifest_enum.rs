///! Enum definition

use serde::Deserialize;
use super::manifest_enum_value::ManifestEnumValue;

/// Enum definition
#[derive(Debug, Deserialize, Clone, Default)]
pub struct ManifestEnum {
    pub name: String,
    #[serde(default)]
    pub values: Vec<ManifestEnumValue>,
}
