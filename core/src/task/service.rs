use crate::service::AsyncService;
use crate::task::tick::Ticker;
use async_trait::async_trait;
use parking_lot::Mutex;
use std::sync::Arc;
use std::time::Duration;
use tokio::task::JoinHandle;

pub struct PeriodicTaskService {
    name: &'static str,
    interval: Duration,
    ticker: Arc<Mutex<Option<Ticker>>>,
}

impl PeriodicTaskService {
    pub fn new(name: &'static str, interval: Duration) -> Self {
        Self {
            name,
            interval,
            ticker: Arc::new(Mutex::new(None)),
        }
    }
}

#[async_trait]
impl AsyncService for PeriodicTaskService {
    fn ident(&self) -> &'static str {
        self.name
    }

    async fn start(&self) -> Result<Vec<JoinHandle<()>>, String> {
        let ticker = Ticker::new(self.interval);
        let shutdown = ticker.shutdown_trigger();
        *self.ticker.lock() = Some(ticker);

        let name = self.name;
        let handle = tokio::spawn(async move {
            log::debug!("Periodic task {name} started");
            shutdown.listener().await;
            log::debug!("Periodic task {name} terminated");
        });

        Ok(vec![handle])
    }

    async fn stop(&self) -> Result<(), String> {
        if let Some(ticker) = self.ticker.lock().take() {
            ticker.shutdown_trigger().trigger();
        }
        Ok(())
    }
}
