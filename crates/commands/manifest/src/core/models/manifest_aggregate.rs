//! DDD aggregate root

use super::manifest_domain_event::ManifestDomainEvent;
use super::manifest_entity::ManifestEntity;
use super::manifest_enum::ManifestEnum;
use super::manifest_repository::ManifestRepository;
use super::manifest_value_object::ManifestValueObject;
use serde::Deserialize;

/// DDD aggregate root
#[derive(Debug, Deserialize, Clone, Default)]
pub struct ManifestAggregate {
    /// Aggregate name.
    pub name: String,
    /// Value objects owned by this aggregate.
    #[serde(default, rename = "valueObjects")]
    pub value_objects: Vec<ManifestValueObject>,
    /// Child entities.
    #[serde(default)]
    pub entities: Vec<ManifestEntity>,
    /// Domain events raised by this aggregate.
    #[serde(default, rename = "domainEvents")]
    pub domain_events: Vec<ManifestDomainEvent>,
    /// Optional repository interface.
    #[serde(default)]
    pub repository: Option<ManifestRepository>,
    /// Enumerations scoped to this aggregate.
    #[serde(default)]
    pub enums: Vec<ManifestEnum>,
}
