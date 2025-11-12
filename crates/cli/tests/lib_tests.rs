#![allow(clippy::assertions_on_constants)]

use nettoolskit_commands::ExitStatus;

#[test]
fn test_exit_status_variants() {
    // Arrange
    let success = ExitStatus::Success;
    let error = ExitStatus::Error;
    let interrupted = ExitStatus::Interrupted;

    // Assert
    assert!(matches!(success, ExitStatus::Success));
    assert!(matches!(error, ExitStatus::Error));
    assert!(matches!(interrupted, ExitStatus::Interrupted));
}

#[test]
fn test_exit_status_debug() {
    // Arrange & Act
    let status = ExitStatus::Success;
    let debug_str = format!("{:?}", status);

    // Assert
    assert!(debug_str.contains("Success"));

    // Arrange & Act
    let status = ExitStatus::Error;
    let debug_str = format!("{:?}", status);

    // Assert
    assert!(debug_str.contains("Error"));

    // Arrange & Act
    let status = ExitStatus::Interrupted;
    let debug_str = format!("{:?}", status);

    // Assert
    assert!(debug_str.contains("Interrupted"));
}

#[test]
fn test_exit_status_clone_copy() {
    // Arrange
    let original = ExitStatus::Success;

    // Act
    let cloned = original; // ExitStatus is Copy, no need for .clone()
    let copied = original;

    // Assert
    assert!(matches!(original, ExitStatus::Success));
    assert!(matches!(cloned, ExitStatus::Success));
    assert!(matches!(copied, ExitStatus::Success));
}

#[test]
fn test_exit_status_to_exit_code() {
    // Arrange & Act
    let success_code = std::process::ExitCode::from(ExitStatus::Success);

    // Assert
    assert_eq!(success_code, std::process::ExitCode::SUCCESS);

    // Arrange & Act
    let error_code = std::process::ExitCode::from(ExitStatus::Error);

    // Assert
    assert_eq!(error_code, std::process::ExitCode::FAILURE);

    // Arrange & Act
    let interrupted_code = std::process::ExitCode::from(ExitStatus::Interrupted);

    // Assert
    assert_eq!(interrupted_code, std::process::ExitCode::from(130));
}

#[test]
fn test_exit_status_all_variants_covered() {
    // Arrange
    let _success = ExitStatus::Success;
    let _error = ExitStatus::Error;
    let _interrupted = ExitStatus::Interrupted;

    // Assert
    // Test ensures all variants are constructible
    assert!(true);
}

#[tokio::test]
async fn test_interactive_mode_function_exists() {
    // Arrange
    use nettoolskit_cli::interactive_mode;

    fn check_signature() -> impl std::future::Future<Output = ExitStatus> {
        interactive_mode(false)
    }

    // Act
    let _future = check_signature();

    // Assert
    // Test ensures function signature is correct
    assert!(true);
}

#[test]
fn test_module_imports() {
    // Arrange & Act
    let _status = ExitStatus::Success;
    use nettoolskit_cli::interactive_mode;
    let _future = interactive_mode(false);

    // Assert
    // Test ensures all modules are importable
    assert!(true);
}

#[test]
fn test_exit_status_pattern_matching() {
    // Arrange
    let statuses = vec![
        ExitStatus::Success,
        ExitStatus::Error,
        ExitStatus::Interrupted,
    ];

    // Act & Assert
    // Test ensures exhaustive pattern matching works for all variants
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
    // Arrange & Act
    let success_result = match ExitStatus::Success {
        ExitStatus::Success => "success",
        _ => "other",
    };

    // Assert
    assert_eq!(success_result, "success");

    // Arrange & Act
    let error_result = match ExitStatus::Error {
        ExitStatus::Error => "error",
        _ => "other",
    };

    // Assert
    assert_eq!(error_result, "error");

    // Arrange & Act
    let interrupted_result = match ExitStatus::Interrupted {
        ExitStatus::Interrupted => "interrupted",
        _ => "other",
    };

    // Assert
    assert_eq!(interrupted_result, "interrupted");
}
