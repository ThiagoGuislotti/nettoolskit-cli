use clap::Parser;
use nettoolskit_commands::{ExitStatus, GlobalArgs};

#[test]
fn test_exit_status_success_conversion() {
    let status = ExitStatus::Success;
    let exit_code = std::process::ExitCode::from(status);
    assert_eq!(exit_code, std::process::ExitCode::SUCCESS);
}

#[test]
fn test_exit_status_error_conversion() {
    let status = ExitStatus::Error;
    let exit_code = std::process::ExitCode::from(status);
    assert_eq!(exit_code, std::process::ExitCode::FAILURE);
}

#[test]
fn test_exit_status_interrupted_conversion() {
    let status = ExitStatus::Interrupted;
    let exit_code = std::process::ExitCode::from(status);
    assert_eq!(exit_code, std::process::ExitCode::from(130));
}

#[test]
fn test_exit_status_debug() {
    let status = ExitStatus::Success;
    let debug_str = format!("{:?}", status);
    assert!(debug_str.contains("Success"));

    let status = ExitStatus::Error;
    let debug_str = format!("{:?}", status);
    assert!(debug_str.contains("Error"));

    let status = ExitStatus::Interrupted;
    let debug_str = format!("{:?}", status);
    assert!(debug_str.contains("Interrupted"));
}

#[test]
fn test_exit_status_clone_copy() {
    let original = ExitStatus::Success;
    let cloned = original.clone();
    let copied = original;

    assert!(matches!(original, ExitStatus::Success));
    assert!(matches!(cloned, ExitStatus::Success));
    assert!(matches!(copied, ExitStatus::Success));
}

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
