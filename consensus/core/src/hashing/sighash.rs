use crate::hashing::sighash_type::SigHashType;
use crate::tx::Transaction;
use jio_hashes::{Hash, TransactionSigningHash, TransactionSigningHashECDSA};

pub fn calc_schnorr_signature_hash(
    tx: &Transaction,
    input_index: usize,
    hash_type: SigHashType,
) -> Hash {
    let mut hasher = TransactionSigningHash::new();
    hasher.write(&tx.version.to_le_bytes());
    hasher.write(&[hash_type.to_u8()]);
    hasher.write(&(input_index as u64).to_le_bytes());

    if !hash_type.is_anyone_can_pay() {
        for input in &tx.inputs {
            hasher.write(input.previous_outpoint.transaction_id);
            hasher.write(&input.previous_outpoint.index.to_le_bytes());
            hasher.write(&input.sequence.to_le_bytes());
        }
    } else if input_index < tx.inputs.len() {
        let input = &tx.inputs[input_index];
        hasher.write(input.previous_outpoint.transaction_id);
        hasher.write(&input.previous_outpoint.index.to_le_bytes());
        hasher.write(&input.sequence.to_le_bytes());
    }

    if !hash_type.is_none() {
        if hash_type.is_single() {
            if input_index < tx.outputs.len() {
                let output = &tx.outputs[input_index];
                hasher.write(&output.value.to_le_bytes());
                hasher.write(output.script_public_key.script());
            }
        } else {
            for output in &tx.outputs {
                hasher.write(&output.value.to_le_bytes());
                hasher.write(output.script_public_key.script());
            }
        }
    }

    hasher.write(&tx.lock_time.to_le_bytes());
    hasher.write(tx.subnetwork_id.as_bytes());
    hasher.write(&tx.gas.to_le_bytes());
    hasher.write(&tx.payload);

    hasher.finalize()
}

pub fn calc_ecdsa_signature_hash(
    tx: &Transaction,
    input_index: usize,
    hash_type: SigHashType,
) -> Hash {
    let mut hasher = TransactionSigningHashECDSA::new();
    hasher.write(&tx.version.to_le_bytes());
    hasher.write(&[hash_type.to_u8()]);
    hasher.write(&(input_index as u64).to_le_bytes());

    if !hash_type.is_anyone_can_pay() {
        for input in &tx.inputs {
            hasher.write(input.previous_outpoint.transaction_id);
            hasher.write(&input.previous_outpoint.index.to_le_bytes());
            hasher.write(&input.sequence.to_le_bytes());
        }
    } else if input_index < tx.inputs.len() {
        let input = &tx.inputs[input_index];
        hasher.write(input.previous_outpoint.transaction_id);
        hasher.write(&input.previous_outpoint.index.to_le_bytes());
        hasher.write(&input.sequence.to_le_bytes());
    }

    if !hash_type.is_none() {
        if hash_type.is_single() {
            if input_index < tx.outputs.len() {
                let output = &tx.outputs[input_index];
                hasher.write(&output.value.to_le_bytes());
                hasher.write(output.script_public_key.script());
            }
        } else {
            for output in &tx.outputs {
                hasher.write(&output.value.to_le_bytes());
                hasher.write(output.script_public_key.script());
            }
        }
    }

    hasher.write(&tx.lock_time.to_le_bytes());
    hasher.write(tx.subnetwork_id.as_bytes());
    hasher.write(&tx.gas.to_le_bytes());
    hasher.write(&tx.payload);

    hasher.finalize()
}
