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

/// Runs multiple futures concurrently with individual timeouts
pub async fn with_timeout_concurrent<T, F>(
    timeout: Duration,
    futures: Vec<F>,
) -> Vec<Result<T, TimeoutError>>
where
    F: Future<Output = T>,
    T: Send + 'static,
{
    use futures::future::join_all;

    let timeout_futures: Vec<_> = futures
        .into_iter()
        .map(|future| with_timeout(timeout, future))
        .collect();

    join_all(timeout_futures).await
}

/// Runs futures concurrently with a global timeout for all operations
pub async fn with_global_timeout<T, F>(
    timeout: Duration,
    futures: Vec<F>,
) -> Result<Vec<T>, TimeoutError>
where
    F: Future<Output = T>,
    T: Send + 'static,
{
    use futures::future::join_all;

    let all_futures = join_all(futures);
    match time::timeout(timeout, all_futures).await {
        Ok(results) => Ok(results),
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