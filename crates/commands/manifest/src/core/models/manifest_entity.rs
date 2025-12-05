///! DDD entity

use serde::Deserialize;
use super::manifest_field::ManifestField;

/// DDD entity
#[derive(Debug, Deserialize, Clone, Default)]
pub struct ManifestEntity {
    pub name: String,
    #[serde(default)]
    pub fields: Vec<ManifestField>,
}
