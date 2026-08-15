use crate::block::Block;
use crate::constants::{BLOCK_VERSION, TX_VERSION};
use crate::header::Header;
use crate::subnets::SUBNETWORK_ID_COINBASE;
use crate::tx::{Transaction, TransactionInput, TransactionOutput, TransactionOutpoint};
use jio_hashes::Hash;
use jio_math::Uint192;
use jio_txscript::ScriptPublicKey;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GenesisBlock {
    pub header: Header,
    pub transactions: Vec<Transaction>,
}

impl GenesisBlock {
    pub const fn new(header: Header, transactions: Vec<Transaction>) -> Self {
        Self { header, transactions }
    }

    pub fn to_block(&self) -> Block {
        let mut txs = self.transactions.clone();
        if txs.is_empty() {
            let spk = ScriptPublicKey::new(0, vec![]);
            let cb = Transaction {
                version: TX_VERSION,
                inputs: vec![TransactionInput {
                    previous_outpoint: TransactionOutpoint::new(Hash::default(), u32::MAX),
                    signature_script: Vec::new(),
                    sequence: 0,
                    sig_op_count: 0,
                }],
                outputs: vec![TransactionOutput {
                    value: 0,
                    script_public_key: spk,
                }],
                lock_time: 0,
                subnetwork_id: SUBNETWORK_ID_COINBASE,
                gas: 0,
                payload: b"jio-genesis-coinbase".to_vec(),
                mass: 0,
            };
            txs.push(cb);
        }
        Block::new(self.header.clone(), txs)
    }

    pub fn hash(&self) -> Hash {
        if self.header.hash != Hash::default() {
            self.header.hash
        } else {
            crate::hashing::header::hash(&self.header)
        }
    }
}

impl From<GenesisBlock> for Block {
    fn from(gen: GenesisBlock) -> Self {
        gen.to_block()
    }
}

impl From<&GenesisBlock> for Block {
    fn from(gen: &GenesisBlock) -> Self {
        gen.to_block()
    }
}

pub const GENESIS: GenesisBlock = GenesisBlock {
    header: Header {
        hash: Hash::from_bytes([0; 32]),
        version: BLOCK_VERSION,
        parents_by_level: Vec::new(),
        hash_merkle_root: Hash::from_bytes([
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ]),
        accepted_id_merkle_root: Hash::from_bytes([0; 32]),
        utxo_commitment: Hash::from_bytes([0; 32]),
        timestamp: 1680000000000,
        bits: 0x1e7f_ffff,
        nonce: 0,
        daa_score: 0,
        blue_score: 0,
        blue_work: Uint192::ZERO,
        pruning_point: Hash::from_bytes([0; 32]),
    },
    transactions: Vec::new(),
};

pub const TESTNET_GENESIS: GenesisBlock = GenesisBlock {
    header: Header {
        hash: Hash::from_bytes([0; 32]),
        version: BLOCK_VERSION,
        parents_by_level: Vec::new(),
        hash_merkle_root: Hash::from_bytes([0; 32]),
        accepted_id_merkle_root: Hash::from_bytes([0; 32]),
        utxo_commitment: Hash::from_bytes([0; 32]),
        timestamp: 1680000000000,
        bits: 0x1e7f_ffff,
        nonce: 0,
        daa_score: 0,
        blue_score: 0,
        blue_work: Uint192::ZERO,
        pruning_point: Hash::from_bytes([0; 32]),
    },
    transactions: Vec::new(),
};

pub const TESTNET11_GENESIS: GenesisBlock = GenesisBlock {
    header: Header {
        hash: Hash::from_bytes([0; 32]),
        version: BLOCK_VERSION,
        parents_by_level: Vec::new(),
        hash_merkle_root: Hash::from_bytes([0; 32]),
        accepted_id_merkle_root: Hash::from_bytes([0; 32]),
        utxo_commitment: Hash::from_bytes([0; 32]),
        timestamp: 1680000000000,
        bits: 0x1e7f_ffff,
        nonce: 0,
        daa_score: 0,
        blue_score: 0,
        blue_work: Uint192::ZERO,
        pruning_point: Hash::from_bytes([0; 32]),
    },
    transactions: Vec::new(),
};

pub const SIMNET_GENESIS: GenesisBlock = GenesisBlock {
    header: Header {
        hash: Hash::from_bytes([0; 32]),
        version: BLOCK_VERSION,
        parents_by_level: Vec::new(),
        hash_merkle_root: Hash::from_bytes([0; 32]),
        accepted_id_merkle_root: Hash::from_bytes([0; 32]),
        utxo_commitment: Hash::from_bytes([0; 32]),
        timestamp: 1680000000000,
        bits: 0x207f_ffff,
        nonce: 0,
        daa_score: 0,
        blue_score: 0,
        blue_work: Uint192::ZERO,
        pruning_point: Hash::from_bytes([0; 32]),
    },
    transactions: Vec::new(),
};

pub const DEVNET_GENESIS: GenesisBlock = GenesisBlock {
    header: Header {
        hash: Hash::from_bytes([0; 32]),
        version: BLOCK_VERSION,
        parents_by_level: Vec::new(),
        hash_merkle_root: Hash::from_bytes([0; 32]),
        accepted_id_merkle_root: Hash::from_bytes([0; 32]),
        utxo_commitment: Hash::from_bytes([0; 32]),
        timestamp: 1680000000000,
        bits: 0x207f_ffff,
        nonce: 0,
        daa_score: 0,
        blue_score: 0,
        blue_work: Uint192::ZERO,
        pruning_point: Hash::from_bytes([0; 32]),
    },
    transactions: Vec::new(),
};

pub fn build_genesis_block(
    timestamp: u64,
    bits: u32,
    extra_payload: &[u8],
) -> GenesisBlock {
    let spk = ScriptPublicKey::new(0, vec![]);
    let payload = extra_payload.to_vec();
    let coinbase = Transaction {
        version: TX_VERSION,
        inputs: vec![TransactionInput {
            previous_outpoint: TransactionOutpoint::new(Hash::default(), u32::MAX),
            signature_script: Vec::new(),
            sequence: 0,
            sig_op_count: 0,
        }],
        outputs: vec![TransactionOutput {
            value: 0,
            script_public_key: spk,
        }],
        lock_time: 0,
        subnetwork_id: SUBNETWORK_ID_COINBASE,
        gas: 0,
        payload,
        mass: 0,
    };

    let hash_merkle_root = crate::merkle::calc_tx_merkle_root(&[coinbase.clone()]);

    let header = Header::new_finalized(
        BLOCK_VERSION,
        vec![],
        hash_merkle_root,
        Hash::default(),
        Hash::default(),
        timestamp,
        bits,
        0,
        0,
        Uint192::ZERO,
        0,
        Hash::default(),
    );

    GenesisBlock {
        header,
        transactions: vec![coinbase],
    }
}
