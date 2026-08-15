use jio_consensus_core::block::Block;
use jio_consensus_core::blockhash::BlockHash;
use jio_consensus_core::header::Header;
use jio_consensus_core::tx::{ScriptPublicKey, Transaction, TransactionOutput};
use jio_hashes::Hash;

pub struct DagBlockGenerator;

impl DagBlockGenerator {
    pub fn create_synthetic_block(
        parents: Vec<BlockHash>,
        daa_score: u64,
        timestamp: u64,
        bits: u32,
        nonce: u64,
    ) -> Block {
        let coinbase_tx = Transaction::new(
            0,
            vec![],
            vec![TransactionOutput::new(500_000_000, ScriptPublicKey::new(0, vec![0xaa, 0xbb]))],
            0,
            jio_consensus_core::subnets::SUBNETWORK_ID_COINBASE,
            0,
            daa_score.to_le_bytes().to_vec(),
        );

        let txs = vec![coinbase_tx];
        let hash_merkle_root = jio_consensus_core::merkle::calc_tx_merkle_root(&txs);

        let header = Header::new(
            1,
            vec![parents],
            hash_merkle_root,
            Hash::default(),
            Hash::default(),
            timestamp,
            bits,
            nonce,
            daa_score,
            daa_score,
            jio_math::Uint192::from_u64(daa_score),
            Hash::default(),
        );

        Block::new(header, txs)
    }
}
