use crate::{
    block::Block,
    header::Header,
    subnets::SUBNETWORK_ID_COINBASE,
    tx::{ScriptPublicKey, Transaction, TransactionOutput},
};
use jio_hashes::Hash;

/// The constants uniquely representing the genesis block
#[derive(Clone, Debug)]
pub struct GenesisBlock {
    pub hash: Hash,
    pub version: u16,
    pub hash_merkle_root: Hash,
    pub utxo_commitment: Hash,
    pub timestamp: u64,
    pub bits: u32,
    pub nonce: u64,
    pub daa_score: u64,
    pub coinbase_payload: &'static [u8],
}

impl GenesisBlock {
    pub fn build_genesis_transactions(&self) -> Vec<Transaction> {
        vec![Transaction::new(
            0,
            Vec::new(),
            Vec::new(),
            0,
            SUBNETWORK_ID_COINBASE,
            0,
            self.coinbase_payload.to_vec(),
        )]
    }
}

impl From<&GenesisBlock> for Header {
    fn from(genesis: &GenesisBlock) -> Self {
        Header::new_finalized(
            genesis.version,
            Vec::new(),
            genesis.hash_merkle_root,
            Hash::ZERO,
            genesis.utxo_commitment,
            genesis.timestamp,
            genesis.bits,
            genesis.nonce,
            genesis.daa_score,
            0.into(),
            0,
            Hash::ZERO,
        )
    }
}

impl From<&GenesisBlock> for Block {
    fn from(genesis: &GenesisBlock) -> Self {
        Block::new(genesis.into(), genesis.build_genesis_transactions())
    }
}

impl From<(&Header, &'static [u8])> for GenesisBlock {
    fn from((header, payload): (&Header, &'static [u8])) -> Self {
        Self {
            hash: header.hash,
            version: header.version,
            hash_merkle_root: header.hash_merkle_root,
            utxo_commitment: header.utxo_commitment,
            timestamp: header.timestamp,
            bits: header.bits,
            nonce: header.nonce,
            daa_score: header.daa_score,
            coinbase_payload: payload,
        }
    }
}

/// Builds the canonical Genesis Block for the JIO network.
pub fn create_genesis_block(timestamp: u64, bits: u32) -> Block {
    let coinbase_out = TransactionOutput {
        value: 0,
        script_public_key: ScriptPublicKey::new(0, vec![0x51]), // OP_TRUE
    };

    let coinbase_tx = Transaction::new(
        1,
        vec![],
        vec![coinbase_out],
        0,
        SUBNETWORK_ID_COINBASE,
        0,
        b"JIO Genesis Block - Decentralized BlockDAG with kHeavyHash".to_vec(),
    );

    let header = Header::new(
        1,
        vec![], // No parents for genesis!
        coinbase_tx.id,
        Hash::ZERO,
        Hash::ZERO,
        timestamp,
        bits,
        0,
        0,
        0.into(),
        0,
        Hash::ZERO,
    );

    Block::new(header, vec![coinbase_tx])
}
