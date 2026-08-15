use crate::key::DbKey;

pub fn make_prefix(prefix: &[u8], key: &[u8]) -> DbKey {
    DbKey::prefix(prefix, key)
}
