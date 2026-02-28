//! Manifest models and backward compatibility re-exports.

pub mod manifest_action;

pub use manifest_action::{get_action, ManifestAction};

// Re-export all domain models from core under models namespace
// This preserves backward compatibility with tests that import from nettoolskit_manifest::models::*
pub use crate::core::models::*;
