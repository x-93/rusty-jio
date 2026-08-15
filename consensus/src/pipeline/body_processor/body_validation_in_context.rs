use jio_consensus_core::block::Block;
use jio_consensus_core::errors::block::BlockRuleError;
use jio_consensus_core::merkle::calc_tx_merkle_root;

pub fn validate_body_in_context(block: &Block) -> Result<(), BlockRuleError> {
    let calculated_merkle = calc_tx_merkle_root(&block.transactions);
    if calculated_merkle != block.header.hash_merkle_root {
        return Err(BlockRuleError::BadMerkleRoot {
            expected: block.header.hash_merkle_root,
            actual: calculated_merkle,
        });
    }

    Ok(())
}
