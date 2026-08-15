use crate::model::stores::headers::HeaderStore;
use crate::model::stores::pruning::PruningStore;
use jio_consensus_core::blockhash::BlockHash;
use jio_consensus_core::config::params::Params;
use jio_consensus_core::errors::pruning::PruningError;
use jio_consensus_core::header::Header;
use jio_consensus_core::pruning::PruningPointProof;
use std::sync::Arc;

#[derive(Clone)]
pub struct PruningProofManager {
    params: Params,
    header_store: HeaderStore,
    pruning_store: PruningStore,
}

impl PruningProofManager {
    pub fn new(
        params: Params,
        header_store: HeaderStore,
        pruning_store: PruningStore,
    ) -> Self {
        Self {
            params,
            header_store,
            pruning_store,
        }
    }

    /// Builds a pruning point proof consisting of sampled header levels
    pub fn build_pruning_point_proof(&self) -> Option<PruningPointProof> {
        let pp = self.pruning_store.get_pruning_point()?;
        let pp_header = self.header_store.get_header(&pp)?;

        let mut levels: Vec<Vec<Arc<Header>>> = Vec::new();
        // Level 0: Recent headers leading up to pruning point
        let mut level0 = Vec::new();
        level0.push(pp_header.clone());

        let mut current = pp;
        for _ in 0..self.params.pruning_proof_m.min(100) {
            if let Some(header) = self.header_store.get_header(&current) {
                let parents = header.direct_parents();
                if parents.is_empty() {
                    break;
                }
                current = parents[0];
                if let Some(parent_header) = self.header_store.get_header(&current) {
                    level0.push(parent_header);
                }
            } else {
                break;
            }
        }
        level0.reverse();
        levels.push(level0);

        Some(levels)
    }

    /// Validates a received pruning point proof
    pub fn validate_pruning_point_proof(
        &self,
        proof: &PruningPointProof,
    ) -> Result<BlockHash, PruningError> {
        if proof.is_empty() || proof[0].is_empty() {
            return Err(PruningError::InvalidProof("proof has no headers".to_string()));
        }

        let level0 = &proof[0];
        let mut prev_header: Option<&Arc<Header>> = None;

        for header in level0 {
            if let Some(prev) = prev_header {
                let parents = header.direct_parents();
                if !parents.contains(&prev.hash) {
                    return Err(PruningError::InvalidProof(format!(
                        "discontinuous chain in proof: block {} is not parent of {}",
                        prev.hash, header.hash
                    )));
                }
            }
            prev_header = Some(header);
        }

        let last_header = level0.last().unwrap();
        Ok(last_header.hash)
    }
}
