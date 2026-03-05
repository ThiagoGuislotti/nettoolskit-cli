//! Shared background worker runtime for service-mode task execution.
//!
//! This crate provides a reusable queue + dispatcher + retry runtime that can
//! be integrated by orchestrator layers through callback hooks.

use std::fmt;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::sync::mpsc::error::TrySendError;
use tokio::sync::Semaphore;

/// Default queue capacity for worker admission.
pub const DEFAULT_TASK_QUEUE_CAPACITY: usize = 64;
/// Default concurrency limit for worker execution.
pub const DEFAULT_TASK_MAX_CONCURRENCY: usize = 2;
/// Default retry count (additional attempts after first attempt).
pub const DEFAULT_TASK_MAX_RETRIES: usize = 2;
/// Default base retry backoff in milliseconds.
pub const DEFAULT_TASK_RETRY_BASE_DELAY_MS: u64 = 300;
/// Default max retry backoff in milliseconds.
pub const DEFAULT_TASK_RETRY_MAX_DELAY_MS: u64 = 1_500;

/// Task worker execution policy.
#[derive(Debug, Clone, Copy)]
pub struct TaskWorkerPolicy {
    /// Max queued tasks before submit is rejected.
    pub queue_capacity: usize,
    /// Number of tasks that may run concurrently.
    pub max_concurrency: usize,
    /// Number of retries after first failed attempt.
    pub max_retries: usize,
    /// Exponential backoff base delay.
    pub retry_base_delay: Duration,
    /// Exponential backoff max delay.
    pub retry_max_delay: Duration,
}

impl Default for TaskWorkerPolicy {
    fn default() -> Self {
        Self {
            queue_capacity: DEFAULT_TASK_QUEUE_CAPACITY,
            max_concurrency: DEFAULT_TASK_MAX_CONCURRENCY,
            max_retries: DEFAULT_TASK_MAX_RETRIES,
            retry_base_delay: Duration::from_millis(DEFAULT_TASK_RETRY_BASE_DELAY_MS),
            retry_max_delay: Duration::from_millis(DEFAULT_TASK_RETRY_MAX_DELAY_MS),
        }
    }
}

/// Worker task execution result status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskWorkerResultStatus {
    /// Task completed successfully.
    Succeeded,
    /// Task finished with failure.
    Failed,
    /// Task was cancelled.
    Cancelled,
}

/// Worker task execution result payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskWorkerResult {
    /// Outcome status.
    pub status: TaskWorkerResultStatus,
    /// Human-readable detail.
    pub detail: String,
}

impl TaskWorkerResult {
    /// Build a task worker result.
    #[must_use]
    pub fn new(status: TaskWorkerResultStatus, detail: impl Into<String>) -> Self {
        Self {
            status,
            detail: detail.into(),
        }
    }

    /// Build successful result.
    #[must_use]
    pub fn succeeded(detail: impl Into<String>) -> Self {
        Self::new(TaskWorkerResultStatus::Succeeded, detail)
    }

    /// Build failure result.
    #[must_use]
    pub fn failed(detail: impl Into<String>) -> Self {
        Self::new(TaskWorkerResultStatus::Failed, detail)
    }

    /// Build cancelled result.
    #[must_use]
    pub fn cancelled(detail: impl Into<String>) -> Self {
        Self::new(TaskWorkerResultStatus::Cancelled, detail)
    }
}

/// Boxed future type used by worker callbacks.
pub type TaskWorkerFuture = Pin<Box<dyn Future<Output = TaskWorkerResult> + Send + 'static>>;

/// Callback contract used by [`TaskWorkerRuntime`] to delegate domain behavior.
pub trait TaskWorkerCallbacks<T>: Send + Sync + 'static {
    /// Returns true when the task should be cancelled.
    fn is_cancelled(&self, task: &T) -> bool;
    /// Called before each attempt starts.
    fn on_attempt_start(&self, task: &T, attempt: usize, max_attempts: usize);
    /// Called when task is cancelled before starting an attempt.
    fn on_cancelled_before_start(&self, task: &T);
    /// Called when task is cancelled after one attempt execution.
    fn on_cancelled_after_attempt(&self, task: &T);
    /// Called when a retry is scheduled.
    fn on_retry_scheduled(
        &self,
        task: &T,
        attempt: usize,
        max_attempts: usize,
        delay: Duration,
        detail: &str,
    );
    /// Called when task reaches terminal state.
    fn on_finished(&self, task: &T, result: &TaskWorkerResult, attempt: usize, max_attempts: usize);
    /// Executes one task attempt.
    fn execute(&self, task: &T) -> TaskWorkerFuture;
}

