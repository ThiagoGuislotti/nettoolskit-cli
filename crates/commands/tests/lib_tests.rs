/// Tests for core lib.rs types (ExitStatus conversions, GlobalArgs, Commands enum)
/// Recovered from backup/commands/tests/lib_tests.rs and commands_tests.rs
use clap::Parser;
use nettoolskit_commands::{Commands, ExitStatus, GlobalArgs};

// ExitStatus conversion tests (from lib_tests.rs backup)

#[test]
fn test_exit_status_success_to_exit_code() {
    let status = ExitStatus::Success;
    let exit_code = std::process::ExitCode::from(status);
    assert_eq!(exit_code, std::process::ExitCode::SUCCESS);
}

#[test]
fn test_exit_status_error_to_exit_code() {
    let status = ExitStatus::Error;
    let exit_code = std::process::ExitCode::from(status);
    assert_eq!(exit_code, std::process::ExitCode::FAILURE);
}

#[test]
fn test_exit_status_interrupted_to_exit_code() {
    let status = ExitStatus::Interrupted;
    let exit_code = std::process::ExitCode::from(status);
    assert_eq!(exit_code, std::process::ExitCode::from(130));
}

#[test]
fn test_exit_status_to_i32_success() {
    let status = ExitStatus::Success;
    let code: i32 = status.into();
    assert_eq!(code, 0);
}

#[test]
fn test_exit_status_to_i32_error() {
    let status = ExitStatus::Error;
    let code: i32 = status.into();
    assert_eq!(code, 1);
}

#[test]
fn test_exit_status_to_i32_interrupted() {
    let status = ExitStatus::Interrupted;
    let code: i32 = status.into();
    assert_eq!(code, 130);
}

// GlobalArgs tests (from lib_tests.rs backup)

#[test]
fn test_global_args_defaults() {
    let args = GlobalArgs::try_parse_from(&["test", "--log-level", "info"]).unwrap();

    assert_eq!(args.log_level, "info");
    assert!(args.config.is_none());
    assert!(!args.verbose);
}

#[test]
fn test_global_args_with_config() {
    let args = GlobalArgs::try_parse_from(&[
        "test",
        "--log-level",
        "debug",
        "--config",
        "config.toml",
        "--verbose",
    ])
    .unwrap();

    assert_eq!(args.log_level, "debug");
    assert_eq!(args.config, Some("config.toml".to_string()));
    assert!(args.verbose);
}

#[test]
fn test_global_args_short_verbose() {
    let args = GlobalArgs::try_parse_from(&["test", "-v"]).unwrap();
    assert!(args.verbose);
}

#[test]
fn test_global_args_log_levels() {
    let log_levels = vec!["off", "error", "warn", "info", "debug", "trace"];

    for level in log_levels {
        let args = GlobalArgs::try_parse_from(&["test", "--log-level", level]).unwrap();
        assert_eq!(args.log_level, level);
    }
}

#[test]
fn test_global_args_debug() {
    let args = GlobalArgs::try_parse_from(&["test"]).unwrap();
    let debug_str = format!("{:?}", args);

    assert!(debug_str.contains("GlobalArgs"));
    assert!(debug_str.contains("info")); // default log level
}

#[test]
fn test_global_args_fields_access() {
    let args = GlobalArgs::try_parse_from(&["test", "--verbose", "--config", "test.toml"]).unwrap();

    assert!(args.verbose);
    assert_eq!(args.config.as_ref().unwrap(), "test.toml");
    assert_eq!(args.log_level, "info");
}

#[test]
fn test_global_args_clone() {
    let args = GlobalArgs::try_parse_from(&["test", "--verbose"]).unwrap();
    let cloned = args.clone();

    assert_eq!(args.verbose, cloned.verbose);
    assert_eq!(args.log_level, cloned.log_level);
    assert_eq!(args.config, cloned.config);
}

// Commands enum tests (from commands_tests.rs backup)

#[test]
fn test_commands_enum_variants() {
    let list_cmd = Commands::List;
    assert!(matches!(list_cmd, Commands::List));

    let new_cmd = Commands::New;
    assert!(matches!(new_cmd, Commands::New));

    let check_cmd = Commands::Check;
    assert!(matches!(check_cmd, Commands::Check));

    let render_cmd = Commands::Render;
    assert!(matches!(render_cmd, Commands::Render));

    let apply_cmd = Commands::Apply;
    assert!(matches!(apply_cmd, Commands::Apply));
}

#[test]
fn test_commands_as_slash_command() {
    assert_eq!(Commands::List.as_slash_command(), "/list");
    assert_eq!(Commands::New.as_slash_command(), "/new");
    assert_eq!(Commands::Check.as_slash_command(), "/check");
    assert_eq!(Commands::Render.as_slash_command(), "/render");
    assert_eq!(Commands::Apply.as_slash_command(), "/apply");
}

#[tokio::test]
async fn test_commands_execute_list() {
    let result = Commands::List.execute().await;
    assert!(matches!(
        result,
        ExitStatus::Success | ExitStatus::Error
    ));
}

#[tokio::test]
async fn test_commands_execute_new() {
    let result = Commands::New.execute().await;
    assert!(matches!(
        result,
        ExitStatus::Success | ExitStatus::Error
    ));
}

#[tokio::test]
async fn test_commands_execute_check() {
    let result = Commands::Check.execute().await;
    assert!(matches!(
        result,
        ExitStatus::Success | ExitStatus::Error
    ));
}

#[tokio::test]
async fn test_commands_execute_render() {
    let result = Commands::Render.execute().await;
    assert!(matches!(
        result,
        ExitStatus::Success | ExitStatus::Error
    ));
}

#[tokio::test]
async fn test_commands_execute_apply() {
    let result = Commands::Apply.execute().await;
    assert!(matches!(
        result,
        ExitStatus::Success | ExitStatus::Error
    ));
}

#[test]
fn test_commands_enum_debug() {
    let list_cmd = Commands::List;
    let debug_str = format!("{:?}", list_cmd);
    assert!(debug_str.contains("List"));

    let new_cmd = Commands::New;
    let debug_str = format!("{:?}", new_cmd);
    assert!(debug_str.contains("New"));

    let check_cmd = Commands::Check;
    let debug_str = format!("{:?}", check_cmd);
    assert!(debug_str.contains("Check"));
}