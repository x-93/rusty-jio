use std::collections::{HashMap, HashSet};
use std::hash::BuildHasherDefault;
use std::hash::Hasher;

#[derive(Default)]
pub struct PassthroughHasher(u64);

impl Hasher for PassthroughHasher {
    fn finish(&self) -> u64 {
        self.0
    }
    fn write(&mut self, bytes: &[u8]) {
        for &byte in bytes {
            self.0 = self.0.rotate_left(5) ^ (byte as u64);
        }
    }
    fn write_u64(&mut self, i: u64) {
        self.0 = i;
    }
}

pub type FastHashMap<K, V> = HashMap<K, V, BuildHasherDefault<PassthroughHasher>>;
pub type FastHashSet<T> = HashSet<T, BuildHasherDefault<PassthroughHasher>>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fast_hashmap() {
        let mut map: FastHashMap<u64, &str> = FastHashMap::default();
        map.insert(42, "answer");
        assert_eq!(map.get(&42), Some(&"answer"));
    }
}
