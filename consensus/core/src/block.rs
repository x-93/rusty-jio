use crate::blockhash::BlockHash;
use crate::header::Header;
use crate::tx::{MutableTransaction, Transaction};
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug, Default)]
pub struct Block {
    pub header: Header,
    pub transactions: Vec<Transaction>,
}

impl Block {
    pub fn new(header: Header, transactions: Vec<Transaction>) -> Self {
        Self {
            header,
            transactions,
        }
    }

    pub fn from_header(header: Header) -> Self {
        Self {
            header,
            transactions: Vec::new(),
        }
    }

    pub fn hash(&self) -> BlockHash {
        self.header.hash
    }

    pub fn tx_count(&self) -> usize {
        self.transactions.len()
    }

    pub fn is_header_only(&self) -> bool {
        self.transactions.is_empty()
    }

    pub fn coinbase_tx(&self) -> Option<&Transaction> {
        self.transactions.first().filter(|tx| tx.is_coinbase())
    }

    pub fn non_coinbase_transactions(&self) -> &[Transaction] {
        if self.transactions.first().map(|tx| tx.is_coinbase()).unwrap_or(false) {
            &self.transactions[1..]
        } else {
            &self.transactions[..]
        }
    }

    pub fn total_mass(&self) -> u64 {
        self.transactions.iter().map(|tx| tx.mass).sum()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MutableBlock {
    pub header: Header,
    pub transactions: Vec<MutableTransaction>,
}

impl MutableBlock {
    pub fn new(header: Header, transactions: Vec<MutableTransaction>) -> Self {
        Self {
            header,
            transactions,
        }
    }

    pub fn from_header(header: Header) -> Self {
        Self {
            header,
            transactions: Vec::new(),
        }
    }

    pub fn from_block(block: Block) -> Self {
        Self {
            header: block.header,
            transactions: block
                .transactions
                .into_iter()
                .map(MutableTransaction::from_tx)
                .collect(),
        }
    }

    pub fn hash(&self) -> BlockHash {
        self.header.hash
    }

    pub fn to_immutable(self) -> Block {
        Block {
            header: self.header,
            transactions: self.transactions.into_iter().map(|mtx| mtx.tx).collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_primitives_and_helpers() {
        let header = Header::default();
        let mut coinbase = Transaction::default();
        coinbase.subnetwork_id = crate::subnets::SUBNETWORK_ID_COINBASE;
        let block = Block::new(header.clone(), vec![coinbase.clone()]);

        assert_eq!(block.hash(), header.hash);
        assert_eq!(block.tx_count(), 1);
        assert!(!block.is_header_only());
        assert!(block.coinbase_tx().is_some());
        assert!(block.non_coinbase_transactions().is_empty());

        let mblock = MutableBlock::from_block(block.clone());
        assert_eq!(mblock.to_immutable(), block);
    }
}
