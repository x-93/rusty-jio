use jio_consensus_core::header::Header;
use jio_consensus_core::{BlockLevel, BlueWorkType};
use jio_hashes::Hash;
use jio_utils::mem_size::MemSizeEstimator;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::mem::size_of;
use std::sync::Arc;

pub trait HeaderStoreReader {
    fn get_daa_score(&self, hash: Hash) -> Option<u64>;
    fn get_blue_score(&self, hash: Hash) -> Option<u64>;
    fn get_blue_work(&self, hash: Hash) -> Option<BlueWorkType>;
    fn get_timestamp(&self, hash: Hash) -> Option<u64>;
    fn get_bits(&self, hash: Hash) -> Option<u32>;
    fn get_header(&self, hash: Hash) -> Option<Arc<Header>>;
    fn get_header_with_block_level(&self, hash: Hash) -> Option<HeaderWithBlockLevel>;
    fn get_compact_header_data(&self, hash: Hash) -> Option<CompactHeaderData>;
    fn has(&self, hash: Hash) -> bool;
}

#[derive(Clone, Serialize, Deserialize)]
pub struct HeaderWithBlockLevel {
    pub header: Arc<Header>,
    pub block_level: BlockLevel,
}

impl MemSizeEstimator for HeaderWithBlockLevel {
    fn estimate_mem_bytes(&self) -> usize {
        self.header.as_ref().estimate_mem_bytes() + size_of::<Self>()
    }
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct CompactHeaderData {
    pub daa_score: u64,
    pub timestamp: u64,
    pub bits: u32,
    pub blue_score: u64,
}

impl MemSizeEstimator for CompactHeaderData {}

impl From<&Header> for CompactHeaderData {
    fn from(header: &Header) -> Self {
        Self {
            daa_score: header.daa_score,
            timestamp: header.timestamp,
            bits: header.bits,
            blue_score: header.blue_score,
        }
    }
}

/// In-memory implementation of header storage
#[derive(Default, Clone)]
pub struct HeaderStore {
    headers: Arc<RwLock<HashMap<Hash, HeaderWithBlockLevel>>>,
}

impl HeaderStore {
    pub fn new() -> Self {
        Self {
            headers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn insert(&self, hash: Hash, header: Arc<Header>) {
        self.headers.write().insert(hash, HeaderWithBlockLevel { header, block_level: 0 });
    }

    pub fn insert_with_level(&self, hash: Hash, header: Arc<Header>, block_level: BlockLevel) {
        self.headers.write().insert(hash, HeaderWithBlockLevel { header, block_level });
    }

    pub fn delete(&self, hash: Hash) {
        self.headers.write().remove(&hash);
    }
}

impl HeaderStoreReader for HeaderStore {
    fn get_daa_score(&self, hash: Hash) -> Option<u64> {
        self.headers.read().get(&hash).map(|h| h.header.daa_score)
    }

    fn get_blue_score(&self, hash: Hash) -> Option<u64> {
        self.headers.read().get(&hash).map(|h| h.header.blue_score)
    }

    fn get_blue_work(&self, hash: Hash) -> Option<BlueWorkType> {
        self.headers.read().get(&hash).map(|h| h.header.blue_work)
    }

    fn get_timestamp(&self, hash: Hash) -> Option<u64> {
        self.headers.read().get(&hash).map(|h| h.header.timestamp)
    }

    fn get_bits(&self, hash: Hash) -> Option<u32> {
        self.headers.read().get(&hash).map(|h| h.header.bits)
    }

    fn get_header(&self, hash: Hash) -> Option<Arc<Header>> {
        self.headers.read().get(&hash).map(|h| h.header.clone())
    }

    fn get_header_with_block_level(&self, hash: Hash) -> Option<HeaderWithBlockLevel> {
        self.headers.read().get(&hash).cloned()
    }

    fn get_compact_header_data(&self, hash: Hash) -> Option<CompactHeaderData> {
        self.headers.read().get(&hash).map(|h| h.header.as_ref().into())
    }

    fn has(&self, hash: Hash) -> bool {
        self.headers.read().contains_key(&hash)
    }
}

// Ergonomic helper methods accepting &Hash
impl HeaderStore {
    pub fn get_header(&self, hash: &Hash) -> Option<Arc<Header>> {
        <Self as HeaderStoreReader>::get_header(self, *hash)
    }

    pub fn has(&self, hash: &Hash) -> bool {
        <Self as HeaderStoreReader>::has(self, *hash)
    }

    pub fn get_timestamp(&self, hash: &Hash) -> Option<u64> {
        <Self as HeaderStoreReader>::get_timestamp(self, *hash)
    }

    pub fn get_bits(&self, hash: &Hash) -> Option<u32> {
        <Self as HeaderStoreReader>::get_bits(self, *hash)
    }

    pub fn get_daa_score(&self, hash: &Hash) -> Option<u64> {
        <Self as HeaderStoreReader>::get_daa_score(self, *hash)
    }

    pub fn get_blue_score(&self, hash: &Hash) -> Option<u64> {
        <Self as HeaderStoreReader>::get_blue_score(self, *hash)
    }

    pub fn get_blue_work(&self, hash: &Hash) -> Option<BlueWorkType> {
        <Self as HeaderStoreReader>::get_blue_work(self, *hash)
    }

    pub fn get_header_with_block_level(&self, hash: &Hash) -> Option<HeaderWithBlockLevel> {
        <Self as HeaderStoreReader>::get_header_with_block_level(self, *hash)
    }

    pub fn get_compact_header_data(&self, hash: &Hash) -> Option<CompactHeaderData> {
        <Self as HeaderStoreReader>::get_compact_header_data(self, *hash)
    }
}
