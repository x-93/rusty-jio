use jio_database::prelude::{CachedDbAccess, DB, DatabaseStorePrefixes, DbWriter, StoreResult};
use jio_hashes::Hash;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub const REACHABILITY_PREFIX: &[u8] = DatabaseStorePrefixes::Reachability.as_ref();

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Interval {
    pub start: u64,
    pub end: u64,
}

impl Interval {
    pub const fn new(start: u64, end: u64) -> Self {
        Self { start, end }
    }

    pub const fn empty() -> Self {
        Self { start: 0, end: 0 }
    }

    pub const fn maximal() -> Self {
        Self {
            start: 1,
            end: u64::MAX - 1,
        }
    }

    #[inline(always)]
    pub fn contains(&self, other: &Interval) -> bool {
        self.start <= other.start && other.end <= self.end
    }

    #[inline(always)]
    pub fn size(&self) -> u64 {
        self.end.saturating_sub(self.start)
    }
}

#[derive(Clone, Default, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReachabilityData {
    pub interval: Interval,
    pub parent: Hash,
    pub children: Vec<Hash>,
}

impl ReachabilityData {
    pub fn new(interval: Interval, parent: Hash, children: Vec<Hash>) -> Self {
        Self {
            interval,
            parent,
            children,
        }
    }
}

pub trait ReachabilityStoreReader {
    fn has(&self, hash: Hash) -> bool;
    fn get_interval(&self, hash: Hash) -> StoreResult<Interval>;
    fn get_parent(&self, hash: Hash) -> StoreResult<Hash>;
    fn get_children(&self, hash: Hash) -> StoreResult<Arc<Vec<Hash>>>;
    fn get_data(&self, hash: Hash) -> StoreResult<Arc<ReachabilityData>>;
}

pub trait ReachabilityStore: ReachabilityStoreReader {
    fn insert(
        &self,
        writer: &mut impl DbWriter,
        hash: Hash,
        data: Arc<ReachabilityData>,
    ) -> StoreResult<()>;
    fn set_interval(
        &self,
        writer: &mut impl DbWriter,
        hash: Hash,
        interval: Interval,
    ) -> StoreResult<()>;
    fn append_child(
        &self,
        writer: &mut impl DbWriter,
        parent: Hash,
        child: Hash,
    ) -> StoreResult<()>;
}

#[derive(Clone)]
pub struct DbReachabilityStore {
    raw_access: CachedDbAccess<Hash, Arc<ReachabilityData>>,
}

impl DbReachabilityStore {
    pub fn new(db: Arc<DB>, cache_size: u64) -> Self {
        Self {
            raw_access: CachedDbAccess::new(db, cache_size, REACHABILITY_PREFIX),
        }
    }
}

impl ReachabilityStoreReader for DbReachabilityStore {
    fn has(&self, hash: Hash) -> bool {
        self.raw_access.has(&hash).unwrap_or(false)
    }

    fn get_interval(&self, hash: Hash) -> StoreResult<Interval> {
        self.raw_access.get(&hash).map(|d| d.interval)
    }

    fn get_parent(&self, hash: Hash) -> StoreResult<Hash> {
        self.raw_access.get(&hash).map(|d| d.parent)
    }

    fn get_children(&self, hash: Hash) -> StoreResult<Arc<Vec<Hash>>> {
        self.raw_access
            .get(&hash)
            .map(|d| Arc::new(d.children.clone()))
    }

    fn get_data(&self, hash: Hash) -> StoreResult<Arc<ReachabilityData>> {
        self.raw_access.get(&hash)
    }
}

impl ReachabilityStore for DbReachabilityStore {
    fn insert(
        &self,
        writer: &mut impl DbWriter,
        hash: Hash,
        data: Arc<ReachabilityData>,
    ) -> StoreResult<()> {
        self.raw_access.set(writer, hash, data)
    }

    fn set_interval(
        &self,
        writer: &mut impl DbWriter,
        hash: Hash,
        interval: Interval,
    ) -> StoreResult<()> {
        let mut data = (*self.get_data(hash)?).clone();
        data.interval = interval;
        self.raw_access.set(writer, hash, Arc::new(data))
    }

    fn append_child(
        &self,
        writer: &mut impl DbWriter,
        parent: Hash,
        child: Hash,
    ) -> StoreResult<()> {
        let mut data = (*self.get_data(parent)?).clone();
        if !data.children.contains(&child) {
            data.children.push(child);
            self.raw_access.set(writer, parent, Arc::new(data))?;
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
    fn test_reachability_interval_containment() {
        let root_interval = Interval::new(100, 1000);
        let child_interval = Interval::new(200, 500);
        let outside_interval = Interval::new(1001, 1500);

        assert!(root_interval.contains(&child_interval));
        assert!(!root_interval.contains(&outside_interval));
    }

    #[test]
    fn test_reachability_store() {
        let temp = tempdir().unwrap();
        let db = open_db(temp.path().to_path_buf(), true, 1);
        let store = DbReachabilityStore::new(db.clone(), 100);
        let mut writer = DirectDbWriter::new(&db);

        let parent = Hash::from_bytes([1u8; 32]);
        let child = Hash::from_bytes([2u8; 32]);
        let data = Arc::new(ReachabilityData::new(
            Interval::new(10, 100),
            Hash::default(),
            vec![],
        ));

        store.insert(&mut writer, parent, data).unwrap();
        store.append_child(&mut writer, parent, child).unwrap();

        assert_eq!(store.get_children(parent).unwrap().as_slice(), &[child]);
    }
}
