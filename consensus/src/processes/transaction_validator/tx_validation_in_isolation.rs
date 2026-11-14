use super::errors::{TxValidationError, TxValidationResult};
use consensus_core::errors::tx::TxRuleError;
use consensus_core::mass::TxMassCalculator;
use consensus_core::tx::Transaction;
use std::collections::HashSet;

pub const DEFAULT_MAX_COINBASE_PAYLOAD_SIZE: usize = 150;

/// Validates a single transaction strictly in isolation without requiring access
/// to the UTXO set, DAG relations, or database state.
pub fn validate_tx_in_isolation(
    tx: &Transaction,
    mass_calculator: &TxMassCalculator,
    max_tx_mass: u64,
    max_coinbase_payload_size: usize,
) -> TxValidationResult<()> {
    if tx.is_coinbase() {
        // 1. Coinbase transactions must have zero inputs
        if !tx.inputs.is_empty() {
            return Err(TxValidationError::CoinbaseWithInputs);
        }

        // 2. Coinbase payload size bound
        if tx.payload.len() > max_coinbase_payload_size {
            return Err(TxValidationError::CoinbasePayloadTooLarge(
                tx.payload.len(),
                max_coinbase_payload_size,
            ));
        }

        // 3. Coinbase must produce at least one output (miner reward)
        if tx.outputs.is_empty() {
            return Err(TxValidationError::NoOutputs);
        }
    } else {
        // 1. Non-coinbase transaction must have at least one input
        if tx.inputs.is_empty() {
            return Err(TxValidationError::NoInputs);
        }

        // 2. Non-coinbase transaction must have at least one output
        if tx.outputs.is_empty() {
            return Err(TxValidationError::NoOutputs);
        }

        // 3. Reject duplicate inputs within the same transaction
        let mut spent_outpoints = HashSet::with_capacity(tx.inputs.len());
        for input in &tx.inputs {
            if !spent_outpoints.insert(input.previous_outpoint) {
                return Err(TxValidationError::DuplicateInput(input.previous_outpoint));
            }
        }
    }

    // 4. Validate output amounts: ensure no overflow and valid range
    let mut total_output_value: u64 = 0;
    for output in &tx.outputs {
        total_output_value = total_output_value
            .checked_add(output.value)
            .ok_or(TxValidationError::OutputValueOverflow)?;
    }

    // 5. Mass check against maximum allowed transaction mass
    let tx_mass = mass_calculator.calc_tx_mass(tx);
    if tx_mass > max_tx_mass {
        return Err(TxValidationError::Rule(TxRuleError::ExceedsMassLimit(
            tx_mass,
            max_tx_mass,
        )));
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

    #[test]
    fn test_isolated_validation_standard_tx() {
        let mass_calc = TxMassCalculator::new(1, 10, 1000);
        let outpoint = TransactionOutpoint::new(Hash::from_le_u64([1, 0, 0, 0]), 0);
        let input = TransactionInput::new(outpoint, vec![0x01; 64], 0, 1);
        let output =
            TransactionOutput::new(100_000, ScriptPublicKey::new(0, vec![0xaa; 32].into()));

        let tx = Transaction::new(
            1,
            vec![input],
            vec![output],
            0,
            SubnetworkId::default(),
            0,
            vec![],
        );

        assert!(validate_tx_in_isolation(&tx, &mass_calc, 100_000, 150).is_ok());
    }

    #[test]
    fn test_isolated_validation_rejects_empty_inputs_outputs() {
        let mass_calc = TxMassCalculator::new(1, 10, 1000);

        // No inputs
        let tx_no_inputs = Transaction::new(
            1,
            vec![],
            vec![TransactionOutput::new(100, ScriptPublicKey::default())],
            0,
            SubnetworkId::default(),
            0,
            vec![],
        );
        assert_eq!(
            validate_tx_in_isolation(&tx_no_inputs, &mass_calc, 100_000, 150),
            Err(TxValidationError::NoInputs)
        );

        // No outputs
        let outpoint = TransactionOutpoint::new(Hash::from_le_u64([1, 0, 0, 0]), 0);
        let input = TransactionInput::new(outpoint, vec![], 0, 1);
        let tx_no_outputs = Transaction::new(
            1,
            vec![input],
            vec![],
            0,
            SubnetworkId::default(),
            0,
            vec![],
        );
        assert_eq!(
            validate_tx_in_isolation(&tx_no_outputs, &mass_calc, 100_000, 150),
            Err(TxValidationError::NoOutputs)
        );
    }

    #[test]
    fn test_isolated_validation_duplicate_inputs() {
        let mass_calc = TxMassCalculator::new(1, 10, 1000);
        let outpoint = TransactionOutpoint::new(Hash::from_le_u64([1, 0, 0, 0]), 0);
        let input1 = TransactionInput::new(outpoint, vec![], 0, 1);
        let input2 = TransactionInput::new(outpoint, vec![], 0, 1);

        let tx = Transaction::new(
            1,
            vec![input1, input2],
            vec![TransactionOutput::new(100, ScriptPublicKey::default())],
            0,
            SubnetworkId::default(),
            0,
            vec![],
        );

        assert_eq!(
            validate_tx_in_isolation(&tx, &mass_calc, 100_000, 150),
            Err(TxValidationError::DuplicateInput(outpoint))
        );
    }

    #[test]
    fn test_isolated_validation_coinbase() {
        let mass_calc = TxMassCalculator::new(1, 10, 1000);
        let output =
            TransactionOutput::new(500_000_000, ScriptPublicKey::new(0, vec![0xbb; 32].into()));

        // Valid coinbase
        let coinbase_tx = Transaction::new(
            1,
            vec![],
            vec![output.clone()],
            0,
            SubnetworkId::COINBASE,
            0,
            vec![0x01, 0x02],
        );
        assert!(validate_tx_in_isolation(&coinbase_tx, &mass_calc, 100_000, 150).is_ok());

        // Coinbase with input -> rejected
        let outpoint = TransactionOutpoint::new(Hash::from_le_u64([1, 0, 0, 0]), 0);
        let bad_coinbase = Transaction::new(
            1,
            vec![TransactionInput::new(outpoint, vec![], 0, 1)],
            vec![output],
            0,
            SubnetworkId::COINBASE,
            0,
            vec![],
        );
        assert_eq!(
            validate_tx_in_isolation(&bad_coinbase, &mass_calc, 100_000, 150),
            Err(TxValidationError::CoinbaseWithInputs)
        );
    }
}
