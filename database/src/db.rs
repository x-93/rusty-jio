//! RocksDB LSM engine wrapper with support for multi-threading and typed column families.

use rocksdb::{ColumnFamilyDescriptor, DBWithThreadMode, MultiThreaded, Options};
use std::{marker::PhantomData, path::PathBuf, sync::Arc};

/// The DB type used for blockchain storage.
pub type DB = DBWithThreadMode<MultiThreaded>;

/// A typed column family descriptor.
pub struct TypedColumnFamily<TKey, TData> {
    pub name: &'static str,
    _marker: PhantomData<(TKey, TData)>,
}

impl<TKey, TData> TypedColumnFamily<TKey, TData> {
    pub const fn new(name: &'static str) -> Self {
        Self {
            name,
            _marker: PhantomData,
        }
    }

    pub fn handle<'a>(&self, db: &'a DB) -> Option<std::sync::Arc<rocksdb::BoundColumnFamily<'a>>> {
        db.cf_handle(self.name)
    }
}

/// Creates or loads an existing DB from the provided directory path.
pub fn open_db(db_path: PathBuf, create_if_missing: bool, parallelism: usize) -> Arc<DB> {
    let mut opts = Options::default();
    if parallelism > 1 {
        opts.increase_parallelism(parallelism as i32);
    }
    opts.create_if_missing(create_if_missing);
    let db = Arc::new(DB::open(&opts, db_path.to_str().unwrap()).unwrap());
    db
}

/// Creates or loads an existing DB with specific column families.
pub fn open_db_with_cf(
    db_path: PathBuf,
    create_if_missing: bool,
    parallelism: usize,
    column_families: &[&str],
) -> Arc<DB> {
    let mut opts = Options::default();
    if parallelism > 1 {
        opts.increase_parallelism(parallelism as i32);
    }
    opts.create_if_missing(create_if_missing);
    opts.create_missing_column_families(true);

    let cf_descriptors: Vec<ColumnFamilyDescriptor> = column_families
        .iter()
        .map(|&name| ColumnFamilyDescriptor::new(name, Options::default()))
        .collect();

    let db = Arc::new(DB::open_cf_descriptors(&opts, db_path.to_str().unwrap(), cf_descriptors).unwrap());
    db
}

/// Deletes an existing DB if it exists.
pub fn delete_db(db_dir: PathBuf) {
    if !db_dir.exists() {
        return;
    }
    let options = Options::default();
    let path = db_dir.to_str().unwrap();
    DB::destroy(&options, path).expect("DB is expected to be deletable");
}
