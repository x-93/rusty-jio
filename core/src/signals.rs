use jio_utils::triggers::SingleTrigger;
use std::sync::Arc;
use tokio::signal;

#[derive(Clone, Debug)]
pub struct ShutdownHandler {
    trigger: SingleTrigger,
}

impl ShutdownHandler {
    pub fn new() -> Self {
        Self {
            trigger: SingleTrigger::new(),
        }
    }

    pub fn trigger(&self) {
        self.trigger.trigger();
    }

    pub fn is_shutdown(&self) -> bool {
        self.trigger.is_triggered()
    }

    pub async fn wait_for_shutdown(&self) {
        self.trigger.listener().await;
    }

    pub fn register_signal_handler(self: Arc<Self>) {
        tokio::spawn(async move {
            if let Ok(()) = signal::ctrl_c().await {
                log::info!("Received Ctrl+C / SIGINT signal, initiating graceful shutdown...");
                self.trigger();
            }
        });
    }
}

impl Default for ShutdownHandler {
    fn default() -> Self {
        Self::new()
    }
}
