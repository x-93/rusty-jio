use super::errors::{TxValidationError, TxValidationResult};
use consensus_core::mass::TxMassCalculator;
use consensus_core::tx::Transaction;
use std::collections::HashSet;

/// Validates block-level transaction constraints that do not depend on the UTXO set:
/// 1. Coinbase presence and position (must be at index 0 and only index 0).
/// 2. Intra-block transaction uniqueness (no duplicate transaction IDs).
/// 3. Aggregate block mass check against `max_block_mass`.
pub fn validate_block_transactions_not_utxo_related(
    transactions: &[Transaction],
    mass_calculator: &TxMassCalculator,
    max_block_mass: u64,
) -> TxValidationResult<()> {
    if transactions.is_empty() {
        return Err(TxValidationError::MissingCoinbase);
    }

    // 1. First transaction must be coinbase
    if !transactions[0].is_coinbase() {
        return Err(TxValidationError::CoinbaseNotFirst);
    }

    let mut seen_tx_ids = HashSet::with_capacity(transactions.len());
    let mut total_block_mass: u64 = 0;

    for (i, tx) in transactions.iter().enumerate() {
        // 2. No other transaction can be coinbase
        if i > 0 && tx.is_coinbase() {
            return Err(TxValidationError::MultipleCoinbases);
        }

        // 3. Unique transaction IDs within the block
        let tx_id = tx.id();
        if !seen_tx_ids.insert(tx_id) {
            return Err(TxValidationError::DuplicateTransactionInBlock(tx_id));
        }

        // 4. Accumulate block mass
        let tx_mass = mass_calculator.calc_tx_mass(tx);
        total_block_mass =
            total_block_mass
                .checked_add(tx_mass)
                .ok_or(TxValidationError::ExceedsBlockMass(
                    total_block_mass,
                    max_block_mass,
                ))?;

        if total_block_mass > max_block_mass {
            return Err(TxValidationError::ExceedsBlockMass(
                total_block_mass,
                max_block_mass,
            ));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use consensus_core::subnets::SubnetworkId;
    use consensus_core::tx::{
        ScriptPublicKey, TransactionInput, TransactionOutpoint, TransactionOutput,
    };
    use jio_hashes::Hash;

    fn create_test_coinbase() -> Transaction {
        Transaction::new(
            1,
            vec![],
            vec![TransactionOutput::new(500, ScriptPublicKey::default())],
            0,
            SubnetworkId::COINBASE,
            0,
            vec![0x01],
        )
    }

    fn create_test_tx(id_nonce: u64) -> Transaction {
        let outpoint = TransactionOutpoint::new(Hash::from_le_u64([id_nonce, 0, 0, 0]), 0);
        let input = TransactionInput::new(outpoint, vec![0x10], 0, 1);
        let output = TransactionOutput::new(200, ScriptPublicKey::default());
        Transaction::new(
            1,
            vec![input],
            vec![output],
            0,
            SubnetworkId::default(),
            0,
            vec![],
        )
    }

    #[test]
    fn test_block_tx_validation_valid() {
        let mass_calc = TxMassCalculator::new(1, 10, 1000);
        let cb = create_test_coinbase();
        let tx1 = create_test_tx(1);
        let tx2 = create_test_tx(2);

        assert!(
            validate_block_transactions_not_utxo_related(&[cb, tx1, tx2], &mass_calc, 500_000)
                .is_ok()
        );
    }

    #[test]
    fn test_block_tx_validation_missing_coinbase() {
        let mass_calc = TxMassCalculator::new(1, 10, 1000);
        let tx1 = create_test_tx(1);

        assert_eq!(
            validate_block_transactions_not_utxo_related(&[tx1], &mass_calc, 500_000),
            Err(TxValidationError::CoinbaseNotFirst)
        );

        assert_eq!(
            validate_block_transactions_not_utxo_related(&[], &mass_calc, 500_000),
            Err(TxValidationError::MissingCoinbase)
        );
    }

    #[test]
    fn test_block_tx_validation_multiple_coinbases() {
        let mass_calc = TxMassCalculator::new(1, 10, 1000);
        let cb1 = create_test_coinbase();
        let cb2 = create_test_coinbase();

        assert_eq!(
            validate_block_transactions_not_utxo_related(&[cb1, cb2], &mass_calc, 500_000),
            Err(TxValidationError::MultipleCoinbases)
        );
    }

    #[test]
    fn test_block_tx_validation_duplicate_tx() {
        let mass_calc = TxMassCalculator::new(1, 10, 1000);
        let cb = create_test_coinbase();
        let tx1 = create_test_tx(1);
        let tx1_dup = create_test_tx(1);

        let dup_id = tx1.id();
        assert_eq!(
            validate_block_transactions_not_utxo_related(&[cb, tx1, tx1_dup], &mass_calc, 500_000),
            Err(TxValidationError::DuplicateTransactionInBlock(dup_id))
        );
    }

    #[test]
    fn test_block_tx_validation_exceeds_block_mass() {
        let mass_calc = TxMassCalculator::new(1000, 1000, 1000);
        let cb = create_test_coinbase();
        let tx1 = create_test_tx(1);

        // Max mass 10 is too small for tx1
        assert!(matches!(
            validate_block_transactions_not_utxo_related(&[cb, tx1], &mass_calc, 10),
            Err(TxValidationError::ExceedsBlockMass(_, 10))
        ));
    }
}
