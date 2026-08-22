use crate::BlueWorkType;
use crate::block::Block;
use crate::header::Header;
use crate::subnets::SUBNETWORK_ID_COINBASE;
use crate::tx::{ScriptPublicKey, Transaction, TransactionOutput};
use jio_hashes::Hash;

/// Represents the genesis block configuration of a Jio network.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GenesisBlock {
    pub header: Header,
    pub coinbase_payload: Vec<u8>,
}

impl GenesisBlock {
    pub fn build_genesis_block(timestamp: u64, bits: u32, coinbase_payload: Vec<u8>) -> Self {
        let header = Header::new_finalized(
            crate::constants::BLOCK_VERSION,
            vec![],
            Hash::default(),
            Hash::default(),
            Hash::default(),
            timestamp,
            bits,
            0,
            0,
            BlueWorkType::from_u64(0),
            0,
            Hash::default(),
        );

        Self {
            header,
            coinbase_payload,
        }
    }

    pub fn to_block(&self) -> Block {
        let coinbase_tx = Transaction::new(
            0,
            vec![],
            vec![TransactionOutput::new(0, ScriptPublicKey::new(0, vec![]))],
            0,
            SUBNETWORK_ID_COINBASE,
            0,
            self.coinbase_payload.clone(),
        );
        Block::new(self.header.clone(), vec![coinbase_tx])
    }

    pub fn hash(&self) -> Hash {
        self.header.hash
    }
}
