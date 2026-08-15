use jio_consensus_core::blockhash::BlockHash;
use jio_consensus_core::trusted::ExternalGhostdagData;
use jio_consensus_core::{BlockHashMap, HashMapCustomHasher, KType};
use jio_hashes::Hash;
use jio_math::Uint192;
use jio_utils::mem_size::MemSizeEstimator;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::mem::size_of;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GhostdagData {
    pub blue_score: u64,
    pub blue_work: Uint192,
    pub selected_parent: BlockHash,
    pub mergeset_blues: Vec<BlockHash>,
    pub mergeset_reds: Vec<BlockHash>,
    pub blues_anticone_sizes: BlockHashMap<KType>,
}

impl MemSizeEstimator for GhostdagData {
    fn estimate_mem_bytes(&self) -> usize {
        size_of::<Self>()
            + self.mergeset_blues.len() * size_of::<BlockHash>()
            + self.mergeset_reds.len() * size_of::<BlockHash>()
            + self.blues_anticone_sizes.len() * (size_of::<BlockHash>() + size_of::<KType>())
    }
}

impl GhostdagData {
    pub fn new(
        blue_score: u64,
        blue_work: Uint192,
        selected_parent: BlockHash,
        mergeset_blues: Vec<BlockHash>,
        mergeset_reds: Vec<BlockHash>,
    ) -> Self {
        Self {
            blue_score,
            blue_work,
            selected_parent,
            mergeset_blues,
            mergeset_reds,
            blues_anticone_sizes: BlockHashMap::new(),
        }
    }

    pub fn new_with_anticone_sizes(
        blue_score: u64,
        blue_work: Uint192,
        selected_parent: BlockHash,
        mergeset_blues: Vec<BlockHash>,
        mergeset_reds: Vec<BlockHash>,
        blues_anticone_sizes: BlockHashMap<KType>,
    ) -> Self {
        Self {
            blue_score,
            blue_work,
            selected_parent,
            mergeset_blues,
            mergeset_reds,
            blues_anticone_sizes,
        }
    }

    pub fn mergeset(&self) -> impl Iterator<Item = &BlockHash> {
        self.mergeset_blues.iter().chain(self.mergeset_reds.iter())
    }

    pub fn mergeset_size(&self) -> usize {
        self.mergeset_blues.len() + self.mergeset_reds.len()
    }

    pub fn to_external(&self) -> ExternalGhostdagData {
        ExternalGhostdagData {
            blue_score: self.blue_score,
            blue_work: self.blue_work,
            selected_parent: self.selected_parent,
            mergeset_blues: self.mergeset_blues.clone(),
            mergeset_reds: self.mergeset_reds.clone(),
            blues_anticone_sizes: self.blues_anticone_sizes.clone(),
        }
    }
}

impl From<&GhostdagData> for ExternalGhostdagData {
    fn from(data: &GhostdagData) -> Self {
        data.to_external()
    }
}

impl From<GhostdagData> for ExternalGhostdagData {
    fn from(data: GhostdagData) -> Self {
        data.to_external()
    }
}

impl From<&ExternalGhostdagData> for GhostdagData {
    fn from(ext: &ExternalGhostdagData) -> Self {
        GhostdagData {
            blue_score: ext.blue_score,
            blue_work: ext.blue_work,
            selected_parent: ext.selected_parent,
            mergeset_blues: ext.mergeset_blues.clone(),
            mergeset_reds: ext.mergeset_reds.clone(),
            blues_anticone_sizes: ext.blues_anticone_sizes.clone(),
        }
    }
}

impl From<ExternalGhostdagData> for GhostdagData {
    fn from(ext: ExternalGhostdagData) -> Self {
        GhostdagData {
            blue_score: ext.blue_score,
            blue_work: ext.blue_work,
            selected_parent: ext.selected_parent,
            mergeset_blues: ext.mergeset_blues,
            mergeset_reds: ext.mergeset_reds,
            blues_anticone_sizes: ext.blues_anticone_sizes,
        }
    }
}

pub trait GhostdagStoreReader {
    fn get_data(&self, hash: Hash) -> Option<Arc<GhostdagData>>;
    fn get_blue_score(&self, hash: Hash) -> Option<u64>;
    fn get_blue_work(&self, hash: Hash) -> Option<Uint192>;
    fn get_selected_parent(&self, hash: Hash) -> Option<BlockHash>;
    fn get_mergeset_blues(&self, hash: Hash) -> Option<Arc<Vec<BlockHash>>>;
    fn get_mergeset_reds(&self, hash: Hash) -> Option<Arc<Vec<BlockHash>>>;
    fn has(&self, hash: Hash) -> bool;
}

#[derive(Default, Clone)]
pub struct GhostdagStore {
    data: Arc<RwLock<HashMap<Hash, Arc<GhostdagData>>>>,
}

impl GhostdagStore {
    pub fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn insert(&self, hash: Hash, data: Arc<GhostdagData>) {
        self.data.write().insert(hash, data);
    }

    pub fn delete(&self, hash: Hash) {
        self.data.write().remove(&hash);
    }
}

impl GhostdagStoreReader for GhostdagStore {
    fn get_data(&self, hash: Hash) -> Option<Arc<GhostdagData>> {
        self.data.read().get(&hash).cloned()
    }

    fn get_blue_score(&self, hash: Hash) -> Option<u64> {
        self.data.read().get(&hash).map(|d| d.blue_score)
    }

    fn get_blue_work(&self, hash: Hash) -> Option<Uint192> {
        self.data.read().get(&hash).map(|d| d.blue_work)
    }

    fn get_selected_parent(&self, hash: Hash) -> Option<BlockHash> {
        self.data.read().get(&hash).map(|d| d.selected_parent)
    }

    fn get_mergeset_blues(&self, hash: Hash) -> Option<Arc<Vec<BlockHash>>> {
        self.data.read().get(&hash).map(|d| Arc::new(d.mergeset_blues.clone()))
    }

    fn get_mergeset_reds(&self, hash: Hash) -> Option<Arc<Vec<BlockHash>>> {
        self.data.read().get(&hash).map(|d| Arc::new(d.mergeset_reds.clone()))
    }

    fn has(&self, hash: Hash) -> bool {
        self.data.read().contains_key(&hash)
    }
}

// Ergonomic helper methods on GhostdagStore accepting &Hash or Hash
impl GhostdagStore {
    pub fn get_data(&self, hash: &Hash) -> Option<Arc<GhostdagData>> {
        <Self as GhostdagStoreReader>::get_data(self, *hash)
    }

    pub fn get_blue_score(&self, hash: &Hash) -> Option<u64> {
        <Self as GhostdagStoreReader>::get_blue_score(self, *hash)
    }

    pub fn get_blue_work(&self, hash: &Hash) -> Option<Uint192> {
        <Self as GhostdagStoreReader>::get_blue_work(self, *hash)
    }

    pub fn get_selected_parent(&self, hash: &Hash) -> Option<BlockHash> {
        <Self as GhostdagStoreReader>::get_selected_parent(self, *hash)
    }

    pub fn has(&self, hash: &Hash) -> bool {
        <Self as GhostdagStoreReader>::has(self, *hash)
    }
}
