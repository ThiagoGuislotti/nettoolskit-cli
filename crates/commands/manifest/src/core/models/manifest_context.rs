//! DDD bounded context

use serde::Deserialize;
use super::manifest_aggregate::ManifestAggregate;
use super::manifest_use_case::ManifestUseCase;

/// DDD bounded context
#[derive(Debug, Deserialize, Clone, Default)]
pub struct ManifestContext {
    pub name: String,
    #[serde(default)]
    pub aggregates: Vec<ManifestAggregate>,
    #[serde(default, rename = "useCases")]
    pub use_cases: Vec<ManifestUseCase>,
}
