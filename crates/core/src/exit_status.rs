//! Exit status codes for command execution
//!
//! Provides standardized exit status codes used across the CLI.

/// Exit status for command execution
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitStatus {
    /// Command executed successfully
    Success,
    /// Command execution failed
    Error,
    /// Command execution was interrupted
    Interrupted,
}

impl From<ExitStatus> for i32 {
    fn from(status: ExitStatus) -> Self {
        match status {
            ExitStatus::Success => 0,
            ExitStatus::Error => 1,
            ExitStatus::Interrupted => 130,
        }
    }
}

impl From<ExitStatus> for std::process::ExitCode {
    fn from(status: ExitStatus) -> Self {
        std::process::ExitCode::from(i32::from(status) as u8)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exit_status_to_i32() {
        // Arrange
        let success = ExitStatus::Success;
        let error = ExitStatus::Error;
        let interrupted = ExitStatus::Interrupted;

        // Act
        let success_code: i32 = success.into();
        let error_code: i32 = error.into();
        let interrupted_code: i32 = interrupted.into();

        // Assert
        assert_eq!(success_code, 0);
        assert_eq!(error_code, 1);
        assert_eq!(interrupted_code, 130);
    }

    #[test]
    fn test_exit_status_to_exit_code() {
        // Arrange
        let success = ExitStatus::Success;

        // Act
        let exit_code: std::process::ExitCode = success.into();

        // Assert - Can't directly compare ExitCode, but we can create one
        let expected: std::process::ExitCode = std::process::ExitCode::from(0);
        // ExitCode doesn't implement PartialEq, so we just verify it compiles
        let _ = exit_code;
        let _ = expected;
    }
}
