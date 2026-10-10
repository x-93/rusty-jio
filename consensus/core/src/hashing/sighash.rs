use super::sighash_type::SigHashType;
use crate::tx::Transaction;
use jio_hashes::{Hash, TransactionSigningHash};

pub fn calc_schnorr_signature_hash(tx: &Transaction, input_index: usize, hash_type: SigHashType) -> Hash {
    let mut hasher = TransactionSigningHash::new();
    hasher.update(tx.version.to_le_bytes());
    hasher.update([hash_type.to_u8()]);

    // Subnetwork ID and gas
    hasher.update(tx.subnetwork_id.as_bytes());
    hasher.update(tx.gas.to_le_bytes());

    // Payload
    hasher.update((tx.payload.len() as u64).to_le_bytes());
    hasher.update(&tx.payload);

    // Inputs
    if !hash_type.is_anyone_can_pay() {
        hasher.update((tx.inputs.len() as u64).to_le_bytes());
        for input in &tx.inputs {
            hasher.update(input.previous_outpoint.transaction_id.as_bytes());
            hasher.update(input.previous_outpoint.index.to_le_bytes());
            hasher.update(input.sequence.to_le_bytes());
            hasher.update([input.sig_op_count]);
        }
    } else if let Some(input) = tx.inputs.get(input_index) {
        hasher.update(input.previous_outpoint.transaction_id.as_bytes());
        hasher.update(input.previous_outpoint.index.to_le_bytes());
        hasher.update(input.sequence.to_le_bytes());
        hasher.update([input.sig_op_count]);
    }

    // Outputs
    if hash_type.is_all() {
        hasher.update((tx.outputs.len() as u64).to_le_bytes());
        for output in &tx.outputs {
            hasher.update(output.value.to_le_bytes());
            hasher.update(output.script_public_key.version().to_le_bytes());
            hasher.update((output.script_public_key.script().len() as u64).to_le_bytes());
            hasher.update(output.script_public_key.script());
        }
    } else if hash_type.is_single() {
        if let Some(output) = tx.outputs.get(input_index) {
            hasher.update(output.value.to_le_bytes());
            hasher.update(output.script_public_key.version().to_le_bytes());
            hasher.update((output.script_public_key.script().len() as u64).to_le_bytes());
            hasher.update(output.script_public_key.script());
        }
    }

    // Lock time
    hasher.update(tx.lock_time.to_le_bytes());

    hasher.finalize()
}
