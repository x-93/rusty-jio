use crate::model::stores::ghostdag::GhostdagStore;
use crate::model::stores::headers::HeaderStore;
use crate::model::stores::pruning::PruningStore;
use jio_consensus_core::blockhash::BlockHash;
use jio_consensus_core::config::params::Params;
use jio_consensus_core::errors::block::BlockRuleError;
use jio_consensus_core::header::Header;

#[derive(Clone)]
pub struct PruningManager {
    params: Params,
    pruning_store: PruningStore,
    header_store: HeaderStore,
    ghostdag_store: GhostdagStore,
}

impl PruningManager {
    pub fn new(
        params: Params,
        pruning_store: PruningStore,
        header_store: HeaderStore,
        ghostdag_store: GhostdagStore,
    ) -> Self {
        Self {
            params,
            pruning_store,
            header_store,
            ghostdag_store,
        }
    }

    /// Returns the current consensus pruning point
    pub fn pruning_point(&self) -> Option<BlockHash> {
        self.pruning_store.get_pruning_point()
    }

    /// Checks if a proposed block violates the network finality depth
    pub fn check_finality_violation(&self, header: &Header) -> Result<(), BlockRuleError> {
        if let Some(pruning_point) = self.pruning_point() {
            if let Some(pp_header) = self.header_store.get_header(&pruning_point) {
                // If the block claims a blue score lower than the pruning point blue score, it is finalized
                if header.blue_score < pp_header.blue_score {
                    return Err(BlockRuleError::FinalityViolation(
                        header.blue_score,
                        pp_header.blue_score,
                    ));
                }
            }
        }
        Ok(())
    }

    /// Checks if a block hash is beyond the pruning point horizon
    pub fn is_pruned(&self, hash: &BlockHash) -> bool {
        if let (Some(pruning_point), Some(block_header)) = (
            self.pruning_point(),
            self.header_store.get_header(hash),
        ) {
            if let Some(pp_header) = self.header_store.get_header(&pruning_point) {
                return block_header.blue_score < pp_header.blue_score;
            }
        }
        false
    }

    /// Calculates a candidate pruning point given a sink block's blue score
    pub fn calc_candidate_pruning_point(&self, sink_hash: &BlockHash) -> Option<BlockHash> {
        let sink_score = self.ghostdag_store.get_blue_score(sink_hash)?;
        if sink_score < self.params.pruning_depth {
            return self.pruning_point();
        }

        let target_score = sink_score.saturating_sub(self.params.pruning_depth);
        let mut current = *sink_hash;

        while let Some(data) = self.ghostdag_store.get_data(&current) {
            if data.blue_score <= target_score || data.selected_parent == current {
                return Some(current);
            }
            current = data.selected_parent;
        }

        Some(current)
    }

    /// Updates the consensus pruning point if candidate satisfies advancement threshold
    pub fn update_pruning_point(&self, sink_hash: &BlockHash) -> Option<BlockHash> {
        if let Some(candidate) = self.calc_candidate_pruning_point(sink_hash) {
            let current_pp = self.pruning_point();
            if current_pp != Some(candidate) {
                let current_index = self.pruning_store.get_pruning_point_index().unwrap_or(0);
                self.pruning_store.set_pruning_point(candidate, current_index + 1);
                return Some(candidate);
            }
        }
        self.pruning_point()
    }
}
