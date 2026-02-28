//! Use case definition

use super::manifest_field::ManifestField;
use serde::Deserialize;

/// Use case definition
#[derive(Debug, Deserialize, Clone, Default)]
pub struct ManifestUseCase {
    /// Use case name.
    pub name: String,
    /// Use case type (e.g. command, query).
    #[serde(rename = "type")]
    pub use_case_type: String,
    /// Input fields.
    #[serde(default)]
    pub input: Vec<ManifestField>,
    /// Output fields.
    #[serde(default)]
    pub output: Vec<ManifestField>,
}
