use crate::service::{AsyncService, AsyncServiceManager};
use crate::signals::ShutdownHandler;
use log::info;
use std::sync::Arc;

pub struct Core {
    service_manager: AsyncServiceManager,
    shutdown: Arc<ShutdownHandler>,
}

impl Core {
    pub fn new() -> Self {
        let shutdown = Arc::new(ShutdownHandler::new());
        shutdown.clone().register_signal_handler();
        Self {
            service_manager: AsyncServiceManager::new(),
            shutdown,
        }
    }

    pub fn shutdown_handler(&self) -> Arc<ShutdownHandler> {
        self.shutdown.clone()
    }

    pub fn bind<S: AsyncService + 'static>(&self, service: Arc<S>) {
        self.service_manager.register(service);
    }

    pub async fn start(&self) -> Result<(), String> {
        info!("Core system starting up...");
        self.service_manager.start_all().await
    }

    pub async fn stop(&self) -> Result<(), String> {
        info!("Core system stopping...");
        self.shutdown.trigger();
        self.service_manager.stop_all().await
    }

    pub async fn run_until_interrupted(&self) -> Result<(), String> {
        self.start().await?;
        self.shutdown.wait_for_shutdown().await;
        self.stop().await?;
        self.service_manager.join_all().await;
        info!("Core system shutdown complete.");
        Ok(())
    }
}

impl Default for Core {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::service::AsyncService;
    use async_trait::async_trait;
    use parking_lot::Mutex;
    use std::time::Duration;
    use tokio::task::JoinHandle;

    struct DummyService {
        started: Arc<Mutex<bool>>,
        stopped: Arc<Mutex<bool>>,
    }

    #[async_trait]
    impl AsyncService for DummyService {
        fn ident(&self) -> &'static str {
            "dummy_service"
        }

        async fn start(&self) -> Result<Vec<JoinHandle<()>>, String> {
            *self.started.lock() = true;
            let handle = tokio::spawn(async {
                tokio::time::sleep(Duration::from_millis(50)).await;
            });
            Ok(vec![handle])
        }

        async fn stop(&self) -> Result<(), String> {
            *self.stopped.lock() = true;
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_core_lifecycle() {
        let core = Core::new();
        let started = Arc::new(Mutex::new(false));
        let stopped = Arc::new(Mutex::new(false));

        let svc = Arc::new(DummyService {
            started: started.clone(),
            stopped: stopped.clone(),
        });

        core.bind(svc);
        core.start().await.unwrap();
        assert!(*started.lock());

        core.stop().await.unwrap();
        assert!(*stopped.lock());
    }
}
