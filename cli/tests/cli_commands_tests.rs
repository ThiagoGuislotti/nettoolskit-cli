use nettoolskit_commands::SlashCommand;
use nettoolskit_core::commands::COMMANDS;

#[test]
fn test_slash_commands_complete() {
    // Verifica se todos os comandos estão presentes
    assert_eq!(COMMANDS.len(), 6);

    // Verifica comandos específicos
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
    assert_eq!(SlashCommand::New.description(), "Create a project from a template");
}

#[test]
fn test_command_availability_during_task() {
    // Quit deve estar sempre disponível
    assert!(SlashCommand::Quit.available_during_task());

    // Comandos de modificação não devem estar disponíveis durante tarefas
    assert!(!SlashCommand::New.available_during_task());
    assert!(!SlashCommand::Apply.available_during_task());
    assert!(!SlashCommand::List.available_during_task());
    assert!(!SlashCommand::Check.available_during_task());
    assert!(!SlashCommand::Render.available_during_task());
}