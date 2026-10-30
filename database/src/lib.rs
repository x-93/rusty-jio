pub mod access;
pub mod batch;
pub mod cache;
pub mod db;
pub mod error;
pub mod key;
pub mod registry;

pub use access::CachedDbAccess;
pub use batch::BatchDbWriter;
pub use cache::{Cache, CachePolicy};
pub use db::DB;
pub use error::DatabaseError;
pub use key::DbKey;
pub use registry::DatabaseStorePrefixes;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prefixed_tables_and_cache_isolation() {
        let (db, _dir) = DB::create_temp_db().unwrap();

        let headers_table = CachedDbAccess::new(db.clone(), b"headers/".to_vec(), 10);
        let status_table = CachedDbAccess::new(db.clone(), b"status/".to_vec(), 10);

        let block_hash = [0xAA; 32];
        headers_table
            .write(&block_hash, b"header_data_bytes".to_vec())
            .unwrap();
        status_table
            .write(&block_hash, b"status_valid".to_vec())
            .unwrap();

        // Even though both use the same block_hash, prefixes keep them isolated on disk and cache
        assert_eq!(
            headers_table.read(&block_hash).unwrap().unwrap(),
            b"header_data_bytes"
        );
        assert_eq!(
            status_table.read(&block_hash).unwrap().unwrap(),
            b"status_valid"
        );

        // Verify cache hit
        assert!(headers_table.cache().contains_key(&block_hash.to_vec()));
        assert!(status_table.cache().contains_key(&block_hash.to_vec()));

        // Delete from one table doesn't affect the other
        headers_table.delete(&block_hash).unwrap();
        assert_eq!(headers_table.read(&block_hash).unwrap(), None);
        assert_eq!(
            status_table.read(&block_hash).unwrap().unwrap(),
            b"status_valid"
        );
    }

    #[test]
    fn test_batch_atomic_writes() {
        let (db, _dir) = DB::create_temp_db().unwrap();

        let headers_table = CachedDbAccess::new(db.clone(), b"headers/".to_vec(), 10);
        let mut batch = BatchDbWriter::new();

        let hash1 = [0x01; 32];
        let hash2 = [0x02; 32];

        headers_table.write_batch(&mut batch, &hash1, b"header_1");
        headers_table.write_batch(&mut batch, &hash2, b"header_2");

        assert_eq!(batch.len(), 2);
        db.write_batch(batch).unwrap();

        // Reading should yield the batched data
        assert_eq!(headers_table.read(&hash1).unwrap().unwrap(), b"header_1");
        assert_eq!(headers_table.read(&hash2).unwrap().unwrap(), b"header_2");
    }

    #[test]
    fn test_random_cache_policy() {
        let (db, _dir) = DB::create_temp_db().unwrap();

        let table =
            CachedDbAccess::new_with_policy(db.clone(), b"utxos/".to_vec(), CachePolicy::Random, 5);

        for i in 0..20u8 {
            table.write(&[i], vec![i * 2]).unwrap();
        }

        assert_eq!(table.cache().len(), 5);

        // Disk still has all entries
        for i in 0..20u8 {
            assert_eq!(table.read(&[i]).unwrap().unwrap(), vec![i * 2]);
        }
    }

    #[test]
    fn test_from_store_catalog() {
        let (db, _dir) = DB::create_temp_db().unwrap();

        let headers = CachedDbAccess::from_store(db.clone(), DatabaseStorePrefixes::Headers, 10);
        let ghostdag = CachedDbAccess::from_store_with_policy(
            db.clone(),
            DatabaseStorePrefixes::Ghostdag,
            CachePolicy::Random,
            10,
        );

        headers.write(b"key1", b"val_header".to_vec()).unwrap();
        ghostdag.write(b"key1", b"val_gd".to_vec()).unwrap();

        assert_eq!(headers.read(b"key1").unwrap().unwrap(), b"val_header");
        assert_eq!(ghostdag.read(b"key1").unwrap().unwrap(), b"val_gd");
    }
}
