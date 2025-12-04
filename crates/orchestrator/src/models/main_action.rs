//! Main menu action definitions - SINGLE SOURCE OF TRUTH
//!
//! This module contains the canonical MainAction enum used throughout the CLI main menu.

use nettoolskit_core::{MenuEntry, MenuProvider};
use strum::{Display, EnumIter, EnumString, IntoEnumIterator, IntoStaticStr};

// Re-export ExitStatus from core
pub use nettoolskit_core::ExitStatus;

/// Main menu action enumeration - SINGLE SOURCE OF TRUTH
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Display, EnumIter, EnumString, IntoStaticStr)]
pub enum MainAction {
    /// Display help information and available commands
    #[strum(serialize = "help")]
    Help,

    /// Manage and apply manifests (submenu)
    #[strum(serialize = "manifest")]
    Manifest,

    /// Translate code between languages (deferred)
    #[strum(serialize = "translate")]
    Translate,

    /// Exit NetToolsKit CLI
    #[strum(serialize = "quit")]
    Quit,
}

impl MainAction {
    /// Get the user-facing description for this action
    pub fn description(&self) -> &'static str {
        match self {
            MainAction::Help => "Display help information and available commands",
            MainAction::Manifest => "Manage and apply manifests (submenu)",
            MainAction::Translate => "Translate code between languages (deferred)",
            MainAction::Quit => "Exit NetToolsKit CLI",
        }
    }

    /// Get the slash command string
    pub fn slash(&self) -> String {
        format!("/{}", <&str>::from(self))
    }

    /// Get the slash command as static string
    pub fn slash_static(&self) -> &'static str {
        match self {
            MainAction::Help => "/help",
            MainAction::Manifest => "/manifest",
            MainAction::Translate => "/translate",
            MainAction::Quit => "/quit",
        }
    }
}

impl MenuEntry for MainAction {
    fn label(&self) -> &str {
        self.slash_static()
    }

    fn description(&self) -> &str {
        self.description()
    }
}

/// Implement MenuProvider to enable generic menu rendering
impl MenuProvider for MainAction {
    fn menu_items() -> Vec<String> {
        Self::iter()
            .map(|item| format!("{} - {}", item.label(), item.description()))
            .collect()
    }

    fn all_variants() -> Vec<Self> {
        Self::iter().collect()
    }
}

/// Get main action by slash string (e.g., "/help" -> Some(MainAction::Help))
/// Also handles commands with subcommands (e.g., "/manifest list" -> Some(MainAction::Manifest))
pub fn get_main_action(slash: &str) -> Option<MainAction> {
    use std::str::FromStr;
    // Extract only the first part (the command) and ignore subcommands
    let name = slash.trim()
        .trim_start_matches('/')
        .split_whitespace()
        .next()
        .unwrap_or("")
        .trim();
    MainAction::from_str(name).ok()
}


