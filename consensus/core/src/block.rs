//! Block and BlockTemplate definitions.

use borsh::{BorshDeserialize, BorshSerialize};
use jio_hashes::Hash;
use serde::{Deserialize, Serialize};
use super::header::Header;
use super::tx::Transaction;

#[derive(Clone, Debug, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct Block {
    pub header: Header,
    pub transactions: Vec<Transaction>,
}

pub type MutableBlock = Block;

impl Block {
    pub fn new(header: Header, transactions: Vec<Transaction>) -> Self {
        Self { header, transactions }
    }

    pub fn hash(&self) -> Hash {
        self.header.hash
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct BlockTemplate {
    pub block: Block,
    pub selected_parent_timestamp: u64,
    pub selected_parent_daa_score: u64,
    pub is_synced: bool,
}

impl BlockTemplate {
    pub fn new(
        block: Block,
        selected_parent_timestamp: u64,
        selected_parent_daa_score: u64,
        is_synced: bool,
    ) -> Self {
        Self {
            block,
            selected_parent_timestamp,
            selected_parent_daa_score,
            is_synced,
        }
    }
}
