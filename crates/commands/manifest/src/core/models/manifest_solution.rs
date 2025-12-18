//! Solution configuration

use serde::Deserialize;
use std::path::PathBuf;

/// Solution configuration
#[derive(Debug, Deserialize, Clone)]
pub struct ManifestSolution {
    pub root: PathBuf,
    #[serde(rename = "slnFile")]
    pub sln_file: PathBuf,
}
