use crate::block::Block;
use crate::header::Header;
use crate::tx::{ScriptPublicKey, SubnetworkId, Transaction, TransactionOutput};
use jio_hashes::Hash;
use jio_math::Uint192;

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
        SubnetworkId::COINBASE,
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
        Uint192::ZERO,
        0,
        Hash::ZERO,
    );

    Block::new(header, vec![coinbase_tx])
}
