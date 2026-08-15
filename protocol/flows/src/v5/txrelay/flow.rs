use crate::flow_context::FlowContext;
use crate::flow_trait::Flow;
use async_trait::async_trait;
use jio_p2p::core::peer::Peer;
use jio_p2p::core::payload_type::JioPayloadType;
use std::sync::Arc;
use tokio::sync::mpsc::Receiver;

pub struct FlowTxRelay {
    ctx: Arc<FlowContext>,
    peer: Arc<Peer>,
    incoming_rx: Receiver<Vec<u8>>,
}

impl FlowTxRelay {
    pub fn new(ctx: Arc<FlowContext>, peer: Arc<Peer>) -> Self {
        let incoming_rx = peer.router.subscribe(vec![
            JioPayloadType::InvTransactions,
            JioPayloadType::Transaction,
            JioPayloadType::TransactionNotFound,
        ]);
        Self {
            ctx,
            peer,
            incoming_rx,
        }
    }
}

#[async_trait]
impl Flow for FlowTxRelay {
    async fn start(&mut self) -> Result<(), String> {
        while let Some(_msg) = self.incoming_rx.recv().await {
            // Process transaction relays
        }
        Ok(())
    }
}
