extern crate alloc;
extern crate core;
extern crate self as consensus_core;

use std::collections::{HashMap, HashSet};
use std::hash::{BuildHasher, Hasher};

pub use jio_hashes::Hash;

pub mod acceptance_data;
pub mod block;
pub mod blockhash;
pub mod coinbase;
pub mod config;
pub mod constants;
pub mod daa_score_timestamp;
pub mod hashing;
pub mod header;
pub mod mass;
pub mod pruning;
pub mod subnets;
pub mod trusted;
pub mod tx;
pub mod utxo;

pub use block::{Block, MutableBlock};
pub use header::Header;
pub use tx::{
    ScriptPublicKey, SubnetworkId, Transaction, TransactionId, TransactionInput,
    TransactionOutpoint, TransactionOutput,
};
pub use utxo::{UtxoCollection, UtxoDiff, UtxoEntry};

/// Integer type for accumulated PoW of blue blocks. We expect no more than
/// 2^128 work in a single block (btc has ~2^80), and no more than 2^64
/// overall blocks, so 2^192 is definitely a justified upper-bound.
pub type BlueWorkType = jio_math::Uint192;

/// The type used to represent the GHOSTDAG K parameter
pub type KType = u16;

/// Map from Block hash to K type
pub type HashKTypeMap = std::sync::Arc<BlockHashMap<KType>>;

/// This HashMap skips the hashing of the key and uses the key directly as the hash.
/// Should only be used for block hashes that have correct DAA,
/// otherwise it is susceptible to DOS attacks via hash collisions.
pub type BlockHashMap<V> = HashMap<Hash, V, BlockHasher>;

/// Same as `BlockHashMap` but a `HashSet`.
pub type BlockHashSet = HashSet<Hash, BlockHasher>;

pub trait HashMapCustomHasher {
    fn new() -> Self;
    fn with_capacity(capacity: usize) -> Self;
}

// HashMap::new and HashMap::with_capacity are only implemented on Hasher=RandomState
// to avoid type inference problems, so we need to provide our own versions.
impl<V> HashMapCustomHasher for BlockHashMap<V> {
    #[inline(always)]
    fn new() -> Self {
        Self::with_hasher(BlockHasher::new())
    }
    #[inline(always)]
    fn with_capacity(cap: usize) -> Self {
        Self::with_capacity_and_hasher(cap, BlockHasher::new())
    }
}

impl HashMapCustomHasher for BlockHashSet {
    #[inline(always)]
    fn new() -> Self {
        Self::with_hasher(BlockHasher::new())
    }
    #[inline(always)]
    fn with_capacity(cap: usize) -> Self {
        Self::with_capacity_and_hasher(cap, BlockHasher::new())
    }
}

#[derive(Default, Debug)]
pub struct ChainPath {
    pub added: Vec<Hash>,
    pub removed: Vec<Hash>,
}

/// `hashes::Hash` writes 4 u64s so we just use the last one as the hash here
#[derive(Default, Clone, Copy)]
pub struct BlockHasher(u64);

impl BlockHasher {
    #[inline(always)]
    pub const fn new() -> Self {
        Self(0)
    }
}

impl Hasher for BlockHasher {
    #[inline(always)]
    fn finish(&self) -> u64 {
        self.0
    }
    #[inline(always)]
    fn write_u64(&mut self, v: u64) {
        self.0 = v;
    }
    #[cold]
    fn write(&mut self, _: &[u8]) {
        unimplemented!("use write_u64")
    }
}

impl BuildHasher for BlockHasher {
    type Hasher = Self;

    #[inline(always)]
    fn build_hasher(&self) -> Self::Hasher {
        Self(0)
    }
}

pub type BlockLevel = u8;

#[cfg(test)]
mod tests {
    use super::*;
    use jio_hashes::Hash;
    use jio_math::Uint192;
    use std::hash::Hash as _;

    #[test]
    fn test_block_hasher() {
        let hash = Hash::from_le_u64([1, 2, 3, 4]);
        let mut hasher = BlockHasher::default();
        hash.hash(&mut hasher);
        assert_eq!(hasher.finish(), 4);
    }

