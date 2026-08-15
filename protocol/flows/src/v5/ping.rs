use crate::flow_context::FlowContext;
use crate::flow_trait::Flow;
use async_trait::async_trait;
use jio_p2p::core::peer::Peer;
use jio_p2p::core::payload_type::JioPayloadType;
use jio_p2p::echo::{PingMessage, PongMessage};
use std::sync::Arc;
use tokio::sync::mpsc::Receiver;

pub struct PingFlow {
    ctx: Arc<FlowContext>,
    peer: Arc<Peer>,
    incoming_rx: Receiver<Vec<u8>>,
}

impl PingFlow {
    pub fn new(ctx: Arc<FlowContext>, peer: Arc<Peer>) -> Self {
        let incoming_rx = peer.router.subscribe(vec![JioPayloadType::Ping, JioPayloadType::Pong]);
        Self {
            ctx,
            peer,
            incoming_rx,
        }
    }
}

#[async_trait]
impl Flow for PingFlow {
    async fn start(&mut self) -> Result<(), String> {
        while let Some(msg) = self.incoming_rx.recv().await {
            if let Ok(ping) = serde_json::from_slice::<PingMessage>(&msg) {
                let pong = PongMessage { nonce: ping.nonce };
                let pong_bytes = serde_json::to_vec(&pong).unwrap_or_default();
                let _ = self.peer.router.enqueue_outgoing(pong_bytes).await;
            } else if let Ok(_pong) = serde_json::from_slice::<PongMessage>(&msg) {
                self.peer.set_last_ping(jio_core::time::unix_now());
            }
        }
        Ok(())
    }
}
