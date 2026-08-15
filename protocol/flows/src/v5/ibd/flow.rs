use crate::flow_context::FlowContext;
use crate::flow_trait::Flow;
use async_trait::async_trait;
use jio_p2p::core::peer::Peer;
use jio_p2p::core::payload_type::JioPayloadType;
use std::sync::Arc;
use tokio::sync::mpsc::Receiver;

pub struct IbdFlow {
    ctx: Arc<FlowContext>,
    peer: Arc<Peer>,
    incoming_rx: Receiver<Vec<u8>>,
}

impl IbdFlow {
    pub fn new(ctx: Arc<FlowContext>, peer: Arc<Peer>) -> Self {
        let incoming_rx = peer.router.subscribe(vec![
            JioPayloadType::BlockHeader,
            JioPayloadType::DoneHeaders,
            JioPayloadType::IbdBlock,
            JioPayloadType::DoneBlocks,
        ]);
        Self {
            ctx,
            peer,
            incoming_rx,
        }
    }
}

#[async_trait]
impl Flow for IbdFlow {
    async fn start(&mut self) -> Result<(), String> {
        while let Some(_msg) = self.incoming_rx.recv().await {
            // Process IBD blocks and headers
        }
        Ok(())
    }
}
