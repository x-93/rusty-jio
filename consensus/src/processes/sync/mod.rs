use crate::model::stores::headers::HeaderStore;
use crate::model::stores::selected_chain::SelectedChainStore;
use jio_consensus_core::blockhash::BlockHash;
use jio_consensus_core::config::params::Params;

#[derive(Clone)]
pub struct SyncManager {
    params: Params,
    header_store: HeaderStore,
    selected_chain_store: SelectedChainStore,
}

impl SyncManager {
    pub fn new(
        params: Params,
        header_store: HeaderStore,
        selected_chain_store: SelectedChainStore,
    ) -> Self {
        Self {
            params,
            header_store,
            selected_chain_store,
        }
    }

    /// Determines if the local node is nearly synchronized with the network tip
    pub fn is_nearly_synced(&self) -> bool {
        if let Some(tip) = self.selected_chain_store.get_tip() {
            if let Some(header) = self.header_store.get_header(&tip) {
                return self.params.is_nearly_synced(header.timestamp, header.daa_score);
            }
        }
        false
    }

    /// Collects a chain segment window for synchronization
    pub fn get_sync_window(&self, high: &BlockHash, low: &BlockHash, max_blocks: usize) -> Vec<BlockHash> {
        let mut window = Vec::new();
        let mut current = *high;

        while current != *low && window.len() < max_blocks {
            window.push(current);
            if let Some(header) = self.header_store.get_header(&current) {
                let parents = header.direct_parents();
                if parents.is_empty() {
                    break;
                }
                current = parents[0];
            } else {
                break;
            }
        }

        window
    }
}
