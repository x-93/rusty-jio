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

pub use access::*;
pub use cache::*;
pub use db::*;
pub use errors::*;
pub use item::*;
pub use key::*;
pub use registry::*;
pub use set_access::*;
pub use utils::*;
pub use writer::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_database_crud() {
        let db = MemoryDatabase::new();
        let key = DbKey::prefix(b"headers", b"block1");
        let value = vec![1, 2, 3, 4];

        assert_eq!(db.get(&key).unwrap(), None);
        db.put(key.clone(), value.clone()).unwrap();
        assert_eq!(db.get(&key).unwrap(), Some(value));
        db.delete(&key).unwrap();
        assert_eq!(db.get(&key).unwrap(), None);
    }
}
