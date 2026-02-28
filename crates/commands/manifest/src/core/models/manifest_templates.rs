//! Template mappings configuration

use super::artifact_kind::ArtifactKind;
use super::template_mapping::TemplateMapping;
use serde::Deserialize;
use std::collections::BTreeMap;

/// Template mappings configuration
#[derive(Debug, Deserialize, Clone, Default)]
pub struct ManifestTemplates {
    /// Template-to-artifact mappings.
    #[serde(default)]
    pub mapping: Vec<TemplateMapping>,
}

impl ManifestTemplates {
    /// Index templates by artifact kind
    pub fn index_by_artifact(&self) -> BTreeMap<ArtifactKind, Vec<&TemplateMapping>> {
        let mut map: BTreeMap<ArtifactKind, Vec<&TemplateMapping>> = BTreeMap::new();
        for mapping in &self.mapping {
            let kind = ArtifactKind::parse_kind(&mapping.artifact);
            map.entry(kind).or_default().push(mapping);
        }
        map
    }
}
