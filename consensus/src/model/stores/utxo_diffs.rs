use consensus_core::utxo::utxo_diff::UtxoDiff;
use jio_database::prelude::{CachedDbAccess, DB, DbWriter, StoreResult};
use jio_hashes::Hash;
use std::sync::Arc;

pub const UTXO_DIFFS_PREFIX: &[u8] = b"utxo-diffs";

pub trait UtxoDiffsStoreReader {
    fn has(&self, hash: Hash) -> bool;
    fn get_utxo_diff(&self, hash: Hash) -> StoreResult<Arc<UtxoDiff>>;
}

pub trait UtxoDiffsStore: UtxoDiffsStoreReader {
    fn set_utxo_diff(
        &self,
        writer: &mut impl DbWriter,
        hash: Hash,
        diff: Arc<UtxoDiff>,
    ) -> StoreResult<()>;
}

#[derive(Clone)]
pub struct DbUtxoDiffsStore {
    raw_access: CachedDbAccess<Hash, Arc<UtxoDiff>>,
}

impl DbUtxoDiffsStore {
    pub fn new(db: Arc<DB>, cache_size: u64) -> Self {
        Self {
            raw_access: CachedDbAccess::new(db, cache_size, UTXO_DIFFS_PREFIX),
        }
    }
}

impl UtxoDiffsStoreReader for DbUtxoDiffsStore {
    fn has(&self, hash: Hash) -> bool {
        self.raw_access.has(&hash).unwrap_or(false)
    }

    fn get_utxo_diff(&self, hash: Hash) -> StoreResult<Arc<UtxoDiff>> {
        self.raw_access.get(&hash)
    }
}

impl UtxoDiffsStore for DbUtxoDiffsStore {
    fn set_utxo_diff(
        &self,
        writer: &mut impl DbWriter,
        hash: Hash,
        diff: Arc<UtxoDiff>,
    ) -> StoreResult<()> {
        self.raw_access.set(writer, hash, diff)
    }
}
