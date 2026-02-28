//! Repository definition

use super::manifest_repository_method::ManifestRepositoryMethod;
use serde::Deserialize;

/// Repository definition
#[derive(Debug, Deserialize, Clone, Default)]
pub struct ManifestRepository {
    /// Repository name.
    pub name: String,
    /// Repository methods.
    #[serde(default)]
    pub methods: Vec<ManifestRepositoryMethod>,
}
