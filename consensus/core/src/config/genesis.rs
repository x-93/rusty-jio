use crate::block::Block;
use crate::header::Header;
use crate::tx::Transaction;
use jio_hashes::Hash;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GenesisBlock {
    pub header: Header,
    pub coinbase_tx: Transaction,
}

impl GenesisBlock {
    pub fn new(header: Header, coinbase_tx: Transaction) -> Self {
        Self { header, coinbase_tx }
    }

    pub fn hash(&self) -> Hash {
        self.header.hash
    }

    pub fn to_block(&self) -> Block {
        Block::new(self.header.clone(), vec![self.coinbase_tx.clone()])
    }
}