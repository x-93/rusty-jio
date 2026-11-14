use jio_database::prelude::{CachedDbAccess, DB, DbWriter, StoreResult};
use jio_hashes::Hash;
use std::sync::Arc;

pub const DEPTH_PREFIX: &[u8] = b"block-depth";

pub trait DepthStoreReader {
    fn has(&self, hash: Hash) -> bool;
    fn get_merge_depth_root(&self, hash: Hash) -> StoreResult<Hash>;
    fn get_finality_point(&self, hash: Hash) -> StoreResult<Hash>;
}

pub trait DepthStore: DepthStoreReader {
    fn set_merge_depth_root(
        &self,
        writer: &mut impl DbWriter,
        hash: Hash,
        root: Hash,
    ) -> StoreResult<()>;
    fn set_finality_point(
        &self,
        writer: &mut impl DbWriter,
        hash: Hash,
        point: Hash,
    ) -> StoreResult<()>;
}

#[derive(Clone)]
pub struct DbDepthStore {
    raw_access: CachedDbAccess<Hash, Hash>,
}

impl DbDepthStore {
    pub fn new(db: Arc<DB>, cache_size: u64) -> Self {
        Self {
            raw_access: CachedDbAccess::new(db, cache_size, DEPTH_PREFIX),
        }
    }
}

impl DepthStoreReader for DbDepthStore {
    fn has(&self, hash: Hash) -> bool {
        self.raw_access.has(&hash).unwrap_or(false)
    }

    fn get_merge_depth_root(&self, hash: Hash) -> StoreResult<Hash> {
        self.raw_access.get(&hash)
    }

    fn get_finality_point(&self, hash: Hash) -> StoreResult<Hash> {
        self.raw_access.get(&hash)
    }
}

impl DepthStore for DbDepthStore {
    fn set_merge_depth_root(
        &self,
        writer: &mut impl DbWriter,
        hash: Hash,
        root: Hash,
    ) -> StoreResult<()> {
        self.raw_access.set(writer, hash, root)
    }

    fn set_finality_point(
        &self,
        writer: &mut impl DbWriter,
        hash: Hash,
        point: Hash,
    ) -> StoreResult<()> {
        self.raw_access.set(writer, hash, point)
    }
}
