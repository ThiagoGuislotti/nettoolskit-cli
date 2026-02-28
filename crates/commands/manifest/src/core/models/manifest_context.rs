//! DDD bounded context

use super::manifest_aggregate::ManifestAggregate;
use super::manifest_use_case::ManifestUseCase;
use serde::Deserialize;

/// DDD bounded context
#[derive(Debug, Deserialize, Clone, Default)]
pub struct ManifestContext {
    /// Context name.
    pub name: String,
    /// Aggregates within this context.
    #[serde(default)]
    pub aggregates: Vec<ManifestAggregate>,
    /// Use cases within this context.
    #[serde(default, rename = "useCases")]
    pub use_cases: Vec<ManifestUseCase>,
}
