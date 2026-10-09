//! Tokio runtime orchestration, custom worker configuration, and lifecycle management.

use std::{future::Future, sync::Arc};
use tokio::runtime::{Builder, Runtime};
use tokio::task::JoinHandle;

#[derive(Clone)]
pub struct AsyncRuntime {
    inner: Arc<Runtime>,
}

impl AsyncRuntime {
    /// Create a new multi-threaded async runtime with configured worker threads and thread name prefix.
    pub fn new(worker_threads: usize, thread_name: &'static str) -> Result<Self, std::io::Error> {
        let mut builder = Builder::new_multi_thread();
        if worker_threads > 0 {
            builder.worker_threads(worker_threads);
        }
        builder.thread_name(thread_name);
        builder.enable_all();
        let runtime = builder.build()?;
        Ok(Self {
            inner: Arc::new(runtime),
        })
    }

    /// Spawn a future onto the orchestrated Tokio runtime.
    pub fn spawn<F>(&self, future: F) -> JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        self.inner.spawn(future)
    }

    /// Spawn a blocking task onto the blocking thread pool.
    pub fn spawn_blocking<F, R>(&self, func: F) -> JoinHandle<R>
    where
        F: FnOnce() -> R + Send + 'static,
        R: Send + 'static,
    {
        self.inner.spawn_blocking(func)
    }

    /// Block on a future to completion.
    pub fn block_on<F: Future>(&self, future: F) -> F::Output {
        self.inner.block_on(future)
    }

    /// Handle to the underlying Tokio runtime.
    pub fn handle(&self) -> tokio::runtime::Handle {
        self.inner.handle().clone()
    }
}
