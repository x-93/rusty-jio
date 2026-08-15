use jio_consensus::consensus::factory::ConsensusFactory;
use jio_consensus_core::config::params::Params;
use jio_consensusmanager::ConsensusManager;
use jio_mining::{MempoolConfig, MiningManager};
use jio_notify::Notifier;
use jio_rpc_core::api::rpc::RpcApi;
use jio_rpc_service::service::RpcCoreService;
use jio_utxoindex::UtxoIndex;
use std::sync::Arc;

pub struct RpcFixture {
    pub rpc_service: Arc<RpcCoreService>,
}

impl RpcFixture {
    pub fn new() -> Self {
        let params = Params::devnet();
        let consensus = ConsensusFactory::new_instance(params);
        let consensus_manager = ConsensusManager::new(consensus);
        let mining_manager = MiningManager::new(consensus_manager.clone(), MempoolConfig::default());
        let utxo_index = Arc::new(UtxoIndex::new());
        let notifier = Arc::new(Notifier::new());

        let rpc_service = Arc::new(RpcCoreService::new(
            consensus_manager,
            mining_manager,
            utxo_index,
            notifier,
        ));

        Self { rpc_service }
    }

    pub async fn test_ping(&self) -> Result<(), String> {
        self.rpc_service.ping().await.map_err(|e| e.to_string())
    }

    pub async fn test_get_info(&self) -> Result<u64, String> {
        let info = self.rpc_service.get_info().await.map_err(|e| e.to_string())?;
        Ok(info.mempool_size)
    }
}
