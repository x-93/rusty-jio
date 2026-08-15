use crate::flow_context::FlowContext;
use jio_p2p::core::peer::Peer;
use std::sync::Arc;

pub struct HandleRequests {
    pub ctx: Arc<FlowContext>,
    pub peer: Arc<Peer>,
}

impl HandleRequests {
    pub fn new(ctx: Arc<FlowContext>, peer: Arc<Peer>) -> Self {
        Self { ctx, peer }
    }
}
