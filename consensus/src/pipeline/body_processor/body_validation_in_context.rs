use consensus_core::block::Block;
use consensus_core::errors::block::{BlockProcessResult, RuleError};
use consensus_core::merkle::calc_block_hash_merkle_root;

/// Validates block body transactions against the header commitments (Merkle root).
pub fn validate_body_in_context(block: &Block) -> BlockProcessResult<()> {
    // Genesis block is exempt from transaction commitments if empty
    if block.header.direct_parents().is_empty() && block.transactions.is_empty() {
        return Ok(());
    }

    let computed_merkle_root = calc_block_hash_merkle_root(&block.transactions);
    if computed_merkle_root != block.header.hash_merkle_root {
        log::debug!(
            "Merkle root mismatch for block {}: computed {}, committed {}",
            block.header.hash,
            computed_merkle_root,
            block.header.hash_merkle_root
        );
        return Err(RuleError::BadMerkleRoot);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use consensus_core::BlueWorkType;
    use consensus_core::header::Header;
    use consensus_core::subnets::SubnetworkId;
    use consensus_core::tx::{ScriptPublicKey, Transaction, TransactionOutput};
    use jio_hashes::Hash;

    #[test]
    fn test_body_merkle_root_validation() {
        let cb = Transaction::new(
            1,
            vec![],
            vec![TransactionOutput::new(500, ScriptPublicKey::default())],
            0,
            SubnetworkId::COINBASE,
            0,
            vec![],
        );

        let txs = vec![cb];
        let correct_root = calc_block_hash_merkle_root(&txs);

        let mut header = Header::new_finalized(
            1,
            vec![vec![Hash::default()]],
            correct_root,
            Hash::default(),
            Hash::default(),
            1000,
            0x1e7fffff,
            0,
            0,
            BlueWorkType::ZERO,
            0,
            Hash::default(),
        );

        let block = Block::new(header.clone(), txs.clone());
        assert!(validate_body_in_context(&block).is_ok());

        // Corrupted merkle root
        header.hash_merkle_root = Hash::from_le_u64([999, 0, 0, 0]);
        let bad_block = Block::new(header, txs);
        assert_eq!(
            validate_body_in_context(&bad_block),
            Err(RuleError::BadMerkleRoot)
        );
    }
}
