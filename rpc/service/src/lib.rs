pub mod collector;
pub mod converter;
pub mod service;

pub use collector::*;
pub use converter::*;
pub use service::*;

#[cfg(test)]
mod tests {
    use super::*;
    use jio_consensus::consensus::factory::ConsensusFactory;
    use jio_consensus_core::config::params::Params;
    use jio_consensusmanager::ConsensusManager;
    use jio_mining::{MempoolConfig, MiningManager};
    use jio_notify::Notifier;
    use jio_rpc_core::api::rpc::RpcApi;
    use jio_utxoindex::UtxoIndex;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_rpc_core_service_info() {
        let params = Params::devnet();
        let consensus = ConsensusFactory::new_instance(params);
        let consensus_mgr = ConsensusManager::new(consensus);
        let mining_mgr = MiningManager::new(consensus_mgr.clone(), MempoolConfig::default());
        let utxo_index = Arc::new(UtxoIndex::new());
        let notifier = Arc::new(Notifier::new());

        let service = RpcCoreService::new(consensus_mgr, mining_mgr, utxo_index, notifier);
        let info = service.get_info().await.expect("info success");
        assert_eq!(info.server_version, "0.1.0");
        assert!(info.is_synced);
    }
}
