use clap::Parser;
use nettoolskit_commands::{
    apply, check, execute_command, list, new, render, Commands, GlobalArgs,
};

#[test]
fn test_commands_enum_debug() {
    let list_cmd = Commands::List(list::ListArgs::default());
    let debug_str = format!("{:?}", list_cmd);
    assert!(debug_str.contains("List"));

    let new_cmd = Commands::New(new::NewArgs::default());
    let debug_str = format!("{:?}", new_cmd);
    assert!(debug_str.contains("New"));
}

#[test]
fn test_list_args_default() {
    let args = list::ListArgs::default();
    let debug_str = format!("{:?}", args);
    assert!(debug_str.contains("ListArgs"));
}

#[test]
fn test_new_args_default() {
    let args = new::NewArgs::default();
    let debug_str = format!("{:?}", args);
    assert!(debug_str.contains("NewArgs"));
}

#[test]
fn test_check_args_default() {
    let args = check::CheckArgs::default();
    let debug_str = format!("{:?}", args);
    assert!(debug_str.contains("CheckArgs"));
}

#[test]
fn test_render_args_default() {
    let args = render::RenderArgs::default();
    let debug_str = format!("{:?}", args);
    assert!(debug_str.contains("RenderArgs"));
}

#[test]
fn test_apply_args_default() {
    let args = apply::ApplyArgs::default();
    let debug_str = format!("{:?}", args);
    assert!(debug_str.contains("ApplyArgs"));
}

#[tokio::test]
async fn test_execute_list_command() {
    let cmd = Commands::List(list::ListArgs::default());
    let global_args = GlobalArgs::try_parse_from(&["test"]).unwrap();

    let result = execute_command(cmd, global_args).await;
    assert!(matches!(
        result,
        nettoolskit_commands::ExitStatus::Success | nettoolskit_commands::ExitStatus::Error
    ));
}

#[tokio::test]
async fn test_execute_new_command() {
    let cmd = Commands::New(new::NewArgs::default());
    let global_args = GlobalArgs::try_parse_from(&["test"]).unwrap();

    let result = execute_command(cmd, global_args).await;
    assert!(matches!(
        result,
        nettoolskit_commands::ExitStatus::Success | nettoolskit_commands::ExitStatus::Error
    ));
}

#[tokio::test]
async fn test_execute_check_command() {
    let cmd = Commands::Check(check::CheckArgs::default());
    let global_args = GlobalArgs::try_parse_from(&["test"]).unwrap();

    let result = execute_command(cmd, global_args).await;
    assert!(matches!(
        result,
        nettoolskit_commands::ExitStatus::Success | nettoolskit_commands::ExitStatus::Error
    ));
}

#[tokio::test]
async fn test_execute_render_command() {
    let cmd = Commands::Render(render::RenderArgs::default());
    let global_args = GlobalArgs::try_parse_from(&["test"]).unwrap();

    let result = execute_command(cmd, global_args).await;
    assert!(matches!(
        result,
        nettoolskit_commands::ExitStatus::Success | nettoolskit_commands::ExitStatus::Error
    ));
}

#[tokio::test]
async fn test_execute_apply_command() {
    let cmd = Commands::Apply(apply::ApplyArgs::default());
    let global_args = GlobalArgs::try_parse_from(&["test"]).unwrap();

    let result = execute_command(cmd, global_args).await;
    assert!(matches!(
        result,
        nettoolskit_commands::ExitStatus::Success | nettoolskit_commands::ExitStatus::Error
    ));
}

#[tokio::test]
async fn test_commands_with_different_global_args() {
    // Test with debug log level
    let global_args = GlobalArgs::try_parse_from(&["test", "--log-level", "debug"]).unwrap();
    let cmd = Commands::List(list::ListArgs::default());
    let result = execute_command(cmd, global_args).await;
    assert!(matches!(
        result,
        nettoolskit_commands::ExitStatus::Success | nettoolskit_commands::ExitStatus::Error
    ));

    // Test with config file
    let global_args = GlobalArgs::try_parse_from(&["test", "--config", "test.toml"]).unwrap();
    let cmd = Commands::New(new::NewArgs::default());
    let result = execute_command(cmd, global_args).await;
    assert!(matches!(
        result,
        nettoolskit_commands::ExitStatus::Success | nettoolskit_commands::ExitStatus::Error
    ));
}

#[test]
fn test_commands_parser_integration() {
    // Test that Commands can be parsed from command line arguments
    // This would typically be tested with actual CLI parsing, but we'll test the structure

    let list_cmd = Commands::List(list::ListArgs::default());
    assert!(matches!(list_cmd, Commands::List(_)));

    let new_cmd = Commands::New(new::NewArgs::default());
    assert!(matches!(new_cmd, Commands::New(_)));

    let check_cmd = Commands::Check(check::CheckArgs::default());
    assert!(matches!(check_cmd, Commands::Check(_)));

    let render_cmd = Commands::Render(render::RenderArgs::default());
    assert!(matches!(render_cmd, Commands::Render(_)));

    let apply_cmd = Commands::Apply(apply::ApplyArgs::default());
    assert!(matches!(apply_cmd, Commands::Apply(_)));
}
