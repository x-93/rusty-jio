//! Service lifecycle orchestration and graceful cancellation signals.

use std::{future::Future, pin::Pin, time::Duration};
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

/// Graceful shutdown signal abstraction based on Tokio cancellation tokens.
#[derive(Clone, Default)]
pub struct ShutdownSignal {
    token: CancellationToken,
}

impl ShutdownSignal {
    pub fn new() -> Self {
        Self {
            token: CancellationToken::new(),
        }
    }

    /// Trigger the shutdown signal across all listeners.
    pub fn trigger(&self) {
        self.token.cancel();
    }

    /// Check if shutdown has been signaled.
    pub fn is_triggered(&self) -> bool {
        self.token.is_cancelled()
    }

    /// Await the shutdown cancellation signal.
    pub async fn wait(&self) {
        self.token.cancelled().await;
    }

    /// Obtain a child cancellation token.
    pub fn child_token(&self) -> CancellationToken {
        self.token.child_token()
    }
}

/// Trait for async background services with graceful termination.
pub trait AsyncService: Send + 'static {
    fn name(&self) -> &'static str;
    fn start(
        self: Box<Self>,
        shutdown: ShutdownSignal,
    ) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>> + Send>>;
}

/// Service supervisor that tracks running tasks and orchestrates graceful cancellation.
#[derive(Default)]
pub struct ServiceSupervisor {
    shutdown: ShutdownSignal,
    handles: Vec<(&'static str, JoinHandle<()>)>,
}

impl ServiceSupervisor {
    pub fn new() -> Self {
        Self {
            shutdown: ShutdownSignal::new(),
            handles: Vec::new(),
        }
    }

    pub fn shutdown_signal(&self) -> ShutdownSignal {
        self.shutdown.clone()
    }

    pub fn spawn<F>(&mut self, name: &'static str, future: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        let handle = tokio::spawn(future);
        self.handles.push((name, handle));
    }

    /// Signal cancellation and wait for all tasks to stop gracefully up to timeout.
    pub async fn stop_all(&mut self, timeout: Duration) {
        self.shutdown.trigger();

        for (name, handle) in self.handles.drain(..) {
            match tokio::time::timeout(timeout, handle).await {
                Ok(Ok(())) => {
                    log::debug!("Service {name} terminated gracefully");
                }
                Ok(Err(e)) => {
                    log::warn!("Service {name} panicked or failed: {e:?}");
                }
                Err(_) => {
                    log::warn!("Service {name} shutdown timed out after {timeout:?}");
                }
            }
        }
    }
}
