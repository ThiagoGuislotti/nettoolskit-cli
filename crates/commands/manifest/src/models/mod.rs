pub mod manifest_action;

pub use manifest_action::{ManifestAction, get_action, palette_entries};

// Re-export all domain models from core under models namespace
// This preserves backward compatibility with tests that import from nettoolskit_manifest::models::*
pub use crate::core::models::*;
