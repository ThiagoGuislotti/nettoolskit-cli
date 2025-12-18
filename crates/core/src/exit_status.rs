//! Exit status codes for command execution
//!
//! Provides standardized exit status codes used across the CLI.

use std::fmt;

/// Exit status for command execution
///
/// Status codes follow POSIX conventions:
/// - 0: Success
/// - 1: General error
/// - 130: Interrupted (128 + SIGINT(2))
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitStatus {
    /// Command executed successfully
    Success,
    /// Command execution failed
    Error,
    /// Command execution was interrupted (e.g., Ctrl+C)
    Interrupted,
}

impl fmt::Display for ExitStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Success => write!(f, "success"),
            Self::Error => write!(f, "error"),
            Self::Interrupted => write!(f, "interrupted"),
        }
    }
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
        match status {
            ExitStatus::Success => Self::SUCCESS,
            ExitStatus::Error => Self::FAILURE,
            ExitStatus::Interrupted => Self::from(130),
        }
    }
}
