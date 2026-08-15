use jio_utils::triggers::SingleTrigger;
use std::time::Duration;
use tokio::time::sleep;

pub struct Ticker {
    interval: Duration,
    shutdown: SingleTrigger,
}

impl Ticker {
    pub fn new(interval: Duration) -> Self {
        Self {
            interval,
            shutdown: SingleTrigger::new(),
        }
    }

    pub fn shutdown_trigger(&self) -> SingleTrigger {
        self.shutdown.clone()
    }

    pub async fn run<F, Fut>(&self, mut callback: F)
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = ()>,
    {
        while !self.shutdown.is_triggered() {
            tokio::select! {
                _ = self.shutdown.listener() => break,
                _ = sleep(self.interval) => {
                    callback().await;
                }
            }
        }
    }
}
