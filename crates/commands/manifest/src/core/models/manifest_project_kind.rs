///! Project kind/type enumeration

use serde::Deserialize;

/// Project kind/type
#[derive(Debug, Deserialize, Clone, Copy, Default)]
#[serde(rename_all = "lowercase")]
pub enum ManifestProjectKind {
    Domain,
    Application,
    Infrastructure,
    Api,
    Worker,
    #[default]
    Unknown,
}

impl ManifestProjectKind {
    pub fn template_path(&self) -> Option<&'static str> {
        match self {
            Self::Domain => Some("dotnet/src/domain/domain.csproj.hbs"),
            _ => None,
        }
    }
}
