use consensus_core::blockstatus::BlockStatus;
use jio_database::prelude::{CachedDbAccess, DB, DatabaseStorePrefixes, DbWriter, StoreResult};
use jio_hashes::Hash;
use std::sync::Arc;

pub const STATUSES_PREFIX: &[u8] = DatabaseStorePrefixes::Statuses.as_ref();

pub trait StatusesStoreReader {
    fn has(&self, hash: Hash) -> bool;
    fn get(&self, hash: Hash) -> StoreResult<BlockStatus>;
}

pub trait StatusesStore: StatusesStoreReader {
    fn set(&self, writer: &mut impl DbWriter, hash: Hash, status: BlockStatus) -> StoreResult<()>;
}

#[derive(Clone)]
pub struct DbStatusesStore {
    raw_access: CachedDbAccess<Hash, BlockStatus>,
}

impl DbStatusesStore {
    pub fn new(db: Arc<DB>, cache_size: u64) -> Self {
        Self {
            raw_access: CachedDbAccess::new(db, cache_size, STATUSES_PREFIX),
        }
    }
}

impl StatusesStoreReader for DbStatusesStore {
    fn has(&self, hash: Hash) -> bool {
        self.raw_access.has(&hash).unwrap_or(false)
    }

    fn get(&self, hash: Hash) -> StoreResult<BlockStatus> {
        self.raw_access.get(&hash)
    }
}

impl StatusesStore for DbStatusesStore {
    fn set(&self, writer: &mut impl DbWriter, hash: Hash, status: BlockStatus) -> StoreResult<()> {
        self.raw_access.set(writer, hash, status)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jio_database::prelude::{DirectDbWriter, open_db};
    use tempfile::tempdir;

    #[test]
    fn test_statuses_store() {
        let temp = tempdir().unwrap();
        let db = open_db(temp.path().to_path_buf(), true, 1);
        let store = DbStatusesStore::new(db.clone(), 100);
        let mut writer = DirectDbWriter::new(&db);

        let hash = Hash::from_bytes([7u8; 32]);
        store
            .set(&mut writer, hash, BlockStatus::STATUS_UTXO_VALID)
            .unwrap();

        assert!(store.has(hash));
        assert_eq!(store.get(hash).unwrap(), BlockStatus::STATUS_UTXO_VALID);
    }
}
