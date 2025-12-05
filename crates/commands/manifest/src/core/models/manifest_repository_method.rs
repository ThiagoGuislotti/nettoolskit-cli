///! Repository method

use serde::Deserialize;
use super::manifest_method_argument::ManifestMethodArgument;

/// Repository method
#[derive(Debug, Deserialize, Clone, Default)]
pub struct ManifestRepositoryMethod {
    pub name: String,
    #[serde(default)]
    pub args: Vec<ManifestMethodArgument>,
    #[serde(default)]
    pub returns: Option<String>,
}
