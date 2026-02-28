//! Repository method

use super::manifest_method_argument::ManifestMethodArgument;
use serde::Deserialize;

/// Repository method
#[derive(Debug, Deserialize, Clone, Default)]
pub struct ManifestRepositoryMethod {
    /// Method name.
    pub name: String,
    /// Method arguments.
    #[serde(default)]
    pub args: Vec<ManifestMethodArgument>,
    /// Optional return type.
    #[serde(default)]
    pub returns: Option<String>,
}
