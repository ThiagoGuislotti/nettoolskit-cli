use nettoolskit_cli::slash_command::{SlashCommand, COMMANDS};

#[test]
fn test_slash_commands_complete() {
    // Verifica se todos os comandos estão presentes
    assert_eq!(COMMANDS.len(), 7);

    // Verifica comandos específicos
    assert!(COMMANDS.iter().any(|(cmd, _)| cmd == &"/list"));
    assert!(COMMANDS.iter().any(|(cmd, _)| cmd == &"/quit"));
    assert!(COMMANDS.iter().any(|(cmd, _)| cmd == &"/help"));
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
    assert_eq!(SlashCommand::Help.command(), "help");

    assert_eq!(SlashCommand::List.description(), "List available templates");
    assert_eq!(SlashCommand::Quit.description(), "Exit NetToolsKit CLI");
}

#[test]
fn test_command_availability_during_task() {
    // Help e Quit devem estar sempre disponíveis
    assert!(SlashCommand::Help.available_during_task());
    assert!(SlashCommand::Quit.available_during_task());

    // Comandos de modificação não devem estar disponíveis durante tarefas
    assert!(!SlashCommand::New.available_during_task());
    assert!(!SlashCommand::Apply.available_during_task());
}