/// Manifest YAML parsing
use crate::error::{ManifestError, ManifestResult};
use crate::models::ManifestDocument;
use std::path::Path;

/// Parser for manifest YAML files
pub struct ManifestParser;

impl ManifestParser {
    /// Parse manifest from file path
    pub fn from_file(path: &Path) -> ManifestResult<ManifestDocument> {
        let content = std::fs::read_to_string(path).map_err(|err| ManifestError::ReadError {
            path: path.display().to_string(),
            source: err,
        })?;

        let manifest: ManifestDocument = serde_yaml::from_str(&content)?;

        // Validate API version
        if manifest.api_version != "ntk/v1" {
            return Err(ManifestError::ValidationError(format!(
                "unsupported apiVersion: {}",
                manifest.api_version
            )));
        }

        Ok(manifest)
    }

    /// Validate manifest structure
    pub fn validate(manifest: &ManifestDocument) -> ManifestResult<()> {
        // Basic validation
        if manifest.meta.name.is_empty() {
            return Err(ManifestError::ValidationError(
                "meta.name cannot be empty".to_string(),
            ));
        }

        if manifest.conventions.namespace_root.is_empty() {
            return Err(ManifestError::ValidationError(
                "conventions.namespaceRoot cannot be empty".to_string(),
            ));
        }

        // Validate apply mode requirements
        use crate::models::ApplyModeKind;
        match manifest.apply.mode {
            ApplyModeKind::Artifact => {
                if manifest.apply.artifact.is_none() {
                    return Err(ManifestError::ValidationError(
                        "apply.artifact section is required for artifact mode".to_string(),
                    ));
                }
            }
            ApplyModeKind::Feature => {
                if manifest.apply.feature.is_none() {
                    return Err(ManifestError::ValidationError(
                        "apply.feature section is required for feature mode".to_string(),
                    ));
                }
            }
            ApplyModeKind::Layer => {
                if manifest.apply.layer.is_none() {
                    return Err(ManifestError::ValidationError(
                        "apply.layer section is required for layer mode".to_string(),
                    ));
                }
            }
        }

        // Validate guards
        if manifest.guards.require_existing_projects {
            tracing::info!(
                "Guard requireExistingProjects=true: existing projects must be present"
            );
        }

        Ok(())
    }
}