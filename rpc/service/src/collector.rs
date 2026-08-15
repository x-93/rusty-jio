use jio_notify::events::Notification;
use jio_rpc_core::notify::RpcCollector;
use async_trait::async_trait;
use tokio::sync::mpsc::Receiver;

pub struct RpcCoreCollector {
    receiver: Receiver<Notification>,
}

impl RpcCoreCollector {
    pub fn new(receiver: Receiver<Notification>) -> Self {
        Self { receiver }
    }
}

#[async_trait]
impl RpcCollector for RpcCoreCollector {
    async fn start(&mut self) -> Result<(), String> {
        while let Some(_notification) = self.receiver.recv().await {
            // Forward notifications
        }
        Ok(())
    }
}
