use crate::tx::Transaction;
use jio_hashes::{Hash, TransactionHash, TransactionID};

pub fn hash(tx: &Transaction, include_mass: bool) -> Hash {
    let mut hasher = TransactionHash::new();
    hasher.update(tx.version.to_le_bytes());
    hasher.update((tx.inputs.len() as u64).to_le_bytes());
    for input in &tx.inputs {
        hasher.update(input.previous_outpoint.transaction_id.as_bytes());
        hasher.update(input.previous_outpoint.index.to_le_bytes());
        hasher.update((input.signature_script.len() as u64).to_le_bytes());
        hasher.update(&input.signature_script);
        hasher.update(input.sequence.to_le_bytes());
        hasher.update([input.sig_op_count]);
    }
    hasher.update((tx.outputs.len() as u64).to_le_bytes());
    for output in &tx.outputs {
        hasher.update(output.value.to_le_bytes());
        hasher.update(output.script_public_key.version().to_le_bytes());
        hasher.update((output.script_public_key.script().len() as u64).to_le_bytes());
        hasher.update(output.script_public_key.script());
    }
    hasher.update(tx.lock_time.to_le_bytes());
    hasher.update(tx.subnetwork_id.as_bytes());
    hasher.update(tx.gas.to_le_bytes());
    hasher.update((tx.payload.len() as u64).to_le_bytes());
    hasher.update(&tx.payload);
    if include_mass {
        hasher.update(tx.mass.to_le_bytes());
    }
    hasher.finalize()
}

pub fn id(tx: &Transaction) -> Hash {
    let mut hasher = TransactionID::new();
    hasher.update(tx.version.to_le_bytes());
    hasher.update((tx.inputs.len() as u64).to_le_bytes());
    for input in &tx.inputs {
        hasher.update(input.previous_outpoint.transaction_id.as_bytes());
        hasher.update(input.previous_outpoint.index.to_le_bytes());
        hasher.update(input.sequence.to_le_bytes());
        hasher.update([input.sig_op_count]);
    }
    hasher.update((tx.outputs.len() as u64).to_le_bytes());
    for output in &tx.outputs {
        hasher.update(output.value.to_le_bytes());
        hasher.update(output.script_public_key.version().to_le_bytes());
        hasher.update((output.script_public_key.script().len() as u64).to_le_bytes());
        hasher.update(output.script_public_key.script());
    }
    hasher.update(tx.lock_time.to_le_bytes());
    hasher.update(tx.subnetwork_id.as_bytes());
    hasher.update(tx.gas.to_le_bytes());
    hasher.update((tx.payload.len() as u64).to_le_bytes());
    hasher.update(&tx.payload);
    hasher.finalize()
}