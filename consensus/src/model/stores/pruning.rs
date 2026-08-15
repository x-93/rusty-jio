use jio_consensus_core::blockhash::BlockHash;
use jio_consensus_core::pruning::PruningPointProof;
use parking_lot::RwLock;
use std::sync::Arc;

pub trait PruningStoreReader {
    fn get_pruning_point(&self) -> Option<BlockHash>;
    fn get_pruning_point_index(&self) -> Option<u64>;
    fn get_past_pruning_points(&self) -> Option<Vec<BlockHash>>;
    fn get_pruning_point_proof(&self) -> Option<Arc<PruningPointProof>>;
}

#[derive(Default, Clone)]
pub struct PruningStore {
    pruning_point: Arc<RwLock<Option<BlockHash>>>,
    pruning_point_index: Arc<RwLock<Option<u64>>>,
    past_pruning_points: Arc<RwLock<Option<Vec<BlockHash>>>>,
    pruning_point_proof: Arc<RwLock<Option<Arc<PruningPointProof>>>>,
}

impl PruningStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_pruning_point(&self, point: BlockHash, index: u64) {
        *self.pruning_point.write() = Some(point);
        *self.pruning_point_index.write() = Some(index);
    }

    pub fn set_past_pruning_points(&self, points: Vec<BlockHash>) {
        *self.past_pruning_points.write() = Some(points);
    }

    pub fn set_pruning_point_proof(&self, proof: Arc<PruningPointProof>) {
        *self.pruning_point_proof.write() = Some(proof);
    }

    pub fn get_pruning_point(&self) -> Option<BlockHash> {
        <Self as PruningStoreReader>::get_pruning_point(self)
    }

    pub fn get_pruning_point_index(&self) -> Option<u64> {
        <Self as PruningStoreReader>::get_pruning_point_index(self)
    }

    pub fn get_past_pruning_points(&self) -> Option<Vec<BlockHash>> {
        <Self as PruningStoreReader>::get_past_pruning_points(self)
    }

    pub fn get_pruning_point_proof(&self) -> Option<Arc<PruningPointProof>> {
        <Self as PruningStoreReader>::get_pruning_point_proof(self)
    }
}

impl PruningStoreReader for PruningStore {
    fn get_pruning_point(&self) -> Option<BlockHash> {
        *self.pruning_point.read()
    }

    fn get_pruning_point_index(&self) -> Option<u64> {
        *self.pruning_point_index.read()
    }

    fn get_past_pruning_points(&self) -> Option<Vec<BlockHash>> {
        self.past_pruning_points.read().clone()
    }

    fn get_pruning_point_proof(&self) -> Option<Arc<PruningPointProof>> {
        self.pruning_point_proof.read().clone()
    }
}
