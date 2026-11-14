use tokio::sync::broadcast;

/// Cross-platform OS signal handler for graceful shutdown.
pub struct ShutdownHandler {
    sender: broadcast::Sender<()>,
}

impl ShutdownHandler {
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(1);
        Self { sender }
    }

    /// Subscribes to the shutdown notification.
    pub fn subscribe(&self) -> broadcast::Receiver<()> {
        self.sender.subscribe()
    }

    /// Triggers manual shutdown across all subscribers.
    pub fn trigger(&self) {
        let _ = self.sender.send(());
    }

    /// Spawns a background task that listens for OS shutdown signals (CTRL+C).
    pub fn listen(&self) -> tokio::task::JoinHandle<()> {
        let sender = self.sender.clone();
        tokio::spawn(async move {
            if let Ok(()) = tokio::signal::ctrl_c().await {
                log::info!(
                    "Received shutdown signal (CTRL+C), initiating graceful node termination..."
                );
                let _ = sender.send(());
            }
        })
    }
}

impl Default for ShutdownHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_shutdown_trigger() {
        let handler = ShutdownHandler::new();
        let mut sub1 = handler.subscribe();
        let mut sub2 = handler.subscribe();

        handler.trigger();
        assert!(sub1.recv().await.is_ok());
        assert!(sub2.recv().await.is_ok());
    }
}
