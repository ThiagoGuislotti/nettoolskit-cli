use strum::IntoEnumIterator;
use strum_macros::AsRefStr;
use strum_macros::EnumIter;
use strum_macros::EnumString;
use strum_macros::IntoStaticStr;

/// Commands that can be invoked by starting a message with a leading slash.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, EnumString, EnumIter, AsRefStr, IntoStaticStr,
)]
#[strum(serialize_all = "kebab-case")]
pub enum SlashCommand {
    // DO NOT ALPHA-SORT! Enum order is presentation order in the popup, so
    // more frequently used commands should be listed first.
    List,
    New,
    Check,
    Render,
    Apply,
    Quit,
}

impl SlashCommand {
    /// User-visible description shown in the popup.
    pub fn description(self) -> &'static str {
        match self {
            SlashCommand::List => "List available templates",
            SlashCommand::New => "Create a project from a template",
            SlashCommand::Check => "Validate a manifest or template",
            SlashCommand::Render => "Render a template preview",
            SlashCommand::Apply => "Apply a manifest to an existing solution",
            SlashCommand::Quit => "Exit NetToolsKit CLI",
        }
    }

    /// Command string without the leading '/'. Provided for compatibility with
    /// existing code that expects a method named `command()`.
    pub fn command(self) -> &'static str {
        self.into()
    }

    /// Whether this command can be run while a task is in progress.
    pub fn available_during_task(self) -> bool {
        match self {
            SlashCommand::List
            | SlashCommand::New
            | SlashCommand::Check
            | SlashCommand::Render
            | SlashCommand::Apply => false,
            SlashCommand::Quit => true,
        }
    }
}

/// Return all built-in commands in a Vec paired with their command string.
pub fn built_in_slash_commands() -> Vec<(&'static str, SlashCommand)> {
    SlashCommand::iter().map(|c| (c.command(), c)).collect()
}

/// Constants for the command palette as specified in paleta-codex.md
pub const COMMANDS: &[(&str, &str)] = &[
    ("/list", "List available templates"),
    ("/check", "Validate a manifest or template"),
    ("/render", "Render a template preview"),
    ("/new", "Create a project from a template"),
    ("/apply", "Apply a manifest to an existing solution"),
    ("/quit", "Exit NetToolsKit CLI"),
];