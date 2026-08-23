use std::{collections::HashMap, fmt, sync::Arc};

use jio_consensus_core::KType;
use jio_database::{
    prelude::{BatchDbWriter, CachePolicy, CachedDbAccess, DbKey, DirectDbWriter, StoreError, DB},
    registry::DatabaseStorePrefixes,
};
use jio_hashes::Hash;
use parking_lot::RwLock;
use rocksdb::WriteBatch;

use crate::model::stores::ghostdag::GhostdagData;

pub trait DagknightStoreReader {
    fn get_selected_parent(&self, dk_key: DagknightKey) -> Result<Hash, StoreError>;
    fn get_data(&self, dk_key: DagknightKey) -> Result<Arc<GhostdagData>, StoreError>;
    fn has(&self, dk_key: DagknightKey) -> Result<bool, StoreError>;
}

#[derive(Clone, Copy)]
pub struct DagknightKey {
    pub pov_hash: Hash,
    pub root_hash: Hash,
    pub k: KType,
    pub free_search: bool,
    // Precomputed bytes in order: root_hash || k(u16 BE) || pov_hash || free_search
    bytes: [u8; jio_hashes::HASH_SIZE * 2 + 3],
}

impl DagknightKey {
    pub fn new(root_hash: Hash, pov_hash: Hash, k: KType, free_search: bool) -> Self {
        let mut bytes = [0u8; jio_hashes::HASH_SIZE * 2 + 3];
        let hash_size = jio_hashes::HASH_SIZE;
        bytes[..hash_size].copy_from_slice(root_hash.as_ref());

        let k_be = k.to_be_bytes();
        bytes[hash_size] = k_be[0];
        bytes[hash_size + 1] = k_be[1];

        bytes[(hash_size + 2)..(hash_size + 2 + hash_size)].copy_from_slice(pov_hash.as_ref());
        bytes[(2 * hash_size) + 2] = if free_search { 1 } else { 0 };

        Self { pov_hash, root_hash, k, free_search, bytes }
    }
}

impl fmt::Display for DagknightKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", &self.bytes)
    }
}

impl fmt::Debug for DagknightKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "DagknightKey {{ root: {}, pov: {}, k: {}, free: {} }}",
            self.root_hash, self.pov_hash, self.k, self.free_search
        )
    }
}

impl AsRef<[u8]> for DagknightKey {
    fn as_ref(&self) -> &[u8] {
        &self.bytes
    }
}

impl Eq for DagknightKey {}

impl std::hash::Hash for DagknightKey {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.root_hash.hash(state);
        self.k.hash(state);
        self.pov_hash.hash(state);
        self.free_search.hash(state);
    }
}

impl PartialEq for DagknightKey {
    fn eq(&self, other: &Self) -> bool {
        self.pov_hash == other.pov_hash
            && self.k == other.k
            && self.root_hash == other.root_hash
            && self.free_search == other.free_search
    }
}

pub trait DagknightStore {
    fn insert(&self, key: DagknightKey, dk_data: Arc<GhostdagData>) -> Result<(), StoreError>;
    fn delete(&self, key: DagknightKey) -> Result<(), StoreError>;
    fn delete_rooted_range(&self, batch: &mut WriteBatch, hash: Hash) -> Result<u32, StoreError>;
}

pub struct MemoryDagknightStore {
    dk_map: RwLock<HashMap<DagknightKey, Arc<GhostdagData>>>,
}

impl MemoryDagknightStore {
    pub fn new() -> Self {
        Self { dk_map: RwLock::new(HashMap::new()) }
    }

    pub fn from_map(dk_map: HashMap<DagknightKey, Arc<GhostdagData>>) -> Self {
        Self { dk_map: RwLock::new(dk_map) }
    }
}

impl Default for MemoryDagknightStore {
    fn default() -> Self {
        Self::new()
    }
}

