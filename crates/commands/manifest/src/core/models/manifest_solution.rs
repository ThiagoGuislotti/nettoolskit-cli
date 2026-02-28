//! Solution configuration

use serde::Deserialize;
use std::path::PathBuf;

/// Solution configuration
#[derive(Debug, Deserialize, Clone)]
pub struct ManifestSolution {
    /// Solution root directory path.
    pub root: PathBuf,
    /// Path to the `.sln` file.
    #[serde(rename = "slnFile")]
    pub sln_file: PathBuf,
}
