//! Main menu action definitions - SINGLE SOURCE OF TRUTH
//!
//! This module contains the canonical MainAction enum used throughout the CLI main menu.

use nettoolskit_core::{CommandEntry, MenuEntry, MenuProvider};
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

// Implement CommandEntry to get default name() and slash_static()
impl CommandEntry for MainAction {}

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
}

impl MenuEntry for MainAction {
    fn label(&self) -> &str {
        match self {
            MainAction::Help => "/help",
            MainAction::Manifest => "/manifest",
            MainAction::Translate => "/translate",
            MainAction::Quit => "/quit",
        }
    }

    fn description(&self) -> &str {
        self.description()
    }
}

/// Implement MenuProvider to enable generic menu rendering
impl MenuProvider for MainAction {
    fn menu_items() -> Vec<String> {
        use nettoolskit_ui::core::formatting::format_menu_item;

        Self::iter()
            .map(|item| format_menu_item(item.label(), item.description(), 20))
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


