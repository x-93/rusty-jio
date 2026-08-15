use crate::core::payload_type::JioPayloadType;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc::{channel, Receiver, Sender};

#[derive(Clone)]
pub struct Router {
    subscribers: Arc<RwLock<HashMap<JioPayloadType, Vec<Sender<Vec<u8>>>>>>,
    outgoing_tx: Sender<Vec<u8>>,
}

impl Router {
    pub fn new(outgoing_tx: Sender<Vec<u8>>) -> Self {
        Self {
            subscribers: Arc::new(RwLock::new(HashMap::new())),
            outgoing_tx,
        }
    }

    pub fn subscribe(&self, payload_types: Vec<JioPayloadType>) -> Receiver<Vec<u8>> {
        let (tx, rx) = channel(128);
        let mut subs = self.subscribers.write();
        for pt in payload_types {
            subs.entry(pt).or_default().push(tx.clone());
        }
        rx
    }

    pub async fn route_inbound(&self, payload_type: JioPayloadType, data: Vec<u8>) -> bool {
        let senders = {
            let subs = self.subscribers.read();
            subs.get(&payload_type).cloned()
        };

        if let Some(senders) = senders {
            let mut delivered = false;
            for sender in senders {
                if sender.send(data.clone()).await.is_ok() {
                    delivered = true;
                }
            }
            delivered
        } else {
            false
        }
    }

    pub async fn enqueue_outgoing(&self, data: Vec<u8>) -> Result<(), tokio::sync::mpsc::error::SendError<Vec<u8>>> {
        self.outgoing_tx.send(data).await
    }
}
