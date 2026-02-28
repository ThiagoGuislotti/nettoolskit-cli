//! DDD value object

use super::manifest_field::ManifestField;
use serde::Deserialize;

/// DDD value object
#[derive(Debug, Deserialize, Clone, Default)]
pub struct ManifestValueObject {
    /// Value object name.
    pub name: String,
    /// Value object fields.
    #[serde(default)]
    pub fields: Vec<ManifestField>,
}
