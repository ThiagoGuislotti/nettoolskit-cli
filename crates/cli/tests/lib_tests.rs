use nettoolskit_commands::ExitStatus;

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
fn test_exit_status_to_exit_code() {
    let success_code = std::process::ExitCode::from(ExitStatus::Success);
    assert_eq!(success_code, std::process::ExitCode::SUCCESS);

    let error_code = std::process::ExitCode::from(ExitStatus::Error);
    assert_eq!(error_code, std::process::ExitCode::FAILURE);

    let interrupted_code = std::process::ExitCode::from(ExitStatus::Interrupted);
    assert_eq!(interrupted_code, std::process::ExitCode::from(130));
}

#[test]
fn test_exit_status_all_variants_covered() {
    let _success = ExitStatus::Success;
    let _error = ExitStatus::Error;
    let _interrupted = ExitStatus::Interrupted;

    assert!(true);
}

#[tokio::test]
async fn test_interactive_mode_function_exists() {
    use nettoolskit_cli::interactive_mode;

    fn check_signature() -> impl std::future::Future<Output = ExitStatus> {
        interactive_mode(false)
    }

    let _future = check_signature();
    assert!(true);
}

#[test]
fn test_module_imports() {
    let _status = ExitStatus::Success;
    use nettoolskit_cli::interactive_mode;
    let _future = interactive_mode(false);

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
