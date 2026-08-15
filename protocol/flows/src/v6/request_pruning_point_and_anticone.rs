use crate::flow_context::FlowContext;
use crate::flow_trait::Flow;
use async_trait::async_trait;
use jio_p2p::core::peer::Peer;
use jio_p2p::core::payload_type::JioPayloadType;
use std::sync::Arc;
use tokio::sync::mpsc::Receiver;

pub struct RequestPruningPointAndAnticoneV6Flow {
    ctx: Arc<FlowContext>,
    peer: Arc<Peer>,
    incoming_rx: Receiver<Vec<u8>>,
}

impl RequestPruningPointAndAnticoneV6Flow {
    pub fn new(ctx: Arc<FlowContext>, peer: Arc<Peer>) -> Self {
        let incoming_rx = peer.router.subscribe(vec![JioPayloadType::RequestPruningPointAndAnticone]);
        Self {
            ctx,
            peer,
            incoming_rx,
        }
    }
}

#[async_trait]
impl Flow for RequestPruningPointAndAnticoneV6Flow {
    async fn start(&mut self) -> Result<(), String> {
        while let Some(_msg) = self.incoming_rx.recv().await {
            // Serve v6 pruning point and anticone
        }
        Ok(())
    }
}
