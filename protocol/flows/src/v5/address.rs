use crate::flow_context::FlowContext;
use crate::flow_trait::Flow;
use async_trait::async_trait;
use jio_p2p::core::peer::Peer;
use jio_p2p::core::payload_type::JioPayloadType;
use std::sync::Arc;
use tokio::sync::mpsc::Receiver;

#[allow(dead_code)]
pub struct AddressFlow {
    ctx: Arc<FlowContext>,
    peer: Arc<Peer>,
    incoming_rx: Receiver<Vec<u8>>,
}

impl AddressFlow {
    pub fn new(ctx: Arc<FlowContext>, peer: Arc<Peer>) -> Self {
        let incoming_rx = peer.router.subscribe(vec![
            JioPayloadType::Addresses,
            JioPayloadType::RequestAddresses,
        ]);
        Self {
            ctx,
            peer,
            incoming_rx,
        }
    }
}

#[async_trait]
impl Flow for AddressFlow {
    async fn start(&mut self) -> Result<(), String> {
        while let Some(_msg) = self.incoming_rx.recv().await {
            // Process address requests/responses
        }
        Ok(())
    }
}
