//! DDD entity

use super::manifest_field::ManifestField;
use serde::Deserialize;

/// DDD entity
#[derive(Debug, Deserialize, Clone, Default)]
pub struct ManifestEntity {
    /// Entity name.
    pub name: String,
    /// Entity fields.
    #[serde(default)]
    pub fields: Vec<ManifestField>,
}
