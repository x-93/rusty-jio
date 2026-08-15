use std::future::Future;
use tokio::runtime::{Builder, Runtime};
use tokio::task::JoinHandle;

pub fn build_multi_threaded_runtime(threads: usize, thread_name: &str) -> std::io::Result<Runtime> {
    let name = thread_name.to_string();
    Builder::new_multi_thread()
        .worker_threads(threads)
        .thread_name_fn(move || name.clone())
        .enable_all()
        .build()
}

pub fn spawn<F>(future: F) -> JoinHandle<F::Output>
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    tokio::spawn(future)
}
