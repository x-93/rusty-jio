use std::future::Future;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ServiceError {
    #[error("Service {0} failed to start: {1}")]
    StartFailure(&'static str, String),

    #[error("Service {0} failed to stop: {1}")]
    StopFailure(&'static str, String),

    #[error("Service {0} already running")]
    AlreadyRunning(&'static str),

    #[error("Service {0} is not running")]
    NotRunning(&'static str),
}

/// The lifecycle trait implemented by all long-running node services (Consensus, P2P, RPC, Mining).
pub trait AsyncService: Send + Sync + 'static {
    /// Human-readable identifier for logging and metrics.
    fn ident(&self) -> &'static str;

    /// Initiates service operations asynchronously.
    fn start(&self) -> impl Future<Output = Result<(), ServiceError>> + Send;

    /// Initiates graceful shutdown and waits for in-flight tasks to unwind.
    fn stop(&self) -> impl Future<Output = Result<(), ServiceError>> + Send;
}
