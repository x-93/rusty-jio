use super::errors::{TxValidationError, TxValidationResult};
use consensus_core::errors::tx::TxRuleError;
use consensus_core::sign::{SigHashType, calc_ecdsa_signature_hash, calc_schnorr_signature_hash};
use consensus_core::tx::Transaction;
use consensus_core::utxo::UtxoView;
use jio_hashes::Hash;
use jio_txscript::{TxScriptEngine, TxSignatureVerifier};

pub struct TransactionVerifier<'a> {
    pub tx: &'a Transaction,
    pub input_idx: usize,
    pub hash_type: SigHashType,
}

impl TxSignatureVerifier for TransactionVerifier<'_> {
    fn compute_schnorr_sighash(&self) -> Hash {
        calc_schnorr_signature_hash(self.tx, self.input_idx, self.hash_type)
    }

    fn compute_ecdsa_sighash(&self) -> Hash {
        calc_ecdsa_signature_hash(self.tx, self.input_idx, self.hash_type)
    }
}

/// Validates a transaction against the current UTXO set and current DAA score:
/// 1. Verifies that all referenced UTXOs exist.
/// 2. Verifies coinbase maturity.
/// 3. Executes the TxScript engine (unlocking script + locking script) for each input.
/// 4. Verifies total input amount >= total output amount.
/// 5. Returns the transaction fee (total_input - total_output).
pub fn validate_populated_transaction_and_calc_fee(
    tx: &Transaction,
    utxo_view: &impl UtxoView,
    current_daa_score: u64,
    coinbase_maturity: u64,
) -> TxValidationResult<u64> {
    if tx.is_coinbase() {
        return Ok(0);
    }

    let mut total_input_value: u64 = 0;

    for (i, input) in tx.inputs.iter().enumerate() {
        // 1. Fetch referenced UTXO
        let utxo = utxo_view
            .get(&input.previous_outpoint)
            .ok_or(TxValidationError::Rule(TxRuleError::MissingTxOut(
                input.previous_outpoint,
            )))?;

        // 2. Enforce coinbase maturity
        if utxo.is_coinbase {
            let mature_daa_score = utxo.block_daa_score + coinbase_maturity;
            if current_daa_score < mature_daa_score {
                return Err(TxValidationError::Rule(TxRuleError::ImmatureCoinbaseSpend(
                    current_daa_score,
                    mature_daa_score,
                )));
            }
        }

        // 3. Accumulate input amount
        total_input_value = total_input_value
            .checked_add(utxo.amount)
            .ok_or(TxValidationError::OutputValueOverflow)?;

        // 4. Verify TxScript signature execution
        let mut engine = TxScriptEngine::new();
        let verifier = TransactionVerifier {
            tx,
            input_idx: i,
            hash_type: SigHashType::ALL,
        };

        let txscript_spk: jio_txscript::ScriptPublicKey = utxo.script_public_key.into();
        if let Err(err) = engine.execute(&input.signature_script, &txscript_spk, &verifier) {
            log::debug!("Transaction input {} script execution failed: {:?}", i, err);
            return Err(TxValidationError::Rule(TxRuleError::InvalidSignature(i)));
        }
    }

    // 5. Total output value
    let mut total_output_value: u64 = 0;
    for output in &tx.outputs {
        total_output_value = total_output_value
            .checked_add(output.value)
            .ok_or(TxValidationError::OutputValueOverflow)?;
    }

    // 6. Ensure inputs >= outputs
    if total_output_value > total_input_value {
        return Err(TxValidationError::SpendExceedsInputs(
            total_output_value,
            total_input_value,
        ));
    }

    let fee = total_input_value - total_output_value;
    Ok(fee)
}

#[cfg(test)]
mod tests {
    use super::*;
    use consensus_core::subnets::SubnetworkId;
    use consensus_core::tx::{
        ScriptPublicKey, TransactionInput, TransactionOutpoint, TransactionOutput, UtxoEntry,
    };
    use consensus_core::utxo::UtxoCollection;
    use jio_txscript::standard::pay_to_pubkey_script;
    use secp256k1::Secp256k1;

    #[test]
    fn test_populated_validation_schnorr_p2pk() {
        use jio_txscript::ScriptBuilder;

        let secp = Secp256k1::new();
        let keypair = secp256k1::Keypair::new(&secp, &mut rand::thread_rng());
        let (x_only, _) = keypair.x_only_public_key();

        // 1. Create a UTXO with P2PK locking script
        let p2pk_script = pay_to_pubkey_script(&x_only.serialize());
        let spk = ScriptPublicKey::from(p2pk_script);

        let outpoint = TransactionOutpoint::new(Hash::from_le_u64([42, 0, 0, 0]), 0);
        let utxo = UtxoEntry::new(1_000_000, spk, 100, false);

        let mut utxo_collection = UtxoCollection::new();
        utxo_collection.insert(outpoint, utxo);

        // 2. Construct transaction spending this UTXO
        let dummy_input = TransactionInput::new(outpoint, vec![], 0, 1);
        let output = TransactionOutput::new(990_000, ScriptPublicKey::default());
        let mut tx = Transaction::new(
            1,
            vec![dummy_input],
            vec![output],
            0,
            SubnetworkId::default(),
            0,
            vec![],
        );

        // 3. Sign transaction input using Schnorr signature
        let sighash = calc_schnorr_signature_hash(&tx, 0, SigHashType::ALL);
        let msg = secp256k1::Message::from_digest(sighash.as_bytes());
        let sig = secp.sign_schnorr(&msg, &keypair);

        // Set signature script: push 64-byte signature
        tx.inputs[0].signature_script = ScriptBuilder::new().add_data(&sig[..]).drain();

        // 4. Validate populated transaction
        let fee =
            validate_populated_transaction_and_calc_fee(&tx, &utxo_collection, 200, 100).unwrap();
        assert_eq!(fee, 10_000); // 1_000_000 - 990_000
    }

    #[test]
    fn test_populated_validation_immature_coinbase() {
        let outpoint = TransactionOutpoint::new(Hash::from_le_u64([99, 0, 0, 0]), 0);
        let utxo = UtxoEntry::new(500, ScriptPublicKey::default(), 100, true); // Coinbase at DAA 100

        let mut utxo_collection = UtxoCollection::new();
        utxo_collection.insert(outpoint, utxo);

        let tx = Transaction::new(
            1,
            vec![TransactionInput::new(outpoint, vec![], 0, 1)],
            vec![TransactionOutput::new(400, ScriptPublicKey::default())],
            0,
            SubnetworkId::default(),
            0,
            vec![],
        );

        // Maturity is 100 blocks; at current DAA score 150, 150 < 100 + 100 = 200 -> immature!
        assert_eq!(
            validate_populated_transaction_and_calc_fee(&tx, &utxo_collection, 150, 100),
            Err(TxValidationError::Rule(TxRuleError::ImmatureCoinbaseSpend(
                150, 200
            )))
        );
    }
}
