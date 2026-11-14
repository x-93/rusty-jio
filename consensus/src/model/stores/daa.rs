use jio_database::prelude::{CachedDbAccess, DB, DatabaseStorePrefixes, DbWriter, StoreResult};
use jio_hashes::Hash;
use std::sync::Arc;

pub const DAA_PREFIX: &[u8] = DatabaseStorePrefixes::DaaScore.as_ref();

pub trait DaaStoreReader {
    fn has(&self, hash: Hash) -> bool;
    fn get_daa_score(&self, hash: Hash) -> StoreResult<u64>;
}

pub trait DaaStore: DaaStoreReader {
    fn set_daa_score(
        &self,
        writer: &mut impl DbWriter,
        hash: Hash,
        daa_score: u64,
    ) -> StoreResult<()>;
}

#[derive(Clone)]
pub struct DbDaaStore {
    raw_access: CachedDbAccess<Hash, u64>,
}

impl DbDaaStore {
    pub fn new(db: Arc<DB>, cache_size: u64) -> Self {
        Self {
            raw_access: CachedDbAccess::new(db, cache_size, DAA_PREFIX),
        }
    }
}

impl DaaStoreReader for DbDaaStore {
    fn has(&self, hash: Hash) -> bool {
        self.raw_access.has(&hash).unwrap_or(false)
    }

    fn get_daa_score(&self, hash: Hash) -> StoreResult<u64> {
        self.raw_access.get(&hash)
    }
}

impl DaaStore for DbDaaStore {
    fn set_daa_score(
        &self,
        writer: &mut impl DbWriter,
        hash: Hash,
        daa_score: u64,
    ) -> StoreResult<()> {
        self.raw_access.set(writer, hash, daa_score)
    }
}
