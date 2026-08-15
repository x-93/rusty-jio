use crate::errors::StoreResult;
use crate::key::DbKey;
use crate::writer::MemoryWriter;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

pub mod conn_builder;

pub trait JioDatabase: Send + Sync {
    fn get(&self, key: &DbKey) -> StoreResult<Option<Vec<u8>>>;
    fn put(&self, key: DbKey, value: Vec<u8>) -> StoreResult<()>;
    fn delete(&self, key: &DbKey) -> StoreResult<()>;
    fn write_batch(&self, writer: MemoryWriter) -> StoreResult<()>;
}

#[derive(Clone, Default)]
pub struct MemoryDatabase {
    data: Arc<RwLock<HashMap<DbKey, Vec<u8>>>>,
}

impl MemoryDatabase {
    pub fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl JioDatabase for MemoryDatabase {
    fn get(&self, key: &DbKey) -> StoreResult<Option<Vec<u8>>> {
        Ok(self.data.read().get(key).cloned())
    }

    fn put(&self, key: DbKey, value: Vec<u8>) -> StoreResult<()> {
        self.data.write().insert(key, value);
        Ok(())
    }

    fn delete(&self, key: &DbKey) -> StoreResult<()> {
        self.data.write().remove(key);
        Ok(())
    }

    fn write_batch(&self, writer: MemoryWriter) -> StoreResult<()> {
        let mut data = self.data.write();
        for op in writer.ops() {
            match op {
                crate::writer::WriteOp::Put(k, v) => {
                    data.insert(k.clone(), v.clone());
                }
                crate::writer::WriteOp::Delete(k) => {
                    data.remove(k);
                }
            }
        }
        Ok(())
    }
}
