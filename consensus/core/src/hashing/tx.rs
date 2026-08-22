use crate::tx::Transaction;
use jio_hashes::{Hash, HasherBase, TransactionHash, TransactionID};

/// Computes the non-malleable TransactionId (excludes signature script witnesses and sig_op_count).
pub fn id(tx: &Transaction) -> Hash {
    let mut hasher = TransactionID::new();
    serialize_tx_header_and_inputs(&mut hasher, tx, false);
    serialize_tx_outputs_and_payload(&mut hasher, tx);
    hasher.finalize()
}

/// Computes the full TransactionHash (includes signature scripts and witnesses).
pub fn hash(tx: &Transaction) -> Hash {
    let mut hasher = TransactionHash::new();
    serialize_tx_header_and_inputs(&mut hasher, tx, true);
    serialize_tx_outputs_and_payload(&mut hasher, tx);
    hasher.finalize()
}

fn serialize_tx_header_and_inputs<H: HasherBase>(
    hasher: &mut H,
    tx: &Transaction,
    include_witnesses: bool,
) {
    hasher.update(tx.version.to_le_bytes());

    hasher.update((tx.inputs.len() as u64).to_le_bytes());
    for input in &tx.inputs {
        hasher.update(input.previous_outpoint.transaction_id.as_bytes());
        hasher.update(input.previous_outpoint.index.to_le_bytes());
        hasher.update(input.sequence.to_le_bytes());

        if include_witnesses {
            hasher.update([input.sig_op_count]);
            hasher.update((input.signature_script.len() as u64).to_le_bytes());
            hasher.update(&input.signature_script);
        }
    }
}

fn serialize_tx_outputs_and_payload<H: HasherBase>(hasher: &mut H, tx: &Transaction) {
    hasher.update((tx.outputs.len() as u64).to_le_bytes());
    for output in &tx.outputs {
        hasher.update(output.value.to_le_bytes());
        hasher.update(output.script_public_key.version.to_le_bytes());
        hasher.update((output.script_public_key.script.len() as u64).to_le_bytes());
        hasher.update(&output.script_public_key.script);
    }

    hasher.update(tx.lock_time.to_le_bytes());
    hasher.update(tx.subnetwork_id.as_bytes());
    hasher.update(tx.gas.to_le_bytes());
    hasher.update((tx.payload.len() as u64).to_le_bytes());
    hasher.update(&tx.payload);
}
