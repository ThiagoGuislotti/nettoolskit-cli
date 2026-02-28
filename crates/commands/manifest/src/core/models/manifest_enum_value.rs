//! Enum value

use serde::Deserialize;

/// Enum value
#[derive(Debug, Deserialize, Clone, Default)]
pub struct ManifestEnumValue {
    /// Member name.
    pub name: String,
    /// Numeric value.
    pub value: i32,
}
