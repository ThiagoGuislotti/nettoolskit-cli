//! Manifest subcommand definitions
//!
//! This module defines all subcommands available under /manifest.

use nettoolskit_core::MenuEntry;
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

/// Get all manifest actions as menu entries for UI display
pub fn menu_entries() -> Vec<ManifestAction> {
    ManifestAction::iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use strum::IntoEnumIterator;

    #[test]
    fn test_action_name() {
        // Arrange
        let check = ManifestAction::Check;
        let render = ManifestAction::Render;
        let apply = ManifestAction::Apply;

        // Act
        let check_name = check.name();
        let render_name = render.name();
        let apply_name = apply.name();

        // Assert
        assert_eq!(check_name, "check");
        assert_eq!(render_name, "render");
        assert_eq!(apply_name, "apply");
    }

    #[test]
    fn test_action_description() {
        // Arrange
        let actions = ManifestAction::iter().collect::<Vec<_>>();

        // Act & Assert
        for action in actions {
            assert!(!action.description().is_empty());
        }
    }

    #[test]
    fn test_full_command() {
        // Arrange
        let check = ManifestAction::Check;

        // Act
        let full = check.full_command();

        // Assert
        assert_eq!(full, "/manifest check");
    }

    #[test]
    fn test_get_action() {
        // Act
        let check = get_action("check");
        let render = get_action("render");
        let invalid = get_action("invalid");

        // Assert
        assert_eq!(check, Some(ManifestAction::Check));
        assert_eq!(render, Some(ManifestAction::Render));
        assert_eq!(invalid, None);
    }

    #[test]
    fn test_enum_iteration() {
        // Act
        let actions: Vec<ManifestAction> = ManifestAction::iter().collect();

        // Assert
        assert_eq!(actions.len(), 3); // check, render, apply
    }

    #[test]
    fn test_palette_entries() {
        // Act
        let entries = palette_entries();

        // Assert
        assert_eq!(entries.len(), 3);
        assert!(entries.iter().any(|(name, _)| *name == "check"));
        assert!(entries.iter().any(|(name, _)| *name == "render"));
        assert!(entries.iter().any(|(name, _)| *name == "apply"));
    }

    #[test]
    fn test_menu_entries() {
        // Act
        let entries = menu_entries();

        // Assert
        assert_eq!(entries.len(), 3);
        assert!(entries.contains(&ManifestAction::Check));
        assert!(entries.contains(&ManifestAction::Render));
        assert!(entries.contains(&ManifestAction::Apply));
    }
}
