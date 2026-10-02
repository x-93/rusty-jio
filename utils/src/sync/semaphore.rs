//! Async semaphore wrapper based on Tokio.

use std::sync::Arc;
use tokio::sync::{Semaphore, SemaphorePermit};

#[derive(Clone)]
pub struct AsyncSemaphore(Arc<Semaphore>);

impl AsyncSemaphore {
    pub fn new(permits: usize) -> Self {
        Self(Arc::new(Semaphore::new(permits)))
    }

    pub async fn acquire(&self) -> SemaphorePermit<'_> {
        self.0.acquire().await.expect("Semaphore is closed")
    }

    pub fn available_permits(&self) -> usize {
        self.0.available_permits()
    }
}
