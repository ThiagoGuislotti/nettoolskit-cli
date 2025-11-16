//! Command definitions - SINGLE SOURCE OF TRUTH
//!
//! This module contains the canonical Command enum and ExitStatus used throughout the CLI.

use nettoolskit_core::MenuEntry;
use strum::{Display, EnumIter, EnumString, IntoEnumIterator, IntoStaticStr};

/// Exit status for command execution
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitStatus {
    /// Command executed successfully
    Success,
    /// Command execution failed
    Error,
    /// Command execution was interrupted
    Interrupted,
}

impl From<ExitStatus> for i32 {
    fn from(status: ExitStatus) -> Self {
        match status {
            ExitStatus::Success => 0,
            ExitStatus::Error => 1,
            ExitStatus::Interrupted => 130,
        }
    }
}

impl From<ExitStatus> for std::process::ExitCode {
    fn from(status: ExitStatus) -> Self {
        std::process::ExitCode::from(i32::from(status) as u8)
    }
}

/// Command enumeration - SINGLE SOURCE OF TRUTH
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Display, EnumIter, EnumString, IntoStaticStr)]
#[strum(serialize_all = "lowercase")]
pub enum Command {
    /// Display help information and available commands
    #[strum(serialize = "/help")]
    Help,

    /// Manage and apply manifests (submenu)
    #[strum(serialize = "/manifest")]
    Manifest,

    /// Translate code between languages (deferred)
    #[strum(serialize = "/translate")]
    Translate,

    /// Exit NetToolsKit CLI
    #[strum(serialize = "/quit")]
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

    /// Get the slash command string (e.g., "/list")
    pub fn slash(&self) -> &'static str {
        self.into()
    }

    /// Get the command name without slash (e.g., "help")
    pub fn name(&self) -> String {
        match self {
            Command::Help => "help".to_string(),
            Command::Manifest => "manifest".to_string(),
            Command::Translate => "translate".to_string(),
            Command::Quit => "quit".to_string(),
        }
    }
}

impl MenuEntry for Command {
    fn label(&self) -> &str {
        self.slash()
    }

    fn description(&self) -> &str {
        self.description()
    }
}

/// Get command by slash string (e.g., "/list" -> Some(Command::List))
pub fn get_command(slash: &str) -> Option<Command> {
    use std::str::FromStr;
    Command::from_str(slash).ok()
}

/// Get all command entries for palette display
pub fn palette_entries() -> Vec<(&'static str, &'static str)> {
    Command::iter()
        .map(|cmd| (cmd.slash(), cmd.description()))
        .collect()
}

/// Get all commands as menu entries for UI display
pub fn menu_entries() -> Vec<Command> {
    Command::iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use strum::IntoEnumIterator;

    #[test]
    fn test_command_slash() {
        assert_eq!(Command::Help.slash(), "/help");
        assert_eq!(Command::Quit.slash(), "/quit");
    }

    #[test]
    fn test_command_description() {
        assert!(Command::Help.description().contains("help"));
        assert!(Command::Manifest.description().contains("manifests"));
        assert!(Command::Translate.description().contains("Translate"));
        assert!(Command::Quit.description().contains("Exit"));
    }

    #[test]
    fn test_get_command() {
        assert_eq!(get_command("/help"), Some(Command::Help));
        assert_eq!(get_command("/invalid"), None);
    }

    #[test]
    fn test_enum_iteration() {
        let commands: Vec<Command> = Command::iter().collect();
        assert_eq!(commands.len(), 4); // help, manifest, translate, quit
    }
}
