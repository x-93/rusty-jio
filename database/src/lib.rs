//! Database engine and cache abstraction layer for Jio.

pub mod access;
pub mod cache;
pub mod db;
pub mod errors;
pub mod item;
pub mod key;
pub mod registry;
pub mod set_access;
pub mod utils;
pub mod writer;

pub use access::{CachedDbAccess, KeySerializer};
pub use cache::CachePolicy;
pub use db::DB;
pub use errors::{StoreError, StoreResult};
pub use item::CachedDbItem;
pub use key::DbKey;
pub use registry::DatabaseStorePrefixes;
pub use set_access::CachedDbSetAccess;
pub use utils::{deserialize_from_slice, serialize_to_vec};
pub use writer::{BatchDbWriter, DirectDbWriter, DbWriter, SharedBatchDbWriter};

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn test_database_cached_access() {
        let db = Arc::new(DB::open_default("").unwrap());
        let access: CachedDbAccess<String, u64> = CachedDbAccess::new(
            db.clone(),
            b"test",
            CachePolicy::Count(100),
        );

        let mut writer = BatchDbWriter::new();
        access.set(&mut writer, "key1".to_string(), 42).unwrap();
        db.write(writer).unwrap();

        assert_eq!(access.get(&"key1".to_string()).unwrap(), 42);
        assert!(access.has(&"key1".to_string()).unwrap());

        let mut del_writer = BatchDbWriter::new();
        access.delete(&mut del_writer, &"key1".to_string()).unwrap();
        db.write(del_writer).unwrap();

        assert!(access.get(&"key1".to_string()).is_err());
    }

    #[test]
    fn test_database_cached_item() {
        let db = Arc::new(DB::open_default("").unwrap());
        let item: CachedDbItem<String> = CachedDbItem::new(db.clone(), b"singleton");

        let mut writer = BatchDbWriter::new();
        item.set(&mut writer, "hello_world".to_string()).unwrap();
        db.write(writer).unwrap();

        assert_eq!(item.get().unwrap(), "hello_world");
    }
}
