//! Enum definition

use super::manifest_enum_value::ManifestEnumValue;
use serde::Deserialize;

/// Enum definition
#[derive(Debug, Deserialize, Clone, Default)]
pub struct ManifestEnum {
    /// Enum name.
    pub name: String,
    /// Enum members.
    #[serde(default)]
    pub values: Vec<ManifestEnumValue>,
}
