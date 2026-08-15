use jio_hashes::Hash;

pub type BlockHash = Hash;
pub type BlockHashes = std::sync::Arc<Vec<BlockHash>>;
pub const ORIGIN: BlockHash = Hash::from_bytes([0u8; 32]);
pub const NONE: BlockHash = Hash::from_bytes([0xff; 32]);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{BlockHashMap, BlockHashSet, HashMapCustomHasher};

    #[test]
    fn test_origin_hash_is_all_zeros() {
        assert_eq!(ORIGIN, Hash::from_bytes([0u8; 32]));
        assert_eq!(ORIGIN.as_bytes(), [0u8; 32]);
    }

    #[test]
    fn test_blockhash_collision_resistance_in_collections() {
        let mut set = BlockHashSet::new();
        let mut map = BlockHashMap::new();

        // Generate 10,000 distinct block hashes
        let count = 10_000;
        let mut hashes = Vec::with_capacity(count);

        for i in 0..count as u64 {
            // Pattern: variations across all 4 u64 words
            let h = Hash::from_le_u64([i, i.wrapping_mul(31), i.wrapping_mul(1337), i.wrapping_mul(0x9e3779b97f4a7c15)]);
            assert!(set.insert(h), "collision detected on insertion: hash {h}");
            map.insert(h, i);
            hashes.push(h);
        }

        assert_eq!(set.len(), count);
        assert_eq!(map.len(), count);

        // Verify retrieval for every element
        for (i, h) in hashes.iter().enumerate() {
            assert!(set.contains(h));
            assert_eq!(map.get(h), Some(&(i as u64)));
        }
    }

    #[test]
    fn test_blockhash_collision_resistance_with_identical_prefixes() {
        // Hashes where first 3 u64 words are identical and only last word differs
        let mut set = BlockHashSet::new();
        for i in 0..1000u64 {
            let h = Hash::from_le_u64([0xdeadbeef, 0xcafebabe, 0x12345678, i]);
            assert!(set.insert(h));
        }
        assert_eq!(set.len(), 1000);

        // Hashes where last word is identical and earlier words differ
        // BlockHasher uses the last u64 word as hash, so bucket collisions occur,
        // but HashMap/HashSet must still distinguish keys properly without dropping entries.
        let mut map = BlockHashMap::new();
        for i in 0..1000u64 {
            let h = Hash::from_le_u64([i, 0, 0, 42]);
            map.insert(h, i);
        }
        assert_eq!(map.len(), 1000);
        for i in 0..1000u64 {
            let h = Hash::from_le_u64([i, 0, 0, 42]);
            assert_eq!(map.get(&h), Some(&i));
        }
    }
}

