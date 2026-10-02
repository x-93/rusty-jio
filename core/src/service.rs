//! AsyncService lifecycle trait.

use async_trait::async_trait;
use std::sync::Arc;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("Service '{0}' failed to start: {1}")]
    StartupFailed(&'static str, String),
    #[error("Service '{0}' failed to stop: {1}")]
    ShutdownFailed(&'static str, String),
}

#[async_trait]
pub trait AsyncService: Send + Sync + 'static {
    fn ident(&self) -> &'static str;
    async fn start(self: Arc<Self>) -> Result<(), ServiceError>;
    fn stop(&self) -> Result<(), ServiceError>;
}
