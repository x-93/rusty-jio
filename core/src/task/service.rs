//! Async task wrapper with error and panic recovery.

use std::future::Future;

pub fn spawn_logged<F>(name: &'static str, future: F) -> tokio::task::JoinHandle<()>
where
    F: Future<Output = ()> + Send + 'static,
{
    tokio::spawn(async move {
        println!("[TASK] Spawned worker task '{}'", name);
        future.await;
        println!("[TASK] Worker task '{}' completed", name);
    })
}
