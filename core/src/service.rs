use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("Service error in '{0}': {1}")]
    ServiceError(&'static str, String),

    #[error("Service join failure in '{0}': {1}")]
    JoinError(&'static str, String),

    #[error("Core daemon is already running")]
    AlreadyRunning,

    #[error("Core daemon is not running")]
    NotRunning,

    #[error("Unknown error: {0}")]
    Other(String),
}

pub type AsyncServiceFuture = Pin<Box<dyn Future<Output = Result<(), CoreError>> + Send>>;

/// Every background service in the node (such as P2P, Consensus, RPC, Mining)
/// implements this lifecycle trait.
pub trait AsyncService: Send + Sync {
    /// Human-readable identifier for the service (e.g. "p2p-network", "consensus-engine").
    fn ident(self: Arc<Self>) -> &'static str;

    /// Launch the service's asynchronous execution loop.
    fn start(self: Arc<Self>) -> AsyncServiceFuture;

    /// Notify the service that it should initiate shutdown.
    fn signal_exit(self: Arc<Self>);

    /// Await graceful completion and perform final cleanup.
    fn stop(self: Arc<Self>) -> AsyncServiceFuture;
}
