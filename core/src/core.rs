//! Central supervisor and service orchestrator for the Jio node.

use std::sync::Arc;
use parking_lot::Mutex;
use tokio_util::sync::CancellationToken;
use super::service::{AsyncService, ServiceError};

#[derive(Clone)]
pub struct Core {
    services: Arc<Mutex<Vec<Arc<dyn AsyncService>>>>,
    shutdown_token: CancellationToken,
}

impl Default for Core {
    fn default() -> Self {
        Self::new()
    }
}

impl Core {
    pub fn new() -> Self {
        Self {
            services: Arc::new(Mutex::new(Vec::new())),
            shutdown_token: CancellationToken::new(),
        }
    }

    /// Registers an asynchronous service with the supervisor.
    pub fn bind<S: AsyncService>(&self, service: Arc<S>) {
        self.services.lock().push(service);
    }

    /// Starts all registered services in order of registration.
    pub async fn start(&self) -> Result<(), ServiceError> {
        let services = self.services.lock().clone();
        for service in services {
            service.start().await?;
        }
        Ok(())
    }

    /// Stops all registered services in reverse order of registration.
    pub fn stop(&self) -> Result<(), ServiceError> {
        self.shutdown_token.cancel();
        let services = self.services.lock().clone();
        for service in services.into_iter().rev() {
            service.stop()?;
        }
        Ok(())
    }

    /// Returns the shutdown cancellation token.
    pub fn shutdown_token(&self) -> CancellationToken {
        self.shutdown_token.clone()
    }

    /// Awaits until shutdown cancellation is triggered.
    pub async fn join(&self) {
        self.shutdown_token.cancelled().await;
    }
}
