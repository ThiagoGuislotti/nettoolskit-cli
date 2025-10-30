use std::future::Future;
use std::time::Duration;
use tokio::time;

/// Runs a future with a timeout
pub async fn with_timeout<T, F>(
    timeout: Duration,
    future: F,
) -> Result<T, TimeoutError>
where
    F: Future<Output = T>,
{
    match time::timeout(timeout, future).await {
        Ok(result) => Ok(result),
        Err(_) => Err(TimeoutError),
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TimeoutError;

impl std::fmt::Display for TimeoutError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "operation timed out")
    }
}

impl std::error::Error for TimeoutError {}