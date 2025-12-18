//! Enum value

use serde::Deserialize;

/// Enum value
#[derive(Debug, Deserialize, Clone, Default)]
pub struct ManifestEnumValue {
    pub name: String,
    pub value: i32,
}
