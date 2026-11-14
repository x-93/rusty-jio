use jio_database::prelude::{CachedDbAccess, DB, DatabaseStorePrefixes, DbWriter, StoreResult};
use jio_hashes::Hash;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub const PARENTS_PREFIX: &[u8] = DatabaseStorePrefixes::RelationsParents.as_ref();
pub const CHILDREN_PREFIX: &[u8] = DatabaseStorePrefixes::RelationsChildren.as_ref();

#[derive(Clone, Default, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockChildren(pub Vec<Hash>);

pub trait RelationsStoreReader {
    fn has(&self, hash: Hash) -> bool;
    fn get_parents(&self, hash: Hash) -> StoreResult<Arc<Vec<Hash>>>;
    fn get_children(&self, hash: Hash) -> StoreResult<Arc<Vec<Hash>>>;
}

pub trait RelationsStore: RelationsStoreReader {
    fn insert(
        &self,
        writer: &mut impl DbWriter,
        hash: Hash,
        parents: Arc<Vec<Hash>>,
    ) -> StoreResult<()>;
    fn add_child(&self, writer: &mut impl DbWriter, parent: Hash, child: Hash) -> StoreResult<()>;
}

#[derive(Clone)]
pub struct DbRelationsStore {
    parents_access: CachedDbAccess<Hash, Arc<Vec<Hash>>>,
    children_access: CachedDbAccess<Hash, Arc<Vec<Hash>>>,
}

impl DbRelationsStore {
    pub fn new(db: Arc<DB>, cache_size: u64) -> Self {
        Self {
            parents_access: CachedDbAccess::new(db.clone(), cache_size, PARENTS_PREFIX),
            children_access: CachedDbAccess::new(db, cache_size, CHILDREN_PREFIX),
        }
    }
}

impl RelationsStoreReader for DbRelationsStore {
    fn has(&self, hash: Hash) -> bool {
        self.parents_access.has(&hash).unwrap_or(false)
    }

    fn get_parents(&self, hash: Hash) -> StoreResult<Arc<Vec<Hash>>> {
        self.parents_access.get(&hash)
    }

    fn get_children(&self, hash: Hash) -> StoreResult<Arc<Vec<Hash>>> {
        match self.children_access.get(&hash) {
            Ok(children) => Ok(children),
            Err(jio_database::prelude::StoreError::KeyNotFound(_)) => Ok(Arc::new(Vec::new())),
            Err(e) => Err(e),
        }
    }
}

impl RelationsStore for DbRelationsStore {
    fn insert(
        &self,
        writer: &mut impl DbWriter,
        hash: Hash,
        parents: Arc<Vec<Hash>>,
    ) -> StoreResult<()> {
        self.parents_access.set(writer, hash, parents)
    }

    fn add_child(&self, writer: &mut impl DbWriter, parent: Hash, child: Hash) -> StoreResult<()> {
        let mut children = match self.children_access.get(&parent) {
            Ok(c) => (*c).clone(),
            Err(jio_database::prelude::StoreError::KeyNotFound(_)) => Vec::new(),
            Err(e) => return Err(e),
        };
        if !children.contains(&child) {
            children.push(child);
            self.children_access
                .set(writer, parent, Arc::new(children))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jio_database::prelude::{DirectDbWriter, open_db};
    use tempfile::tempdir;

    #[test]
    fn test_relations_store_parents_and_children() {
        let temp = tempdir().unwrap();
        let db = open_db(temp.path().to_path_buf(), true, 1);
        let store = DbRelationsStore::new(db.clone(), 100);
        let mut writer = DirectDbWriter::new(&db);

        let parent1 = Hash::from_bytes([1u8; 32]);
        let parent2 = Hash::from_bytes([2u8; 32]);
        let child = Hash::from_bytes([3u8; 32]);

        let parents = Arc::new(vec![parent1, parent2]);
        store.insert(&mut writer, child, parents.clone()).unwrap();
        store.add_child(&mut writer, parent1, child).unwrap();
        store.add_child(&mut writer, parent2, child).unwrap();

        assert_eq!(store.get_parents(child).unwrap(), parents);
        assert_eq!(store.get_children(parent1).unwrap(), Arc::new(vec![child]));
        assert_eq!(store.get_children(parent2).unwrap(), Arc::new(vec![child]));
    }
}