impl DagknightStoreReader for MemoryDagknightStore {
    fn get_selected_parent(&self, dk_key: DagknightKey) -> Result<Hash, StoreError> {
        self.dk_map
            .read()
            .get(&dk_key)
            .map(|d| d.selected_parent)
            .ok_or_else(|| StoreError::KeyNotFound(DbKey::new(DatabaseStorePrefixes::Dagknight.as_ref(), dk_key)))
    }

    fn get_data(&self, dk_key: DagknightKey) -> Result<Arc<GhostdagData>, StoreError> {
        self.dk_map
            .read()
            .get(&dk_key)
            .cloned()
            .ok_or_else(|| StoreError::KeyNotFound(DbKey::new(DatabaseStorePrefixes::Dagknight.as_ref(), dk_key)))
    }

    fn has(&self, dk_key: DagknightKey) -> Result<bool, StoreError> {
        Ok(self.dk_map.read().contains_key(&dk_key))
    }
}

impl DagknightStore for MemoryDagknightStore {
    fn insert(&self, key: DagknightKey, dk_data: Arc<GhostdagData>) -> Result<(), StoreError> {
        self.dk_map.write().insert(key, dk_data);
        Ok(())
    }

    fn delete(&self, key: DagknightKey) -> Result<(), StoreError> {
        self.dk_map.write().remove(&key);
        Ok(())
    }

    fn delete_rooted_range(&self, _batch: &mut WriteBatch, hash: Hash) -> Result<u32, StoreError> {
        let mut map = self.dk_map.write();
        let initial_len = map.len();
        map.retain(|k, _| k.root_hash != hash);
        Ok((initial_len - map.len()) as u32)
    }
}

#[derive(Clone)]
pub struct DbDagknightStore {
    db: Arc<DB>,
    access: CachedDbAccess<DagknightKey, Arc<GhostdagData>>,
}

impl DbDagknightStore {
    pub fn new(db: Arc<DB>, cache_policy: CachePolicy) -> Self {
        Self {
            db: Arc::clone(&db),
            access: CachedDbAccess::new(db, cache_policy, DatabaseStorePrefixes::Dagknight.into()),
        }
    }

    pub fn insert_batch(&self, batch: &mut WriteBatch, key: DagknightKey, data: Arc<GhostdagData>) -> Result<(), StoreError> {
        self.access.write(BatchDbWriter::new(batch), key, data)
    }

    pub fn delete_batch(&self, batch: &mut WriteBatch, key: DagknightKey) -> Result<(), StoreError> {
        self.access.delete(BatchDbWriter::new(batch), key)
    }
}

impl DagknightStoreReader for DbDagknightStore {
    fn get_selected_parent(&self, dk_key: DagknightKey) -> Result<Hash, StoreError> {
        self.access.read(dk_key).map(|d| d.selected_parent)
    }

    fn get_data(&self, dk_key: DagknightKey) -> Result<Arc<GhostdagData>, StoreError> {
        self.access.read(dk_key)
    }

    fn has(&self, dk_key: DagknightKey) -> Result<bool, StoreError> {
        self.access.has(dk_key)
    }
}

impl DagknightStore for DbDagknightStore {
    fn insert(&self, key: DagknightKey, dk_data: Arc<GhostdagData>) -> Result<(), StoreError> {
        self.access.write(DirectDbWriter::new(&self.db), key, dk_data)
    }

    fn delete(&self, key: DagknightKey) -> Result<(), StoreError> {
        self.access.delete(DirectDbWriter::new(&self.db), key)
    }

    fn delete_rooted_range(&self, batch: &mut WriteBatch, hash: Hash) -> Result<u32, StoreError> {
        // Iterator scan for root_hash prefix and delete matching keys in batch
        let mut count = 0;
        let prefix = DatabaseStorePrefixes::Dagknight.into_iter().chain(hash.as_bytes()).collect::<Vec<u8>>();
        let mut iter = self.db.raw_iterator();
        iter.seek(&prefix);
        while iter.valid() {
            if let Some(key) = iter.key() {
                if key.starts_with(&prefix) {
                    batch.delete(key);
                    count += 1;
                    iter.next();
                    continue;
                }
            }
            break;
        }
        Ok(count)
    }
}
