pub mod access;
pub mod cache;
pub mod db;
pub mod errors;
pub mod item;
pub mod key;
pub mod registry;
pub mod writer;

pub mod prelude {
    pub use super::access::CachedDbAccess;
    pub use super::cache::Cache;
    pub use super::db::{DB, delete_db, open_db};
    pub use super::errors::{
        StoreError, StoreResult, StoreResultEmptyTuple, StoreResultExtensions,
    };
    pub use super::item::CachedDbItem;
    pub use super::key::{DbKey, SEP, SEP_SIZE};
    pub use super::registry::{DatabaseStorePrefixes, SEPARATOR};
    pub use super::writer::{BatchDbWriter, DbWriter, DirectDbWriter};
}

#[cfg(test)]
mod tests {
    use super::prelude::*;
    use tempfile::tempdir;

    #[test]
    fn test_database_crud_operations() {
        let temp = tempdir().unwrap();
        let db = open_db(temp.path().to_path_buf(), true, 1);

        let access = CachedDbAccess::<String, u64>::new(db.clone(), 10, b"metrics");
        let mut writer = DirectDbWriter::new(&db);

        // Insertion
        access.set(&mut writer, "blocks".to_string(), 42).unwrap();
        assert!(access.has(&"blocks".to_string()).unwrap());
        assert_eq!(access.get(&"blocks".to_string()).unwrap(), 42);

        // Delete
        access.delete(&mut writer, &"blocks".to_string()).unwrap();
        assert!(!access.has(&"blocks".to_string()).unwrap());

        // Batch writer
        let mut batch = BatchDbWriter::new(&db);
        access.set(&mut batch, "txs".to_string(), 100).unwrap();
        access.set(&mut batch, "peers".to_string(), 8).unwrap();
        batch.commit().unwrap();

        assert_eq!(access.get(&"txs".to_string()).unwrap(), 100);
        assert_eq!(access.get(&"peers".to_string()).unwrap(), 8);
    }
}
