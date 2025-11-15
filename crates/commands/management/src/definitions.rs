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
    /// List discovered manifest files in workspace
    #[strum(serialize = "/list")]
    List,

    /// Validate manifest or template structure
    #[strum(serialize = "/check")]
    Check,

    /// Preview template rendering without writing files
    #[strum(serialize = "/render")]
    Render,

    /// Create a new project from a template
    #[strum(serialize = "/new")]
    New,

    /// Apply manifest changes to existing solution
    #[strum(serialize = "/apply")]
    Apply,

    /// Exit NetToolsKit CLI
    #[strum(serialize = "/quit")]
    Quit,
}

impl Command {
    /// Get the user-facing description for this command
    pub fn description(&self) -> &'static str {
        match self {
            Command::List => "List discovered manifest files in workspace",
            Command::Check => "Validate manifest or template structure",
            Command::Render => "Preview template rendering without writing files",
            Command::New => "Create a new project from a template",
            Command::Apply => "Apply manifest changes to existing solution",
            Command::Quit => "Exit NetToolsKit CLI",
        }
    }

    /// Get the slash command string (e.g., "/list")
    pub fn slash(&self) -> &'static str {
        self.into()
    }

    /// Get the command name without slash (e.g., "list")
    pub fn name(&self) -> String {
        match self {
            Command::List => "list".to_string(),
            Command::Check => "check".to_string(),
            Command::Render => "render".to_string(),
            Command::New => "new".to_string(),
            Command::Apply => "apply".to_string(),
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
        assert_eq!(Command::List.slash(), "/list");
        assert_eq!(Command::Quit.slash(), "/quit");
    }

    #[test]
    fn test_command_description() {
        assert!(Command::List.description().contains("manifest"));
        assert!(Command::Quit.description().contains("Exit"));
    }

    #[test]
    fn test_get_command() {
        assert_eq!(get_command("/list"), Some(Command::List));
        assert_eq!(get_command("/invalid"), None);
    }

    #[test]
    fn test_enum_iteration() {
        let commands: Vec<Command> = Command::iter().collect();
        assert_eq!(commands.len(), 6);
    }
}
