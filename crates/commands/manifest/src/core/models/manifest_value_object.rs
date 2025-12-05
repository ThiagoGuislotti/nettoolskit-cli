///! DDD value object

use serde::Deserialize;
use super::manifest_field::ManifestField;

/// DDD value object
#[derive(Debug, Deserialize, Clone, Default)]
pub struct ManifestValueObject {
    pub name: String,
    #[serde(default)]
    pub fields: Vec<ManifestField>,
}
