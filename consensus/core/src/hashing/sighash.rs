use super::sighash_type::SigHashType;
use crate::tx::Transaction;
use crate::tx::script_public_key::ScriptPublicKey;
use jio_hashes::{Hash, HasherBase, TransactionSigningHash, TransactionSigningHashECDSA};

/// Computes the Schnorr signature hash for a specific transaction input.
pub fn calc_schnorr_signature_hash(
    tx: &Transaction,
    input_index: usize,
    hash_type: SigHashType,
    script_public_key: &ScriptPublicKey,
) -> Hash {
    let mut hasher = TransactionSigningHash::new();
    serialize_sighash_into(&mut hasher, tx, input_index, hash_type, script_public_key);
    hasher.finalize()
}

/// Computes the ECDSA signature hash for a specific transaction input.
pub fn calc_ecdsa_signature_hash(
    tx: &Transaction,
    input_index: usize,
    hash_type: SigHashType,
    script_public_key: &ScriptPublicKey,
) -> Hash {
    let mut hasher = TransactionSigningHashECDSA::new();
    serialize_sighash_into(&mut hasher, tx, input_index, hash_type, script_public_key);
    hasher.finalize()
}

fn serialize_sighash_into<H: HasherBase>(
    hasher: &mut H,
    tx: &Transaction,
    input_index: usize,
    hash_type: SigHashType,
    script_public_key: &ScriptPublicKey,
) {
    hasher.update(tx.version.to_le_bytes());

    // Previous outpoints
    if !hash_type.is_anyone_can_pay() {
        let mut prev_outpoints_hasher = TransactionSigningHash::new();
        for input in &tx.inputs {
            prev_outpoints_hasher.update(input.previous_outpoint.transaction_id.as_bytes());
            prev_outpoints_hasher.update(input.previous_outpoint.index.to_le_bytes());
        }
        hasher.update(prev_outpoints_hasher.finalize().as_bytes());
    } else {
        hasher.update([0u8; 32]);
    }

    // Sequences
    if !hash_type.is_anyone_can_pay()
        && !hash_type.is_sighash_single()
        && !hash_type.is_sighash_none()
    {
        let mut sequences_hasher = TransactionSigningHash::new();
        for input in &tx.inputs {
            sequences_hasher.update(input.sequence.to_le_bytes());
        }
        hasher.update(sequences_hasher.finalize().as_bytes());
    } else {
        hasher.update([0u8; 32]);
    }

    // Current input details
    if let Some(input) = tx.inputs.get(input_index) {
        hasher.update([input.sig_op_count]);
        hasher.update(input.previous_outpoint.transaction_id.as_bytes());
        hasher.update(input.previous_outpoint.index.to_le_bytes());
        hasher.update(script_public_key.version.to_le_bytes());
        hasher.update((script_public_key.script.len() as u64).to_le_bytes());
        hasher.update(&script_public_key.script);
        hasher.update(input.sequence.to_le_bytes());
    }

    // Outputs
    if hash_type.is_sighash_all() {
        let mut outputs_hasher = TransactionSigningHash::new();
        for output in &tx.outputs {
            outputs_hasher.update(output.value.to_le_bytes());
            outputs_hasher.update(output.script_public_key.version.to_le_bytes());
            outputs_hasher.update((output.script_public_key.script.len() as u64).to_le_bytes());
            outputs_hasher.update(&output.script_public_key.script);
        }
        hasher.update(outputs_hasher.finalize().as_bytes());
    } else if hash_type.is_sighash_single() && input_index < tx.outputs.len() {
        let output = &tx.outputs[input_index];
        let mut output_hasher = TransactionSigningHash::new();
        output_hasher.update(output.value.to_le_bytes());
        output_hasher.update(output.script_public_key.version.to_le_bytes());
        output_hasher.update((output.script_public_key.script.len() as u64).to_le_bytes());
        output_hasher.update(&output.script_public_key.script);
        hasher.update(output_hasher.finalize().as_bytes());
    } else {
        hasher.update([0u8; 32]);
    }

    hasher.update(tx.lock_time.to_le_bytes());
    hasher.update(tx.subnetwork_id.as_bytes());
    hasher.update(tx.gas.to_le_bytes());

    let mut payload_hasher = TransactionSigningHash::new();
    payload_hasher.update(&tx.payload);
    hasher.update(payload_hasher.finalize().as_bytes());

    hasher.update([hash_type.to_u8()]);
}
