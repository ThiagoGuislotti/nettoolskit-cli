///! DDD aggregate root

use serde::Deserialize;
use super::manifest_domain_event::ManifestDomainEvent;
use super::manifest_entity::ManifestEntity;
use super::manifest_enum::ManifestEnum;
use super::manifest_repository::ManifestRepository;
use super::manifest_value_object::ManifestValueObject;

/// DDD aggregate root
#[derive(Debug, Deserialize, Clone, Default)]
pub struct ManifestAggregate {
    pub name: String,
    #[serde(default, rename = "valueObjects")]
    pub value_objects: Vec<ManifestValueObject>,
    #[serde(default)]
    pub entities: Vec<ManifestEntity>,
    #[serde(default, rename = "domainEvents")]
    pub domain_events: Vec<ManifestDomainEvent>,
    #[serde(default)]
    pub repository: Option<ManifestRepository>,
    #[serde(default)]
    pub enums: Vec<ManifestEnum>,
}
