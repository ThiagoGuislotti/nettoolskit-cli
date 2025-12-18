//! Manifest document root

use serde::Deserialize;
use std::collections::BTreeMap;
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

/// Manifest document root
#[derive(Debug, Deserialize)]
pub struct ManifestDocument {
    #[serde(rename = "apiVersion")]
    pub api_version: String,
    pub kind: ManifestKind,
    pub meta: ManifestMeta,
    pub conventions: ManifestConventions,
    pub solution: ManifestSolution,
    #[serde(default)]
    pub guards: ManifestGuards,
    #[serde(default)]
    pub projects: BTreeMap<String, ManifestProject>,
    #[serde(default)]
    pub contexts: Vec<ManifestContext>,
    #[serde(default)]
    pub templates: ManifestTemplates,
    #[serde(default)]
    pub render: ManifestRender,
    pub apply: ManifestApply,
}
