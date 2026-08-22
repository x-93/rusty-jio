use crate::header::Header;
use crate::merkle::calc_merkle_root_from_transactions;
use crate::tx::Transaction;
use borsh::{BorshDeserialize, BorshSerialize};
use jio_hashes::Hash;
use jio_utils::mem_size::MemSizeEstimator;
use serde::{Deserialize, Serialize};
use std::mem::size_of;
use std::sync::Arc;

pub type BlockArc = Arc<Block>;

/// An immutable verified block in the Jio consensus network.
#[derive(Clone, Debug, PartialEq, Eq, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Block {
    pub header: Header,
    pub transactions: Vec<Transaction>,
}

impl Block {
    pub const fn new(header: Header, transactions: Vec<Transaction>) -> Self {
        Self {
            header,
            transactions,
        }
    }

    pub fn from_header_and_transactions(header: Header, transactions: Vec<Transaction>) -> Self {
        Self {
            header,
            transactions,
        }
    }

    pub fn from_precomputed_hash(hash: Hash, parents: Vec<Hash>) -> Self {
        Self {
            header: Header::from_precomputed_hash(hash, parents),
            transactions: vec![],
        }
    }
}

impl MemSizeEstimator for Block {
    fn estimate_mem_bytes(&self) -> usize {
        size_of::<Self>()
            + self.header.estimate_mem_bytes()
            + self.transactions.len() * size_of::<Transaction>()
    }
}

/// A mutable block used for mining template construction.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MutableBlock {
    pub header: Header,
    pub transactions: Vec<Transaction>,
}

impl MutableBlock {
    pub fn new(header: Header, transactions: Vec<Transaction>) -> Self {
        Self {
            header,
            transactions,
        }
    }

    pub fn from_header(header: Header) -> Self {
        Self {
            header,
            transactions: vec![],
        }
    }

    /// Recomputes and assigns the hash Merkle root from the included transactions.
    pub fn build_hash_merkle_root(&mut self) {
        self.header.hash_merkle_root = calc_merkle_root_from_transactions(&self.transactions);
        self.header.finalize();
    }

    /// Converts into an immutable Block.
    pub fn to_immutable(self) -> Block {
        Block::new(self.header, self.transactions)
    }
}

/// A block template generated for mining work.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockTemplate {
    pub block: MutableBlock,
    pub is_synced: bool,
}

impl BlockTemplate {
    pub fn new(block: MutableBlock, is_synced: bool) -> Self {
        Self { block, is_synced }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coinbase::create_coinbase_transaction;
    use crate::tx::ScriptPublicKey;

    #[test]
    fn test_mutable_block_merkle_root_generation() {
        let header = Header::from_precomputed_hash(Hash::from([1u8; 32]), vec![]);
        let coinbase_tx =
            create_coinbase_transaction(1, 50_000_000, ScriptPublicKey::new(0, vec![0x51]), vec![]);

        let mut mblock = MutableBlock::new(header, vec![coinbase_tx]);
        mblock.build_hash_merkle_root();

        assert_ne!(mblock.header.hash_merkle_root, Hash::default());
        let block = mblock.to_immutable();
        assert_eq!(block.transactions.len(), 1);
    }
}
