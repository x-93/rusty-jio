use crate::flowcontext::{OrphanBlocksPool, ProcessQueue, TransactionsRelayPool};
use jio_addressmanager::AddressManager;
use jio_connectionmanager::ConnectionManager;
use jio_consensusmanager::ConsensusManager;
use std::sync::Arc;

#[derive(Clone)]
pub struct FlowContext {
    pub consensus_manager: ConsensusManager,
    pub address_manager: AddressManager,
    pub connection_manager: ConnectionManager,
    pub orphans: OrphanBlocksPool,
    pub process_queue: ProcessQueue,
    pub transactions: TransactionsRelayPool,
}

impl FlowContext {
    pub fn new(
        consensus_manager: ConsensusManager,
        address_manager: AddressManager,
        connection_manager: ConnectionManager,
    ) -> Arc<Self> {
        Arc::new(Self {
            consensus_manager,
            address_manager,
            connection_manager,
            orphans: OrphanBlocksPool::new(),
            process_queue: ProcessQueue::new(),
            transactions: TransactionsRelayPool::new(),
        })
    }
}
