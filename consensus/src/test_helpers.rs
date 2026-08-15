use crate::processes::coinbase::{create_coinbase_transaction, serialize_coinbase_payload};
use jio_consensus_core::block::Block;
use jio_consensus_core::blockhash::BlockHash;
use jio_consensus_core::constants::BLOCK_VERSION;
use jio_consensus_core::header::Header;
use jio_hashes::Hash;
use jio_math::Uint192;
use jio_txscript::ScriptPublicKey;

pub fn create_test_block(
    parents: Vec<BlockHash>,
    timestamp: u64,
    bits: u32,
    blue_score: u64,
) -> Block {
    let spk = ScriptPublicKey::new(0, vec![1, 2, 3]);
    let mut coinbase = create_coinbase_transaction(blue_score, 5000000000, spk.clone());
    coinbase.payload = serialize_coinbase_payload(
        blue_score,
        &spk,
        &timestamp.to_le_bytes(),
    );
    let hash_merkle_root = jio_consensus_core::merkle::calc_tx_merkle_root(&[coinbase.clone()]);

    let header = Header::new_finalized(
        BLOCK_VERSION,
        vec![parents],
        hash_merkle_root,
        Hash::default(),
        Hash::default(),
        timestamp,
        bits,
        0,
        blue_score,
        Uint192::from(blue_score),
        blue_score,
        Hash::default(),
    );

    Block {
        header,
        transactions: vec![coinbase],
    }
}
