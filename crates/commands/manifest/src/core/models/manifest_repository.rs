///! Repository definition

use serde::Deserialize;
use super::manifest_repository_method::ManifestRepositoryMethod;

/// Repository definition
#[derive(Debug, Deserialize, Clone, Default)]
pub struct ManifestRepository {
    pub name: String,
    #[serde(default)]
    pub methods: Vec<ManifestRepositoryMethod>,
}
