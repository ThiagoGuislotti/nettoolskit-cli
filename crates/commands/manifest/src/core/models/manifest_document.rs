//! Manifest document root

use super::manifest_apply::ManifestApply;
use super::manifest_context::ManifestContext;
use super::manifest_conventions::ManifestConventions;
use super::manifest_guards::ManifestGuards;
use super::manifest_kind::ManifestKind;
use super::manifest_meta::ManifestMeta;
use super::manifest_project::ManifestProject;
use super::manifest_render::ManifestRender;
use super::manifest_solution::ManifestSolution;
use super::manifest_templates::ManifestTemplates;
use serde::Deserialize;
use std::collections::BTreeMap;

/// Manifest document root
#[derive(Debug, Deserialize)]
pub struct ManifestDocument {
    /// Manifest schema version.
    #[serde(rename = "apiVersion")]
    pub api_version: String,
    /// Document kind.
    pub kind: ManifestKind,
    /// Manifest metadata.
    pub meta: ManifestMeta,
    /// Naming and framework conventions.
    pub conventions: ManifestConventions,
    /// Solution structure.
    pub solution: ManifestSolution,
    /// Validation guards.
    #[serde(default)]
    pub guards: ManifestGuards,
    /// Project definitions keyed by identifier.
    #[serde(default)]
    pub projects: BTreeMap<String, ManifestProject>,
    /// DDD bounded contexts.
    #[serde(default)]
    pub contexts: Vec<ManifestContext>,
    /// Template mapping configuration.
    #[serde(default)]
    pub templates: ManifestTemplates,
    /// Render rules.
    #[serde(default)]
    pub render: ManifestRender,
    /// Apply configuration.
    pub apply: ManifestApply,
}
