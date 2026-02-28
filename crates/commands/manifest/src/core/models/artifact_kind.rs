//! Artifact kinds for code generation

/// Artifact kinds for code generation
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ArtifactKind {
    /// DDD value object.
    ValueObject,
    /// DDD entity.
    Entity,
    /// DDD domain event.
    DomainEvent,
    /// Repository interface.
    RepositoryInterface,
    /// Enumeration type.
    EnumType,
    /// Use-case command.
    UseCaseCommand,
    /// API endpoint.
    Endpoint,
    /// Unrecognised artifact kind.
    Unknown(String),
}

impl ArtifactKind {
    /// Parse a string into an [`ArtifactKind`].
    pub fn parse_kind(value: &str) -> Self {
        match value {
            "value-object" => Self::ValueObject,
            "entity" => Self::Entity,
            "domain-event" => Self::DomainEvent,
            "repository-interface" => Self::RepositoryInterface,
            "enum" => Self::EnumType,
            "usecase-command" => Self::UseCaseCommand,
            "endpoint" => Self::Endpoint,
            other => Self::Unknown(other.to_string()),
        }
    }

    /// Return the canonical string label.
    pub fn label(&self) -> &str {
        match self {
            Self::ValueObject => "value-object",
            Self::Entity => "entity",
            Self::DomainEvent => "domain-event",
            Self::RepositoryInterface => "repository-interface",
            Self::EnumType => "enum",
            Self::UseCaseCommand => "usecase-command",
            Self::Endpoint => "endpoint",
            Self::Unknown(value) => value.as_str(),
        }
    }
}
