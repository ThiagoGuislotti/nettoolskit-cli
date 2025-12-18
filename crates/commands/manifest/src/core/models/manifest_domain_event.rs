//! DDD domain event

use serde::Deserialize;

/// DDD domain event
#[derive(Debug, Deserialize, Clone, Default)]
pub struct ManifestDomainEvent {
    pub name: String,
}
