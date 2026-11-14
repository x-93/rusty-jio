use consensus_core::{BlockHashMap, BlueWorkType, HashKTypeMap, KType};
use jio_database::prelude::{CachedDbAccess, DB, DatabaseStorePrefixes, DbWriter, StoreResult};
use jio_hashes::Hash;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub const GHOSTDAG_PREFIX: &[u8] = DatabaseStorePrefixes::Ghostdag.as_ref();

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GhostdagData {
    pub blue_score: u64,
    pub blue_work: BlueWorkType,
    pub selected_parent: Hash,
    pub mergeset_blues: Arc<Vec<Hash>>,
    pub mergeset_reds: Arc<Vec<Hash>>,
    pub blues_anticone_sizes: HashKTypeMap,
}

impl GhostdagData {
    pub fn new(
        blue_score: u64,
        blue_work: BlueWorkType,
        selected_parent: Hash,
        mergeset_blues: Vec<Hash>,
        mergeset_reds: Vec<Hash>,
        blues_anticone_sizes: BlockHashMap<KType>,
    ) -> Self {
        Self {
            blue_score,
            blue_work,
            selected_parent,
            mergeset_blues: Arc::new(mergeset_blues),
            mergeset_reds: Arc::new(mergeset_reds),
            blues_anticone_sizes: Arc::new(blues_anticone_sizes),
        }
    }

    pub fn mergeset_size(&self) -> usize {
        self.mergeset_blues.len() + self.mergeset_reds.len()
    }
}

pub trait GhostdagStoreReader {
    fn has(&self, hash: Hash) -> bool;
    fn get(&self, hash: Hash) -> StoreResult<Arc<GhostdagData>>;
    fn get_blue_score(&self, hash: Hash) -> StoreResult<u64>;
    fn get_blue_work(&self, hash: Hash) -> StoreResult<BlueWorkType>;
    fn get_selected_parent(&self, hash: Hash) -> StoreResult<Hash>;
    fn get_mergeset_blues(&self, hash: Hash) -> StoreResult<Arc<Vec<Hash>>>;
    fn get_mergeset_reds(&self, hash: Hash) -> StoreResult<Arc<Vec<Hash>>>;
    fn get_blues_anticone_sizes(&self, hash: Hash) -> StoreResult<HashKTypeMap>;
}

pub trait GhostdagStore: GhostdagStoreReader {
    fn insert(
        &self,
        writer: &mut impl DbWriter,
        hash: Hash,
        data: Arc<GhostdagData>,
    ) -> StoreResult<()>;
}

#[derive(Clone)]
pub struct DbGhostdagStore {
    raw_access: CachedDbAccess<Hash, Arc<GhostdagData>>,
}

impl DbGhostdagStore {
    pub fn new(db: Arc<DB>, cache_size: u64) -> Self {
        Self {
            raw_access: CachedDbAccess::new(db, cache_size, GHOSTDAG_PREFIX),
        }
    }
}

impl GhostdagStoreReader for DbGhostdagStore {
    fn has(&self, hash: Hash) -> bool {
        self.raw_access.has(&hash).unwrap_or(false)
    }

    fn get(&self, hash: Hash) -> StoreResult<Arc<GhostdagData>> {
        self.raw_access.get(&hash)
    }

    fn get_blue_score(&self, hash: Hash) -> StoreResult<u64> {
        self.get(hash).map(|d| d.blue_score)
    }

    fn get_blue_work(&self, hash: Hash) -> StoreResult<BlueWorkType> {
        self.get(hash).map(|d| d.blue_work)
    }

    fn get_selected_parent(&self, hash: Hash) -> StoreResult<Hash> {
        self.get(hash).map(|d| d.selected_parent)
    }

    fn get_mergeset_blues(&self, hash: Hash) -> StoreResult<Arc<Vec<Hash>>> {
        self.get(hash).map(|d| d.mergeset_blues.clone())
    }

    fn get_mergeset_reds(&self, hash: Hash) -> StoreResult<Arc<Vec<Hash>>> {
        self.get(hash).map(|d| d.mergeset_reds.clone())
    }

    fn get_blues_anticone_sizes(&self, hash: Hash) -> StoreResult<HashKTypeMap> {
        self.get(hash).map(|d| d.blues_anticone_sizes.clone())
    }
}

impl GhostdagStore for DbGhostdagStore {
    fn insert(
        &self,
        writer: &mut impl DbWriter,
        hash: Hash,
        data: Arc<GhostdagData>,
    ) -> StoreResult<()> {
        self.raw_access.set(writer, hash, data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use consensus_core::HashMapCustomHasher;
    use jio_database::prelude::{DirectDbWriter, open_db};
    use tempfile::tempdir;

    #[test]
    fn test_ghostdag_store() {
        let temp = tempdir().unwrap();
        let db = open_db(temp.path().to_path_buf(), true, 1);
        let store = DbGhostdagStore::new(db.clone(), 100);
        let mut writer = DirectDbWriter::new(&db);

        let hash = Hash::from_bytes([5u8; 32]);
        let data = Arc::new(GhostdagData::new(
            120,
            500.into(),
            Hash::from_bytes([4u8; 32]),
            vec![Hash::from_bytes([1u8; 32])],
            vec![Hash::from_bytes([2u8; 32])],
            BlockHashMap::new(),
        ));

        store.insert(&mut writer, hash, data.clone()).unwrap();
        assert!(store.has(hash));
        assert_eq!(store.get(hash).unwrap(), data);
        assert_eq!(store.get_blue_score(hash).unwrap(), 120);
        assert_eq!(
            store.get_blue_work(hash).unwrap(),
            BlueWorkType::from_u64(500)
        );
    }
}
