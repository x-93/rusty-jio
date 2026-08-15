use crate::consensus::storage::ConsensusStorage;
use crate::model::services::reachability::ReachabilityService;
use crate::processes::coinbase::CoinbaseManager;
use crate::processes::difficulty::DifficultyManager;
use crate::processes::ghostdag::GhostdagManager;
use crate::processes::past_median_time::PastMedianTimeManager;
use crate::processes::pruning::PruningManager;
use crate::processes::pruning_proof::PruningProofManager;
use crate::processes::relations::RelationsManager;
use crate::processes::sync::SyncManager;
use crate::processes::traversal_manager::TraversalManager;
use jio_consensus_core::config::params::Params;

#[derive(Clone)]
pub struct ConsensusServices {
    pub reachability_service: ReachabilityService,
    pub ghostdag_manager: GhostdagManager,
    pub difficulty_manager: DifficultyManager,
    pub pmt_manager: PastMedianTimeManager,
    pub coinbase_manager: CoinbaseManager,
    pub pruning_manager: PruningManager,
    pub pruning_proof_manager: PruningProofManager,
    pub sync_manager: SyncManager,
    pub relations_manager: RelationsManager,
    pub traversal_manager: TraversalManager,
}

impl ConsensusServices {
    pub fn new(storage: &ConsensusStorage, params: &Params) -> Self {
        let reachability_service = ReachabilityService::new(
            storage.relations_store.clone(),
            storage.reachability_store.clone(),
        );
        let ghostdag_manager = GhostdagManager::new(
            params.ghostdag_k as u64,
            storage.ghostdag_store.clone(),
            storage.relations_store.clone(),
            reachability_service.clone(),
        );
        let difficulty_manager = DifficultyManager::new(
            storage.header_store.clone(),
            params.target_time_per_block,
            params.genesis.header.bits,
        );
        let pmt_manager = PastMedianTimeManager::new(
            storage.header_store.clone(),
            params.past_median_time_window_size(0),
        );
        let coinbase_manager = CoinbaseManager::new(params.clone());
        let pruning_manager = PruningManager::new(
            params.clone(),
            storage.pruning_store.clone(),
            storage.header_store.clone(),
            storage.ghostdag_store.clone(),
        );
        let pruning_proof_manager = PruningProofManager::new(
            params.clone(),
            storage.header_store.clone(),
            storage.pruning_store.clone(),
        );
        let sync_manager = SyncManager::new(
            params.clone(),
            storage.header_store.clone(),
            storage.selected_chain_store.clone(),
        );
        let relations_manager = RelationsManager::new(storage.relations_store.clone());
        let traversal_manager = TraversalManager::new(storage.relations_store.clone());

        Self {
            reachability_service,
            ghostdag_manager,
            difficulty_manager,
            pmt_manager,
            coinbase_manager,
            pruning_manager,
            pruning_proof_manager,
            sync_manager,
            relations_manager,
            traversal_manager,
        }
    }
}
