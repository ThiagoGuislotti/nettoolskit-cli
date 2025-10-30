use tokio::sync::broadcast;
use std::future::Future;

/// A cancellation token that can be used to cancel async operations
#[derive(Debug, Clone)]
pub struct CancellationToken {
    sender: broadcast::Sender<()>,
}

impl CancellationToken {
    /// Create a new cancellation token
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(1);
        Self { sender }
    }

    /// Cancel all operations using this token
    pub fn cancel(&self) {
        let _ = self.sender.send(());
    }

    /// Get a receiver for cancellation notifications
    pub fn receiver(&self) -> CancellationReceiver {
        CancellationReceiver {
            receiver: self.sender.subscribe(),
        }
    }

    /// Run a future that can be cancelled
    pub async fn with_cancellation<T, F>(&self, future: F) -> Result<T, CancellationError>
    where
        F: Future<Output = T>,
    {
        let mut receiver = self.receiver();
        tokio::select! {
            result = future => Ok(result),
            _ = receiver.cancelled() => Err(CancellationError),
        }
    }
}

impl Default for CancellationToken {
    fn default() -> Self {
        Self::new()
    }
}

/// Receiver for cancellation notifications
pub struct CancellationReceiver {
    receiver: broadcast::Receiver<()>,
}

impl CancellationReceiver {
    /// Wait for cancellation
    pub async fn cancelled(&mut self) {
        let _ = self.receiver.recv().await;
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CancellationError;

impl std::fmt::Display for CancellationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "operation was cancelled")
    }
}

impl std::error::Error for CancellationError {}