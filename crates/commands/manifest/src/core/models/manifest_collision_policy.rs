//! File collision handling policy

use serde::Deserialize;

/// File collision handling policy
#[derive(Debug, Deserialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum ManifestCollisionPolicy {
    /// Abort on collision.
    Fail,
    /// Silently overwrite the existing file.
    Overwrite,
}
