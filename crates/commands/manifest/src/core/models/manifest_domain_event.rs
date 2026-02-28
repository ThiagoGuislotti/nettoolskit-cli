//! DDD domain event

use serde::Deserialize;

/// DDD domain event
#[derive(Debug, Deserialize, Clone, Default)]
pub struct ManifestDomainEvent {
    /// Event name.
    pub name: String,
}
