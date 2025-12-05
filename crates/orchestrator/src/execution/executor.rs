/// Async command execution system
///
/// Provides non-blocking command execution with progress tracking,
/// cancellation support, and concurrent command handling.
use std::future::Future;
use std::pin::Pin;
use tokio::sync::{mpsc, oneshot};
use tokio::task::JoinHandle;

/// Result of command execution
pub type CommandResult = Result<String, Box<dyn std::error::Error + Send + Sync>>;

/// Future that resolves to a command result
pub type CommandFuture = Pin<Box<dyn Future<Output = CommandResult> + Send>>;

/// Handle to a running asynchronous command.
///
/// Provides control over command execution, including:
/// - Waiting for completion
/// - Polling for results
/// - Requesting cancellation (if supported)
///
/// # Examples
///
/// ```rust,no_run
/// use nettoolskit_orchestrator::CommandHandle;
///
/// async fn example(mut handle: CommandHandle) {
///     // Wait for completion
///     match handle.wait().await {
///         Ok(result) => println!("Result: {:?}", result),
///         Err(e) => eprintln!("Command failed: {}", e),
///     }
/// }
/// ```
pub struct CommandHandle {
    receiver: oneshot::Receiver<CommandResult>,
    cancel_tx: Option<mpsc::Sender<()>>,
}

impl CommandHandle {
    /// Create a new command handle
    pub fn new(receiver: oneshot::Receiver<CommandResult>) -> Self {
        Self {
            receiver,
            cancel_tx: None,
        }
    }

    /// Create a cancellable command handle
    pub fn cancellable(
        receiver: oneshot::Receiver<CommandResult>,
        cancel_tx: mpsc::Sender<()>,
    ) -> Self {
        Self {
            receiver,
            cancel_tx: Some(cancel_tx),
        }
    }

    /// Wait for command to complete
    pub async fn wait(self) -> Result<CommandResult, oneshot::error::RecvError> {
        self.receiver.await
    }

    /// Try to get result if ready
    pub fn try_result(&mut self) -> Option<CommandResult> {
        self.receiver.try_recv().ok()
    }

    /// Request cancellation of the command
    pub async fn cancel(&mut self) -> bool {
        if let Some(tx) = &self.cancel_tx {
            tx.send(()).await.is_ok()
        } else {
            false
        }
    }
}

/// Progress update for a running command.
///
/// Provides real-time feedback about command execution progress,
/// including status messages, completion percentages, and step tracking.
///
/// # Examples
///
/// ```rust
/// use nettoolskit_orchestrator::CommandProgress;
///
/// let progress = CommandProgress::message("Processing files...");
/// let progress_with_percent = CommandProgress::percent("Downloading", 75);
/// let progress_with_steps = CommandProgress::steps("Uploading", 2, 5);
/// ```
#[derive(Debug, Clone)]
pub struct CommandProgress {
    /// Current step description
    pub message: String,
    /// Progress percentage (0-100)
    pub percent: Option<u8>,
    /// Total steps
    pub total: Option<usize>,
    /// Completed steps
    pub completed: Option<usize>,
}

impl CommandProgress {
    /// Create a simple progress message
    pub fn message(msg: impl Into<String>) -> Self {
        Self {
            message: msg.into(),
            percent: None,
            total: None,
            completed: None,
        }
    }

    /// Create progress with percentage
    pub fn percent(msg: impl Into<String>, percent: u8) -> Self {
        Self {
            message: msg.into(),
            percent: Some(percent.min(100)),
            total: None,
            completed: None,
        }
    }

    /// Create progress with steps
    pub fn steps(msg: impl Into<String>, completed: usize, total: usize) -> Self {
        let percent = if total > 0 {
            Some(((completed * 100) / total).min(100) as u8)
        } else {
            None
        };

        Self {
            message: msg.into(),
            percent,
            total: Some(total),
            completed: Some(completed),
        }
    }
}

/// Channel for sending progress updates
pub type ProgressSender = mpsc::UnboundedSender<CommandProgress>;

/// Async command executor
pub struct AsyncCommandExecutor {
    /// Maximum concurrent commands
    max_concurrent: usize,
    /// Currently running commands
    running: Vec<JoinHandle<()>>,
}

impl AsyncCommandExecutor {
    /// Create a new executor with default concurrency limit
    pub fn new() -> Self {
        Self::with_limit(10)
    }

    /// Create executor with specific concurrency limit
    pub fn with_limit(max_concurrent: usize) -> Self {
        Self {
            max_concurrent,
            running: Vec::new(),
        }
    }

    /// Spawn a command for execution
    pub fn spawn<F>(&mut self, future: F) -> CommandHandle
    where
        F: Future<Output = CommandResult> + Send + 'static,
    {
        let (tx, rx) = oneshot::channel();

        let handle = tokio::spawn(async move {
            let result = future.await;
            let _ = tx.send(result);
        });

        self.running.push(handle);
        self.cleanup_finished();

        CommandHandle::new(rx)
    }

    /// Spawn a cancellable command
    pub fn spawn_cancellable<F>(&mut self, future: F) -> CommandHandle
    where
        F: Future<Output = CommandResult> + Send + 'static,
    {
        let (result_tx, result_rx) = oneshot::channel();
        let (cancel_tx, mut cancel_rx) = mpsc::channel(1);

        let handle = tokio::spawn(async move {
            tokio::pin!(future);

            tokio::select! {
                result = &mut future => {
                    let _ = result_tx.send(result);
                }
                _ = cancel_rx.recv() => {
                    let _ = result_tx.send(Err("Command was cancelled".into()));
                }
            }
        });

        self.running.push(handle);
        self.cleanup_finished();

        CommandHandle::cancellable(result_rx, cancel_tx)
    }

    /// Spawn a command with progress reporting
    pub fn spawn_with_progress<F, Fut>(
        &mut self,
        factory: F,
    ) -> (CommandHandle, mpsc::UnboundedReceiver<CommandProgress>)
    where
        F: FnOnce(ProgressSender) -> Fut + Send + 'static,
        Fut: Future<Output = CommandResult> + Send + 'static,
    {
        let (result_tx, result_rx) = oneshot::channel();
        let (progress_tx, progress_rx) = mpsc::unbounded_channel();

        let progress_tx_clone = progress_tx.clone();
        let handle = tokio::spawn(async move {
            let future = factory(progress_tx_clone);
            let result = future.await;
            let _ = result_tx.send(result);
        });

        self.running.push(handle);
        self.cleanup_finished();

        (CommandHandle::new(result_rx), progress_rx)
    }

    /// Check if executor is at capacity
    pub fn is_full(&self) -> bool {
        self.running.len() >= self.max_concurrent
    }

    /// Get number of running commands
    pub fn running_count(&self) -> usize {
        self.running.len()
    }

    /// Wait for all commands to complete
    pub async fn wait_all(&mut self) {
        for handle in self.running.drain(..) {
            let _ = handle.await;
        }
    }

    /// Cancel all running commands
    pub async fn cancel_all(&mut self) {
        for handle in self.running.drain(..) {
            handle.abort();
        }
    }

    /// Remove finished tasks from the running list
    fn cleanup_finished(&mut self) {
        self.running.retain(|handle| !handle.is_finished());
    }
}

impl Default for AsyncCommandExecutor {
    fn default() -> Self {
        Self::new()
    }
}


