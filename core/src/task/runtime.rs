//! Tokio multi-threaded async runtime builder.

use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::runtime::{Builder, Runtime};

static THREAD_COUNTER: AtomicUsize = AtomicUsize::new(1);

pub fn create_tokio_runtime(worker_threads: Option<usize>) -> std::io::Result<Runtime> {
    let mut builder = Builder::new_multi_thread();
    if let Some(workers) = worker_threads {
        builder.worker_threads(workers);
    }
    builder
        .thread_name_fn(|| {
            let id = THREAD_COUNTER.fetch_add(1, Ordering::SeqCst);
            format!("jio-worker-{}", id)
        })
        .enable_all()
        .build()
}
