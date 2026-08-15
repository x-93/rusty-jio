use crate::model::stores::acceptance_data::AcceptanceDataStore;
use crate::model::stores::daa::DaaStore;
use crate::model::stores::depth::DepthStore;
use crate::model::stores::ghostdag::GhostdagStore;
use crate::model::stores::headers::HeaderStore;
use crate::model::stores::pruning::PruningStore;
use crate::model::stores::reachability::ReachabilityStore;
use crate::model::stores::relations::RelationsStore;
use crate::model::stores::selected_chain::SelectedChainStore;
use crate::model::stores::statuses::StatusesStore;
use crate::model::stores::tips::TipsStore;
use crate::model::stores::utxo_set::UtxoSetStore;
use crate::model::stores::virtual_state::VirtualStateStore;

#[derive(Clone, Default)]
pub struct ConsensusStorage {
    pub header_store: HeaderStore,
    pub ghostdag_store: GhostdagStore,
    pub relations_store: RelationsStore,
    pub reachability_store: ReachabilityStore,
    pub statuses_store: StatusesStore,
    pub selected_chain_store: SelectedChainStore,
    pub utxo_set_store: UtxoSetStore,
    pub virtual_state_store: VirtualStateStore,
    pub tips_store: TipsStore,
    pub daa_store: DaaStore,
    pub depth_store: DepthStore,
    pub pruning_store: PruningStore,
    pub acceptance_data_store: AcceptanceDataStore,
}

impl ConsensusStorage {
    pub fn new() -> Self {
        Self::default()
    }
}
