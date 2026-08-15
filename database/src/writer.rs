use crate::key::DbKey;

pub enum WriteOp {
    Put(DbKey, Vec<u8>),
    Delete(DbKey),
}

#[derive(Default)]
pub struct MemoryWriter {
    ops: Vec<WriteOp>,
}

impl MemoryWriter {
    pub fn new() -> Self {
        Self { ops: Vec::new() }
    }

    pub fn put(&mut self, key: DbKey, value: Vec<u8>) {
        self.ops.push(WriteOp::Put(key, value));
    }

    pub fn delete(&mut self, key: DbKey) {
        self.ops.push(WriteOp::Delete(key));
    }

    pub fn ops(&self) -> &[WriteOp] {
        &self.ops
    }

    pub fn is_empty(&self) -> bool {
        self.ops.is_empty()
    }
}
