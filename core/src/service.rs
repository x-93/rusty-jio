use async_trait::async_trait;
use log::{error, info};
use parking_lot::Mutex;
use std::sync::Arc;
use tokio::task::JoinHandle;

#[async_trait]
pub trait AsyncService: Send + Sync {
    fn ident(&self) -> &'static str;
    async fn start(&self) -> Result<Vec<JoinHandle<()>>, String>;
    async fn stop(&self) -> Result<(), String>;
}

pub trait Service: Send + Sync {
    fn ident(&self) -> &'static str;
    fn start(&self) -> Result<(), String>;
    fn stop(&self) -> Result<(), String>;
}

#[derive(Default, Clone)]
pub struct AsyncServiceManager {
    services: Arc<Mutex<Vec<Arc<dyn AsyncService>>>>,
    handles: Arc<Mutex<Vec<JoinHandle<()>>>>,
}

impl AsyncServiceManager {
    pub fn new() -> Self {
        Self {
            services: Arc::new(Mutex::new(Vec::new())),
            handles: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn register<S: AsyncService + 'static>(&self, service: Arc<S>) {
        self.services.lock().push(service);
    }

    pub async fn start_all(&self) -> Result<(), String> {
        let services = self.services.lock().clone();
        for svc in services {
            info!("Starting async service: {}", svc.ident());
            match svc.start().await {
                Ok(mut h) => {
                    self.handles.lock().append(&mut h);
                }
                Err(e) => {
                    error!("Failed to start service {}: {}", svc.ident(), e);
                    return Err(e);
                }
            }
        }
        Ok(())
    }

    pub async fn stop_all(&self) -> Result<(), String> {
        let services = self.services.lock().clone();
        for svc in services.iter().rev() {
            info!("Stopping async service: {}", svc.ident());
            if let Err(e) = svc.stop().await {
                error!("Error stopping service {}: {}", svc.ident(), e);
            }
        }
        Ok(())
    }

    pub async fn join_all(&self) {
        let mut handles = std::mem::take(&mut *self.handles.lock());
        for handle in handles.drain(..) {
            let _ = handle.await;
        }
    }
}
