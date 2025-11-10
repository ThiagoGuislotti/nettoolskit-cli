use nettoolskit_commands::SlashCommand;
use nettoolskit_core::commands::COMMANDS;

#[test]
fn test_slash_commands_complete() {
    // Verify that all commands are present
    assert_eq!(COMMANDS.len(), 6);

    // Verify specific commands
    assert!(COMMANDS.iter().any(|(cmd, _)| cmd == &"/list"));
    assert!(COMMANDS.iter().any(|(cmd, _)| cmd == &"/quit"));
    assert!(COMMANDS.iter().any(|(cmd, _)| cmd == &"/new"));
}

#[test]
fn test_slash_command_descriptions() {
    for (cmd, desc) in COMMANDS {
        assert!(!cmd.is_empty(), "Command should not be empty");
        assert!(cmd.starts_with('/'), "Command should start with '/'");
        assert!(!desc.is_empty(), "Description should not be empty");
    }
}

#[test]
fn test_slash_command_enum() {
    assert_eq!(SlashCommand::List.command(), "list");
    assert_eq!(SlashCommand::Quit.command(), "quit");
    assert_eq!(SlashCommand::New.command(), "new");

    assert_eq!(SlashCommand::List.description(), "List available templates");
    assert_eq!(SlashCommand::Quit.description(), "Exit NetToolsKit CLI");
    assert_eq!(
        SlashCommand::New.description(),
        "Create a project from a template"
    );
}

#[test]
fn test_command_availability_during_task() {
    // Quit should always be available
    assert!(SlashCommand::Quit.available_during_task());

    // Modification commands should not be available during tasks
    assert!(!SlashCommand::New.available_during_task());
    assert!(!SlashCommand::Apply.available_during_task());
    assert!(!SlashCommand::List.available_during_task());
    assert!(!SlashCommand::Check.available_during_task());
    assert!(!SlashCommand::Render.available_during_task());
}
