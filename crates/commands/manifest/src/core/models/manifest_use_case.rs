///! Use case definition

use serde::Deserialize;
use super::manifest_field::ManifestField;

/// Use case definition
#[derive(Debug, Deserialize, Clone, Default)]
pub struct ManifestUseCase {
    pub name: String,
    #[serde(rename = "type")]
    pub use_case_type: String,
    #[serde(default)]
    pub input: Vec<ManifestField>,
    #[serde(default)]
    pub output: Vec<ManifestField>,
}
