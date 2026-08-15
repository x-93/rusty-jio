use jio_consensus_core::block::Block;
use jio_consensus_core::blockhash::BlockHash;
use crate::processes::coinbase::{create_coinbase_transaction, serialize_coinbase_payload};
use jio_consensus_core::constants::BLOCK_VERSION;
use jio_consensus_core::header::Header;
use jio_consensus_core::merkle::calc_tx_merkle_root;
use jio_consensus_core::tx::Transaction;
use jio_hashes::Hash;
use jio_math::Uint192;
use jio_txscript::ScriptPublicKey;

pub struct TestBlockBuilder {
    version: u16,
    parents: Vec<BlockHash>,
    timestamp: u64,
    bits: u32,
    nonce: u64,
    daa_score: u64,
    blue_score: u64,
    blue_work: Uint192,
    coinbase_subsidy: u64,
    coinbase_spk: ScriptPublicKey,
    extra_data: Vec<u8>,
    transactions: Vec<Transaction>,
    override_merkle_root: Option<Hash>,
    omit_coinbase: bool,
}

impl TestBlockBuilder {
    pub fn new(parents: Vec<BlockHash>, daa_score: u64) -> Self {
        Self {
            version: BLOCK_VERSION,
            parents,
            timestamp: jio_core::time::unix_now(),
            bits: 0x207f_ffff,
            nonce: 0,
            daa_score,
            blue_score: daa_score,
            blue_work: Uint192::from(daa_score),
            coinbase_subsidy: 50_000_000_000,
            coinbase_spk: ScriptPublicKey::new(0, vec![1, 2, 3]),
            extra_data: Vec::new(),
            transactions: Vec::new(),
            override_merkle_root: None,
            omit_coinbase: false,
        }
    }

    pub fn with_version(mut self, version: u16) -> Self {
        self.version = version;
        self
    }

    pub fn with_timestamp(mut self, timestamp: u64) -> Self {
        self.timestamp = timestamp;
        self
    }

    pub fn with_bits(mut self, bits: u32) -> Self {
        self.bits = bits;
        self
    }

    pub fn with_nonce(mut self, nonce: u64) -> Self {
        self.nonce = nonce;
        self
    }

    pub fn with_blue_work(mut self, blue_work: Uint192) -> Self {
        self.blue_work = blue_work;
        self
    }

    pub fn with_coinbase_subsidy(mut self, subsidy: u64) -> Self {
        self.coinbase_subsidy = subsidy;
        self
    }

    pub fn with_coinbase_spk(mut self, spk: ScriptPublicKey) -> Self {
        self.coinbase_spk = spk;
        self
    }

    pub fn with_extra_data(mut self, extra: Vec<u8>) -> Self {
        self.extra_data = extra;
        self
    }

    pub fn with_tx(mut self, tx: Transaction) -> Self {
        self.transactions.push(tx);
        self
    }

    pub fn with_override_merkle_root(mut self, root: Hash) -> Self {
        self.override_merkle_root = Some(root);
        self
    }

    pub fn omit_coinbase(mut self) -> Self {
        self.omit_coinbase = true;
        self
    }

    pub fn build(self) -> Block {
        let mut txs = Vec::new();

        if !self.omit_coinbase {
            let mut cb = create_coinbase_transaction(
                self.blue_score,
                self.coinbase_subsidy,
                self.coinbase_spk.clone(),
            );
            let extra = if self.extra_data.is_empty() {
                self.timestamp.to_le_bytes().to_vec()
            } else {
                self.extra_data
            };
            cb.payload = serialize_coinbase_payload(
                self.blue_score,
                &self.coinbase_spk,
                &extra,
            );
            txs.push(cb);
        }

        txs.extend(self.transactions);

        let hash_merkle_root = self.override_merkle_root.unwrap_or_else(|| {
            if txs.is_empty() {
                Hash::default()
            } else {
                calc_tx_merkle_root(&txs)
            }
        });

        let header = Header::new_finalized(
            self.version,
            vec![self.parents],
            hash_merkle_root,
            Hash::default(),
            Hash::default(),
            self.timestamp,
            self.bits,
            self.nonce,
            self.daa_score,
            self.blue_work,
            self.blue_score,
            Hash::default(),
        );

        Block {
            header,
            transactions: txs,
        }
    }
}
