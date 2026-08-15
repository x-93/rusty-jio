use std::future::Future;
use std::pin::Pin;

pub trait DaemonService: Send + Sync {
    fn start(&mut self) -> Pin<Box<dyn Future<Output = Result<(), String>> + Send>>;
    fn stop(&mut self) -> Pin<Box<dyn Future<Output = Result<(), String>> + Send>>;
}
