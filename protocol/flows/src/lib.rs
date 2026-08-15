#![allow(dead_code)]

pub mod flow_context;
pub mod flow_trait;
pub mod flowcontext;
pub mod service;
pub mod v5;
pub mod v6;

pub use flow_context::*;
pub use flow_trait::*;
pub use flowcontext::*;
pub use service::*;
pub use v5 as flows_v5;
pub use v6 as flows_v6;

#[cfg(test)]
mod tests {
    use super::*;
    use jio_addressmanager::AddressManager;
    use jio_connectionmanager::ConnectionManager;
    use jio_consensus::consensus::factory::ConsensusFactory;
    use jio_consensus_core::config::params::Params;
    use jio_consensusmanager::ConsensusManager;

    #[test]
    fn test_flow_context_initialization() {
        let params = Params::devnet();
        let consensus = ConsensusFactory::new_instance(params);
        let consensus_mgr = ConsensusManager::new(consensus);
        let addr_mgr = AddressManager::new();
        let conn_mgr = ConnectionManager::new(addr_mgr.clone(), 8);

        let ctx = FlowContext::new(consensus_mgr, addr_mgr, conn_mgr);
        assert_eq!(ctx.orphans.len(), 0);
        assert_eq!(ctx.process_queue.len(), 0);
        assert_eq!(ctx.transactions.len(), 0);
    }
}
