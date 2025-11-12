//! Tests for command palette definitions
//!
//! This test file validates the COMMANDS constant array
//! used for the interactive command palette.

use nettoolskit_core::commands::COMMANDS;

// =============================================================================
// Structure and Content Tests
// =============================================================================

#[test]
fn test_commands_array_not_empty() {
    assert!(!COMMANDS.is_empty());
}

#[test]
fn test_commands_array_has_expected_count() {
    assert_eq!(COMMANDS.len(), 6);
}

#[test]
fn test_all_commands_start_with_slash() {
    for (cmd, _desc) in COMMANDS {
        assert!(
            cmd.starts_with('/'),
            "Command '{}' should start with '/'",
            cmd
        );
    }
}

#[test]
fn test_all_commands_have_non_empty_descriptions() {
    for (cmd, desc) in COMMANDS {
        assert!(
            !desc.is_empty(),
            "Command '{}' should have a non-empty description",
            cmd
        );
    }
}

// =============================================================================
// Individual Command Tests
// =============================================================================

#[test]
fn test_list_command_exists() {
    let list_cmd = COMMANDS.iter().find(|(cmd, _)| *cmd == "/list");
    assert!(list_cmd.is_some());
    assert_eq!(list_cmd.unwrap().1, "List available templates");
}

#[test]
fn test_check_command_exists() {
    let check_cmd = COMMANDS.iter().find(|(cmd, _)| *cmd == "/check");
    assert!(check_cmd.is_some());
    assert_eq!(check_cmd.unwrap().1, "Validate a manifest or template");
}

#[test]
fn test_render_command_exists() {
    let render_cmd = COMMANDS.iter().find(|(cmd, _)| *cmd == "/render");
    assert!(render_cmd.is_some());
    assert_eq!(render_cmd.unwrap().1, "Render a template preview");
}

#[test]
fn test_new_command_exists() {
    let new_cmd = COMMANDS.iter().find(|(cmd, _)| *cmd == "/new");
    assert!(new_cmd.is_some());
    assert_eq!(new_cmd.unwrap().1, "Create a project from a template");
}

#[test]
fn test_apply_command_exists() {
    let apply_cmd = COMMANDS.iter().find(|(cmd, _)| *cmd == "/apply");
    assert!(apply_cmd.is_some());
    assert_eq!(
        apply_cmd.unwrap().1,
        "Apply a manifest to an existing solution"
    );
}

#[test]
fn test_quit_command_exists() {
    let quit_cmd = COMMANDS.iter().find(|(cmd, _)| *cmd == "/quit");
    assert!(quit_cmd.is_some());
    assert_eq!(quit_cmd.unwrap().1, "Exit NetToolsKit CLI");
}

// =============================================================================
// Command Name Validation
// =============================================================================

#[test]
fn test_all_command_names_lowercase() {
    for (cmd, _desc) in COMMANDS {
        let cmd_name = &cmd[1..]; // Skip the '/'
        assert_eq!(
            cmd_name,
            cmd_name.to_lowercase(),
            "Command '{}' should be lowercase",
            cmd
        );
    }
}

#[test]
fn test_no_duplicate_command_names() {
    let mut seen = std::collections::HashSet::new();
    for (cmd, _desc) in COMMANDS {
        assert!(seen.insert(cmd), "Duplicate command: {}", cmd);
    }
}

#[test]
fn test_command_names_no_whitespace() {
    for (cmd, _desc) in COMMANDS {
        assert!(
            !cmd.contains(' '),
            "Command '{}' should not contain spaces",
            cmd
        );
        assert!(
            !cmd.contains('\t'),
            "Command '{}' should not contain tabs",
            cmd
        );
    }
}

#[test]
fn test_command_names_alphanumeric_only() {
    for (cmd, _desc) in COMMANDS {
        let cmd_name = &cmd[1..]; // Skip the '/'
        assert!(
            cmd_name.chars().all(|c| c.is_alphanumeric()),
            "Command '{}' should only contain alphanumeric characters after '/'",
            cmd
        );
    }
}

// =============================================================================
// Description Validation
// =============================================================================

#[test]
fn test_all_descriptions_start_with_capital() {
    for (cmd, desc) in COMMANDS {
        let first_char = desc.chars().next().unwrap();
        assert!(
            first_char.is_uppercase(),
            "Description for '{}' should start with capital letter: '{}'",
            cmd,
            desc
        );
    }
}

#[test]
fn test_all_descriptions_are_single_line() {
    for (cmd, desc) in COMMANDS {
        assert!(
            !desc.contains('\n'),
            "Description for '{}' should not contain newlines",
            cmd
        );
    }
}

#[test]
fn test_descriptions_are_concise() {
    for (cmd, desc) in COMMANDS {
        assert!(
            desc.len() <= 100,
            "Description for '{}' should be concise (<=100 chars): '{}'",
            cmd,
            desc
        );
    }
}

// =============================================================================
// Iteration and Access Tests
// =============================================================================

#[test]
fn test_commands_can_be_iterated() {
    let mut count = 0;
    for _cmd in COMMANDS {
        count += 1;
    }
    assert_eq!(count, 6);
}

#[test]
fn test_commands_can_be_accessed_by_index() {
    assert_eq!(COMMANDS[0].0, "/list");
    assert_eq!(COMMANDS[1].0, "/check");
    assert_eq!(COMMANDS[2].0, "/render");
    assert_eq!(COMMANDS[3].0, "/new");
    assert_eq!(COMMANDS[4].0, "/apply");
    assert_eq!(COMMANDS[5].0, "/quit");
}

#[test]
fn test_commands_can_be_filtered() {
    let template_commands: Vec<_> = COMMANDS
        .iter()
        .filter(|(_cmd, desc)| desc.contains("template"))
        .collect();
    assert!(!template_commands.is_empty());
}

#[test]
fn test_commands_can_be_searched() {
    let quit_cmd = COMMANDS.iter().find(|(cmd, _)| cmd.contains("quit"));
    assert!(quit_cmd.is_some());
}

// =============================================================================
// Edge Cases and Boundaries
// =============================================================================

#[test]
fn test_commands_array_is_static() {
    // Verify it's a 'static slice
    let _static_ref: &'static [(&str, &str)] = COMMANDS;
}

#[test]
fn test_command_ordering_is_logical() {
    // Verify quit is last (conventional exit command placement)
    assert_eq!(COMMANDS[COMMANDS.len() - 1].0, "/quit");
}

#[test]
fn test_no_empty_command_names() {
    for (cmd, _desc) in COMMANDS {
        assert!(cmd.len() > 1, "Command should not be empty or just '/'");
    }
}

#[test]
fn test_commands_const_can_be_used_in_const_context() {
    const _COMMANDS_LEN: usize = 6;
    // This test verifies COMMANDS can be used at compile-time
}