    #[test]
    fn test_header_creation_and_hash_stability() {
        let header = Header::new(
            1,
            vec![vec![Hash::from_le_u64([1, 0, 0, 0])]],
            Hash::ZERO,
            Hash::ZERO,
            Hash::ZERO,
            1_700_000_000_000,
            0x1e7fffff,
            42,
            100,
            Uint192::from_u64(500),
            100,
            Hash::ZERO,
        );

        assert_eq!(header.direct_parents().len(), 1);
        assert!(!header.is_genesis());
        assert_ne!(header.hash, Hash::ZERO);
        assert_ne!(header.compute_pre_pow_hash(), Hash::ZERO);
        assert_ne!(header.hash, header.compute_pre_pow_hash());
    }

    #[test]
    fn test_tx_id_immutability_under_signature_change() {
        let outpoint = TransactionOutpoint::new(Hash::from_le_u64([10, 0, 0, 0]), 0);
        let mut input1 = TransactionInput {
            previous_outpoint: outpoint,
            signature_script: vec![1, 2, 3],
            sequence: 0,
            sig_op_count: 1,
        };
        let output = TransactionOutput {
            value: 50_000_000,
            script_public_key: ScriptPublicKey::new(0, vec![0x42]),
        };

        let tx1 = Transaction::new(
            1,
            vec![input1.clone()],
            vec![output.clone()],
            0,
            SubnetworkId::NATIVE,
            0,
            vec![],
        );

        // Mutate signature script
        input1.signature_script = vec![9, 9, 9, 9, 9];
        let tx2 = Transaction::new(
            1,
            vec![input1],
            vec![output],
            0,
            SubnetworkId::NATIVE,
            0,
            vec![],
        );

        // TxID MUST remain identical (non-malleable)!
        assert_eq!(tx1.id, tx2.id);
        // Full hash MUST change!
        assert_ne!(tx1.compute_hash(), tx2.compute_hash());
    }

    #[test]
    fn test_utxo_diff_reversal_and_composite() {
        let op1 = TransactionOutpoint::new(Hash::from_le_u64([1, 0, 0, 0]), 0);
        let op2 = TransactionOutpoint::new(Hash::from_le_u64([2, 0, 0, 0]), 0);
        let entry1 = UtxoEntry::new(100, ScriptPublicKey::new(0, vec![]), 1, false);
        let entry2 = UtxoEntry::new(200, ScriptPublicKey::new(0, vec![]), 2, false);

        let mut add = UtxoCollection::new();
        add.insert(op1, entry1.clone());

        let mut remove = UtxoCollection::new();
        remove.insert(op2, entry2.clone());

        let diff = UtxoDiff::new(add, remove);
        let rev = diff.reversed();

        assert_eq!(rev.add.len(), 1);
        assert_eq!(rev.remove.len(), 1);
        assert!(rev.add.contains_key(&op2));
        assert!(rev.remove.contains_key(&op1));
    }

    #[test]
    fn test_kip0009_dust_penalty_storage_mass() {
        let op = TransactionOutpoint::new(Hash::ZERO, 0);
        let input = TransactionInput {
            previous_outpoint: op,
            signature_script: vec![],
            sequence: 0,
            sig_op_count: 0,
        };
        let in_entry = UtxoEntry::new(10_000_000_000, ScriptPublicKey::new(0, vec![]), 1, false);

        // Transaction creating a 100 sompi dust output
        let dust_out = TransactionOutput {
            value: 100,
            script_public_key: ScriptPublicKey::new(0, vec![]),
        };
        let tx = Transaction::new(
            1,
            vec![input],
            vec![dust_out],
            0,
            SubnetworkId::NATIVE,
            0,
            vec![],
        );

        let storage_mass = mass::calc_storage_mass(&tx, &[in_entry]);
        // Storage mass must penalize dust with massive mass!
        assert!(storage_mass > 10_000_000);
    }
}
