//! `CommandEntry` trait tests
//!
//! Tests for command-like menu entries with slash notation.

use nettoolskit_core::{CommandEntry, MenuEntry};
use strum::{EnumIter, IntoStaticStr};

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, IntoStaticStr)]
#[strum(serialize_all = "lowercase")]
enum TestCommand {
    Help,
    Manifest,
    Template,
}

impl MenuEntry for TestCommand {
    fn label(&self) -> &str {
        match self {
            Self::Help => "help",
            Self::Manifest => "manifest",
            Self::Template => "template",
        }
    }

    fn description(&self) -> &str {
        match self {
            Self::Help => "Show help",
            Self::Manifest => "Manifest operations",
            Self::Template => "Template operations",
        }
    }
}

impl CommandEntry for TestCommand {}

#[test]
fn test_command_entry_name() {
    assert_eq!(TestCommand::Help.name(), "help");
    assert_eq!(TestCommand::Manifest.name(), "manifest");
    assert_eq!(TestCommand::Template.name(), "template");
}

#[test]
fn test_command_entry_slash_static() {
    assert_eq!(TestCommand::Help.slash_static(), "/help");
    assert_eq!(TestCommand::Manifest.slash_static(), "/manifest");
    assert_eq!(TestCommand::Template.slash_static(), "/template");
}

#[test]
fn test_command_entry_implements_menu_entry() {
    let cmd = TestCommand::Help;
    assert_eq!(cmd.label(), "help");
    assert_eq!(cmd.description(), "Show help");
    assert_eq!(cmd.name(), "help");
    assert_eq!(cmd.slash_static(), "/help");
}
