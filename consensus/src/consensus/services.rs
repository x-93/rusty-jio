use super::storage::ConsensusStorage;
use crate::model::stores::ghostdag::DbGhostdagStore;
use crate::model::stores::headers::DbHeaderStore;
use crate::model::stores::reachability::DbReachabilityStore;
use crate::model::stores::relations::DbRelationsStore;
use crate::processes::difficulty::DifficultyManager;
use crate::processes::ghostdag::GhostdagManager;
use crate::processes::reachability::ReachabilityManager;
use consensus_core::KType;
use jio_math::Uint256;
use std::sync::Arc;

pub type ReachabilityServiceType = ReachabilityManager<DbReachabilityStore, DbRelationsStore>;
pub type GhostdagServiceType =
    GhostdagManager<DbGhostdagStore, DbRelationsStore, DbHeaderStore, ReachabilityServiceType>;

pub struct ConsensusServices {
    pub reachability_manager: Arc<ReachabilityServiceType>,
    pub ghostdag_manager: Arc<GhostdagServiceType>,
    pub difficulty_manager: Arc<DifficultyManager>,
}

impl ConsensusServices {
    pub fn new(
        storage: &ConsensusStorage,
        k: KType,
        target_time_per_block: u64,
        max_difficulty_target: Uint256,
    ) -> Self {
        let reachability_manager = Arc::new(ReachabilityManager::new(
            storage.reachability.clone(),
            storage.relations.clone(),
        ));

        let ghostdag_manager = Arc::new(GhostdagManager::new(
            k,
            storage.ghostdag.clone(),
            storage.relations.clone(),
            storage.headers.clone(),
            reachability_manager.clone(),
        ));

        let difficulty_manager = Arc::new(DifficultyManager::new(
            target_time_per_block,
            10,
            2641,
            max_difficulty_target,
        ));

        Self {
            reachability_manager,
            ghostdag_manager,
            difficulty_manager,
        }
    }
}
