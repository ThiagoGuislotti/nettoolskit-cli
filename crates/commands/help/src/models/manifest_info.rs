//! Manifest information model

use std::path::PathBuf;

/// Information about a discovered manifest file
#[derive(Debug, Clone)]
pub struct ManifestInfo {
    /// Path to the manifest file
    pub path: PathBuf,
    /// Project name from manifest metadata
    pub project_name: String,
    /// Target language/framework
    pub language: String,
    /// Number of contexts in the manifest
    pub context_count: usize,
}