/// Submit error for worker queue admission.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskWorkerSubmitError {
    /// Queue capacity was reached.
    QueueFull,
    /// Queue is closed/unavailable.
    QueueClosed,
}

impl fmt::Display for TaskWorkerSubmitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::QueueFull => write!(f, "task queue capacity reached"),
            Self::QueueClosed => write!(f, "task queue unavailable"),
        }
    }
}

impl std::error::Error for TaskWorkerSubmitError {}

/// Retry backoff calculator with exponential growth and max bound.
#[must_use]
pub fn task_worker_retry_delay(policy: TaskWorkerPolicy, attempt: usize) -> Duration {
    let exponent = attempt.saturating_sub(1) as u32;
    let factor = 2_u128.saturating_pow(exponent);
    let base_ms = policy.retry_base_delay.as_millis();
    let max_ms = policy.retry_max_delay.as_millis();
    let delay_ms = (base_ms.saturating_mul(factor)).min(max_ms) as u64;
    Duration::from_millis(delay_ms)
}

/// Reusable background worker runtime.
pub struct TaskWorkerRuntime<T> {
    sender: mpsc::Sender<T>,
}

impl<T> Clone for TaskWorkerRuntime<T> {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
        }
    }
}

impl<T> TaskWorkerRuntime<T>
where
    T: Clone + Send + Sync + 'static,
{
    /// Start a worker runtime with provided policy and callbacks.
    #[must_use]
    pub fn start<C>(policy: TaskWorkerPolicy, callbacks: Arc<C>) -> Self
    where
        C: TaskWorkerCallbacks<T>,
    {
        let (sender, mut receiver) = mpsc::channel(policy.queue_capacity.max(1));

        tokio::spawn(async move {
            let semaphore = Arc::new(Semaphore::new(policy.max_concurrency.max(1)));

            while let Some(task) = receiver.recv().await {
                let permit = match semaphore.clone().acquire_owned().await {
                    Ok(permit) => permit,
                    Err(_) => break,
                };
                let callbacks = Arc::clone(&callbacks);
                tokio::spawn(async move {
                    let _permit = permit;
                    run_task(task, policy, callbacks).await;
                });
            }
        });

        Self { sender }
    }

    /// Submit one task for queued execution.
    ///
    /// # Errors
    ///
    /// Returns [`TaskWorkerSubmitError::QueueFull`] when queue is full and
    /// [`TaskWorkerSubmitError::QueueClosed`] when queue is unavailable.
    pub fn submit(&self, task: T) -> Result<(), TaskWorkerSubmitError> {
        match self.sender.try_send(task) {
            Ok(()) => Ok(()),
            Err(TrySendError::Full(_)) => Err(TaskWorkerSubmitError::QueueFull),
            Err(TrySendError::Closed(_)) => Err(TaskWorkerSubmitError::QueueClosed),
        }
    }
}

