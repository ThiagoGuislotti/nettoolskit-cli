//! Project kind/type enumeration

use serde::Deserialize;

/// Project kind/type
#[derive(Debug, Deserialize, Clone, Copy, Default)]
#[serde(rename_all = "lowercase")]
pub enum ManifestProjectKind {
    /// Domain layer project.
    Domain,
    /// Application layer project.
    Application,
    /// Infrastructure layer project.
    Infrastructure,
    /// API layer project.
    Api,
    /// Worker/background service project.
    Worker,
    /// Unknown or unrecognized project type.
    #[default]
    Unknown,
}

impl ManifestProjectKind {
    /// Returns the Handlebars template path for this project kind, if available.
    pub fn template_path(&self) -> Option<&'static str> {
        match self {
            Self::Domain => Some("dotnet/src/domain/domain.csproj.hbs"),
            _ => None,
        }
    }
}
