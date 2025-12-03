//! Command definitions - SINGLE SOURCE OF TRUTH
//!
//! This module contains the canonical Command enum used throughout the CLI.

use nettoolskit_core::MenuEntry;
use strum::{Display, EnumIter, EnumString, IntoEnumIterator, IntoStaticStr};

// Re-export ExitStatus from core
pub use nettoolskit_core::ExitStatus;

/// Command enumeration - SINGLE SOURCE OF TRUTH
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Display, EnumIter, EnumString, IntoStaticStr)]
pub enum Command {
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

impl Command {
    /// Get the user-facing description for this command
    pub fn description(&self) -> &'static str {
        match self {
            Command::Help => "Display help information and available commands",
            Command::Manifest => "Manage and apply manifests (submenu)",
            Command::Translate => "Translate code between languages (deferred)",
            Command::Quit => "Exit NetToolsKit CLI",
        }
    }

    /// Get the slash command string
    pub fn slash(&self) -> String {
        format!("/ {}", <&str>::from(self))
    }

    /// Get the slash command as static string
    pub fn slash_static(&self) -> &'static str {
        match self {
            Command::Help => "/ help",
            Command::Manifest => "/ manifest",
            Command::Translate => "/ translate",
            Command::Quit => "/ quit",
        }
    }
}

impl MenuEntry for Command {
    fn label(&self) -> &str {
        self.slash_static()
    }

    fn description(&self) -> &str {
        self.description()
    }
}

/// Get command by slash string (e.g., "/ help" -> Some(Command::Help))
pub fn get_command(slash: &str) -> Option<Command> {
    use std::str::FromStr;
    let name = slash.trim().trim_start_matches('/').trim();
    Command::from_str(name).ok()
}

/// Get all commands as menu entries for UI display
pub fn menu_entries() -> Vec<Command> {
    Command::iter().collect()
}
