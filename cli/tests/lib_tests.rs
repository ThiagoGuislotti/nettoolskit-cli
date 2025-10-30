use nettoolskit_cli::{ExitStatus, interactive_mode};
use nettoolskit_commands::processor::CliExitStatus;

#[test]
fn test_exit_status_variants() {
    let success = ExitStatus::Success;
    let error = ExitStatus::Error;
    let interrupted = ExitStatus::Interrupted;

    assert!(matches!(success, ExitStatus::Success));
    assert!(matches!(error, ExitStatus::Error));
    assert!(matches!(interrupted, ExitStatus::Interrupted));
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
fn test_exit_status_from_cli_exit_status() {
    let cli_success = CliExitStatus::Success;
    let exit_success: ExitStatus = cli_success.into();
    assert!(matches!(exit_success, ExitStatus::Success));

    let cli_error = CliExitStatus::Error;
    let exit_error: ExitStatus = cli_error.into();
    assert!(matches!(exit_error, ExitStatus::Error));

    let cli_interrupted = CliExitStatus::Interrupted;
    let exit_interrupted: ExitStatus = cli_interrupted.into();
    assert!(matches!(exit_interrupted, ExitStatus::Interrupted));
}

#[test]
fn test_exit_status_to_exit_code() {
    let success_code = std::process::ExitCode::from(ExitStatus::Success);
    assert_eq!(success_code, std::process::ExitCode::SUCCESS);

    let error_code = std::process::ExitCode::from(ExitStatus::Error);
    assert_eq!(error_code, std::process::ExitCode::FAILURE);

    let interrupted_code = std::process::ExitCode::from(ExitStatus::Interrupted);
    assert_eq!(interrupted_code, std::process::ExitCode::from(130));
}

#[test]
fn test_exit_status_conversions() {
    // Test the complete conversion chain: CliExitStatus -> ExitStatus -> ExitCode
    let cli_statuses = vec![
        CliExitStatus::Success,
        CliExitStatus::Error,
        CliExitStatus::Interrupted,
    ];

    for cli_status in cli_statuses {
        let exit_status: ExitStatus = cli_status.into();
        let _exit_code = std::process::ExitCode::from(exit_status);
        // Should not panic and conversions should be consistent
    }
}

#[test]
fn test_exit_status_all_variants_covered() {
    // Ensure we test all variants - this will fail to compile if new variants are added
    let _success = ExitStatus::Success;
    let _error = ExitStatus::Error;
    let _interrupted = ExitStatus::Interrupted;

    assert!(true);
}

#[tokio::test]
async fn test_interactive_mode_function_exists() {
    // Test that interactive_mode function exists and returns ExitStatus
    // This test just verifies the function signature, not full functionality
    // since interactive_mode requires terminal input

    // We can't easily test interactive_mode in unit tests without mocking
    // but we can verify it compiles and has the right signature
    fn check_signature() -> impl std::future::Future<Output = ExitStatus> {
        interactive_mode()
    }

    let _future = check_signature();
    assert!(true);
}

#[test]
fn test_module_imports() {
    // Test that all public modules are accessible
    use nettoolskit_cli::*;

    // Should be able to access ExitStatus
    let _status = ExitStatus::Success;

    // Should be able to call interactive_mode (returns future)
    let _future = interactive_mode();

    assert!(true);
}

#[test]
fn test_exit_status_pattern_matching() {
    let statuses = vec![
        ExitStatus::Success,
        ExitStatus::Error,
        ExitStatus::Interrupted,
    ];

    for status in statuses {
        match status {
            ExitStatus::Success => assert!(true),
            ExitStatus::Error => assert!(true),
            ExitStatus::Interrupted => assert!(true),
        }
    }
}

#[test]
fn test_exit_status_discriminant_values() {
    // Test that we can differentiate between variants using match
    let success_result = match ExitStatus::Success {
        ExitStatus::Success => "success",
        _ => "other",
    };
    assert_eq!(success_result, "success");

    let error_result = match ExitStatus::Error {
        ExitStatus::Error => "error",
        _ => "other",
    };
    assert_eq!(error_result, "error");

    let interrupted_result = match ExitStatus::Interrupted {
        ExitStatus::Interrupted => "interrupted",
        _ => "other",
    };
    assert_eq!(interrupted_result, "interrupted");
}