async fn run_task<T, C>(task: T, policy: TaskWorkerPolicy, callbacks: Arc<C>)
where
    T: Clone + Send + Sync + 'static,
    C: TaskWorkerCallbacks<T>,
{
    let max_attempts = policy.max_retries.saturating_add(1);

    for attempt in 1..=max_attempts {
        if callbacks.is_cancelled(&task) {
            callbacks.on_cancelled_before_start(&task);
            return;
        }

        callbacks.on_attempt_start(&task, attempt, max_attempts);
        let result = callbacks.execute(&task).await;

        if callbacks.is_cancelled(&task) {
            callbacks.on_cancelled_after_attempt(&task);
            return;
        }

        if result.status == TaskWorkerResultStatus::Failed && attempt < max_attempts {
            let delay = task_worker_retry_delay(policy, attempt);
            callbacks.on_retry_scheduled(&task, attempt, max_attempts, delay, &result.detail);
            tokio::time::sleep(delay).await;
            continue;
        }

        callbacks.on_finished(&task, &result, attempt, max_attempts);
        return;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{HashMap, HashSet};
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    #[derive(Debug, Clone)]
    struct MockTask {
        id: String,
        fail_once: bool,
    }

    #[derive(Default)]
    struct MockCallbacks {
        attempts: Arc<Mutex<HashMap<String, usize>>>,
        events: Arc<Mutex<Vec<String>>>,
        cancelled_ids: Arc<Mutex<HashSet<String>>>,
    }

    impl MockCallbacks {
        fn mark_cancelled(&self, task_id: &str) {
            self.cancelled_ids
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .insert(task_id.to_string());
        }

        fn events_snapshot(&self) -> Vec<String> {
            self.events
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .clone()
        }
    }

    impl TaskWorkerCallbacks<MockTask> for MockCallbacks {
        fn is_cancelled(&self, task: &MockTask) -> bool {
            self.cancelled_ids
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .contains(&task.id)
        }

        fn on_attempt_start(&self, task: &MockTask, attempt: usize, max_attempts: usize) {
            self.events
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .push(format!("attempt:{}:{attempt}/{max_attempts}", task.id));
        }

        fn on_cancelled_before_start(&self, task: &MockTask) {
            self.events
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .push(format!("cancelled-before:{}", task.id));
        }

        fn on_cancelled_after_attempt(&self, task: &MockTask) {
            self.events
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .push(format!("cancelled-after:{}", task.id));
        }

        fn on_retry_scheduled(
            &self,
            task: &MockTask,
            attempt: usize,
            max_attempts: usize,
            delay: Duration,
            _detail: &str,
        ) {
            self.events
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .push(format!(
                    "retry:{}:{attempt}/{max_attempts}:{}",
                    task.id,
                    delay.as_millis()
                ));
        }

        fn on_finished(
            &self,
            task: &MockTask,
            result: &TaskWorkerResult,
            attempt: usize,
            max_attempts: usize,
        ) {
            self.events
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .push(format!(
                    "finished:{}:{:?}:{attempt}/{max_attempts}",
                    task.id, result.status
                ));
        }

        fn execute(&self, task: &MockTask) -> TaskWorkerFuture {
            let attempts = Arc::clone(&self.attempts);
            let task_id = task.id.clone();
            let fail_once = task.fail_once;

            Box::pin(async move {
                let attempt = {
                    let mut guard = attempts
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner());
                    let counter = guard.entry(task_id).or_insert(0);
                    *counter += 1;
                    *counter
                };

                if fail_once && attempt == 1 {
                    TaskWorkerResult::failed("first attempt failed")
                } else {
                    TaskWorkerResult::succeeded("done")
                }
            })
        }
    }

    async fn wait_until(predicate: impl Fn() -> bool) {
        for _ in 0..50 {
            if predicate() {
                return;
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    }

    #[test]
    fn retry_delay_is_exponential_and_bounded() {
        let policy = TaskWorkerPolicy {
            queue_capacity: 8,
            max_concurrency: 2,
            max_retries: 2,
            retry_base_delay: Duration::from_millis(40),
            retry_max_delay: Duration::from_millis(90),
        };
        assert_eq!(
            task_worker_retry_delay(policy, 1),
            Duration::from_millis(40)
        );
        assert_eq!(
            task_worker_retry_delay(policy, 2),
            Duration::from_millis(80)
        );
        assert_eq!(
            task_worker_retry_delay(policy, 3),
            Duration::from_millis(90)
        );
    }

    #[tokio::test]
    async fn runtime_retries_failed_task_then_succeeds() {
        let callbacks = Arc::new(MockCallbacks::default());
        let policy = TaskWorkerPolicy {
            max_retries: 1,
            retry_base_delay: Duration::from_millis(1),
            retry_max_delay: Duration::from_millis(1),
            ..TaskWorkerPolicy::default()
        };
        let runtime = TaskWorkerRuntime::start(policy, Arc::clone(&callbacks));
        runtime
            .submit(MockTask {
                id: "task-1".to_string(),
                fail_once: true,
            })
            .expect("submit should succeed");

        wait_until(|| {
            callbacks
                .events_snapshot()
                .iter()
                .any(|event| event == "finished:task-1:Succeeded:2/2")
        })
        .await;

        let events = callbacks.events_snapshot();
        assert!(events.iter().any(|event| event == "attempt:task-1:1/2"));
        assert!(events
            .iter()
            .any(|event| event.starts_with("retry:task-1:1/2")));
        assert!(events
            .iter()
            .any(|event| event == "finished:task-1:Succeeded:2/2"));
    }

    #[tokio::test]
    async fn runtime_cancels_task_before_first_attempt() {
        let callbacks = Arc::new(MockCallbacks::default());
        callbacks.mark_cancelled("task-cancel");
        let runtime = TaskWorkerRuntime::start(TaskWorkerPolicy::default(), Arc::clone(&callbacks));
        runtime
            .submit(MockTask {
                id: "task-cancel".to_string(),
                fail_once: false,
            })
            .expect("submit should succeed");

        wait_until(|| {
            callbacks
                .events_snapshot()
                .iter()
                .any(|event| event == "cancelled-before:task-cancel")
        })
        .await;

        let events = callbacks.events_snapshot();
        assert!(events
            .iter()
            .any(|event| event == "cancelled-before:task-cancel"));
        assert!(!events
            .iter()
            .any(|event| event.starts_with("attempt:task-cancel")));
    }
}
