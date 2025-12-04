//! Manifest subcommand definitions
//!
//! This module defines all subcommands available under /manifest.

use nettoolskit_core::{MenuEntry, MenuProvider};
use strum::{Display, EnumIter, EnumString, IntoEnumIterator, IntoStaticStr};

/// Manifest subcommand enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Display, EnumIter, EnumString, IntoStaticStr)]
#[strum(serialize_all = "lowercase")]
pub enum ManifestAction {
    /// Validate manifest file syntax and structure
    #[strum(serialize = "check")]
    Check,

    /// Preview generated files without creating them
    #[strum(serialize = "render")]
    Render,

    /// Apply manifest to generate code artifacts
    #[strum(serialize = "apply")]
    Apply,
}

impl ManifestAction {
    /// Get the user-facing description for this action
    pub fn description(&self) -> &'static str {
        match self {
            ManifestAction::Check => "Validate manifest file syntax and structure",
            ManifestAction::Render => "Preview generated files without creating them",
            ManifestAction::Apply => "Apply manifest to generate code artifacts",
        }
    }

    /// Get the action name (e.g., "check")
    pub fn name(&self) -> &'static str {
        self.into()
    }

    /// Get the full command with parent (e.g., "/manifest check")
    pub fn full_command(&self) -> String {
        format!("/manifest {}", self.name())
    }
}

impl MenuEntry for ManifestAction {
    fn label(&self) -> &str {
        self.name()
    }

    fn description(&self) -> &str {
        self.description()
    }
}

/// Implement MenuProvider to enable generic menu rendering
impl MenuProvider for ManifestAction {
    fn menu_items() -> Vec<String> {
        Self::iter()
            .map(|item| format!("{} - {}", item.label(), item.description()))
            .collect()
    }

    fn all_variants() -> Vec<Self> {
        Self::iter().collect()
    }
}

/// Get manifest action by name (e.g., "check" -> Some(ManifestAction::Check))
pub fn get_action(name: &str) -> Option<ManifestAction> {
    use std::str::FromStr;
    ManifestAction::from_str(name).ok()
}

/// Get all manifest action entries for palette display
pub fn palette_entries() -> Vec<(&'static str, &'static str)> {
    ManifestAction::iter()
        .map(|action| (action.name(), action.description()))
        .collect()
}
