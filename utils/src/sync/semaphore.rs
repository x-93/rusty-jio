use std::sync::Arc;
use tokio::sync::{Semaphore as TokioSemaphore, SemaphorePermit as TokioSemaphorePermit};

#[derive(Clone, Debug)]
pub struct Semaphore {
    inner: Arc<TokioSemaphore>,
}

impl Semaphore {
    pub fn new(permits: usize) -> Self {
        Self {
            inner: Arc::new(TokioSemaphore::new(permits)),
        }
    }

    pub async fn acquire(&self) -> TokioSemaphorePermit<'_> {
        self.inner
            .acquire()
            .await
            .expect("semaphore is never closed in this wrapper")
    }

    pub fn available_permits(&self) -> usize {
        self.inner.available_permits()
    }
}
