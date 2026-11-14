use consensus_core::header::{ArcHeader, Header};
use jio_database::prelude::{CachedDbAccess, DB, DbWriter, StoreResult};
use jio_hashes::Hash;
use std::sync::Arc;

pub const HEADERS_PREFIX: &[u8] = b"headers";

pub trait HeaderStoreReader {
    fn has(&self, hash: Hash) -> bool;
    fn get_header(&self, hash: Hash) -> StoreResult<ArcHeader>;
    fn get_timestamp(&self, hash: Hash) -> StoreResult<u64>;
    fn get_bits(&self, hash: Hash) -> StoreResult<u32>;
    fn get_daa_score(&self, hash: Hash) -> StoreResult<u64>;
    fn get_blue_score(&self, hash: Hash) -> StoreResult<u64>;
}

pub trait HeaderStore: HeaderStoreReader {
    fn insert(&self, writer: &mut impl DbWriter, hash: Hash, header: ArcHeader) -> StoreResult<()>;
}

#[derive(Clone)]
pub struct DbHeaderStore {
    raw_access: CachedDbAccess<Hash, Arc<Header>>,
}

impl DbHeaderStore {
    pub fn new(db: Arc<DB>, cache_size: u64) -> Self {
        Self {
            raw_access: CachedDbAccess::new(db, cache_size, HEADERS_PREFIX),
        }
    }
}

impl HeaderStoreReader for DbHeaderStore {
    fn has(&self, hash: Hash) -> bool {
        self.raw_access.has(&hash).unwrap_or(false)
    }

    fn get_header(&self, hash: Hash) -> StoreResult<ArcHeader> {
        self.raw_access.get(&hash)
    }

    fn get_timestamp(&self, hash: Hash) -> StoreResult<u64> {
        self.get_header(hash).map(|h| h.timestamp)
    }

    fn get_bits(&self, hash: Hash) -> StoreResult<u32> {
        self.get_header(hash).map(|h| h.bits)
    }

    fn get_daa_score(&self, hash: Hash) -> StoreResult<u64> {
        self.get_header(hash).map(|h| h.daa_score)
    }

    fn get_blue_score(&self, hash: Hash) -> StoreResult<u64> {
        self.get_header(hash).map(|h| h.blue_score)
    }
}

impl HeaderStore for DbHeaderStore {
    fn insert(&self, writer: &mut impl DbWriter, hash: Hash, header: ArcHeader) -> StoreResult<()> {
        self.raw_access.set(writer, hash, header)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use consensus_core::header::Header;
    use jio_database::prelude::{DirectDbWriter, open_db};
    use tempfile::tempdir;

    #[test]
    fn test_header_store_insert_get() {
        let temp = tempdir().unwrap();
        let db = open_db(temp.path().to_path_buf(), true, 1);
        let store = DbHeaderStore::new(db.clone(), 100);
        let mut writer = DirectDbWriter::new(&db);

        let header = Arc::new(Header::new_finalized(
            1,
            vec![vec![Hash::from_bytes([1u8; 32])]],
            Hash::default(),
            Hash::default(),
            Hash::default(),
            1700000000,
            0x1e7fffff,
            0,
            100,
            50.into(),
            100,
            Hash::default(),
        ));
        let hash = header.hash;

        store.insert(&mut writer, hash, header.clone()).unwrap();
        assert!(store.has(hash));
        assert_eq!(store.get_header(hash).unwrap(), header);
        assert_eq!(store.get_timestamp(hash).unwrap(), 1700000000);
        assert_eq!(store.get_bits(hash).unwrap(), 0x1e7fffff);
        assert_eq!(store.get_daa_score(hash).unwrap(), 100);
    }
}
