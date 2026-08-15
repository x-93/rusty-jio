use crate::tx::{Transaction, TransactionId};
use jio_hashes::{Hash, TransactionHash, TransactionID};

pub fn tx_id(tx: &Transaction) -> TransactionId {
    let mut hasher = TransactionID::new();
    hasher.write(&tx.version.to_le_bytes());
    hasher.write(&(tx.inputs.len() as u64).to_le_bytes());
    for input in &tx.inputs {
        hasher.write(input.previous_outpoint.transaction_id);
        hasher.write(&input.previous_outpoint.index.to_le_bytes());
        hasher.write(&input.sequence.to_le_bytes());
        hasher.write(&[input.sig_op_count]);
    }
    hasher.write(&(tx.outputs.len() as u64).to_le_bytes());
    for output in &tx.outputs {
        hasher.write(&output.value.to_le_bytes());
        hasher.write(&output.script_public_key.version().to_le_bytes());
        hasher.write(&(output.script_public_key.script().len() as u64).to_le_bytes());
        hasher.write(output.script_public_key.script());
    }
    hasher.write(&tx.lock_time.to_le_bytes());
    hasher.write(tx.subnetwork_id.as_bytes());
    hasher.write(&tx.gas.to_le_bytes());
    hasher.write(&(tx.payload.len() as u64).to_le_bytes());
    hasher.write(&tx.payload);

    hasher.finalize()
}

pub fn tx_hash(tx: &Transaction) -> Hash {
    let mut hasher = TransactionHash::new();
    hasher.write(&tx.version.to_le_bytes());
    hasher.write(&(tx.inputs.len() as u64).to_le_bytes());
    for input in &tx.inputs {
        hasher.write(input.previous_outpoint.transaction_id);
        hasher.write(&input.previous_outpoint.index.to_le_bytes());
        hasher.write(&(input.signature_script.len() as u64).to_le_bytes());
        hasher.write(&input.signature_script);
        hasher.write(&input.sequence.to_le_bytes());
        hasher.write(&[input.sig_op_count]);
    }
    hasher.write(&(tx.outputs.len() as u64).to_le_bytes());
    for output in &tx.outputs {
        hasher.write(&output.value.to_le_bytes());
        hasher.write(&output.script_public_key.version().to_le_bytes());
        hasher.write(&(output.script_public_key.script().len() as u64).to_le_bytes());
        hasher.write(output.script_public_key.script());
    }
    hasher.write(&tx.lock_time.to_le_bytes());
    hasher.write(tx.subnetwork_id.as_bytes());
    hasher.write(&tx.gas.to_le_bytes());
    hasher.write(&(tx.payload.len() as u64).to_le_bytes());
    hasher.write(&tx.payload);

    hasher.finalize()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tx::{TransactionInput, TransactionOutpoint};

    #[test]
    fn test_tx_id_uniqueness() {
        let mut tx = Transaction::default();
        let id1 = tx_id(&tx);

        tx.lock_time = 100;
        let id2 = tx_id(&tx);
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_tx_id_vs_tx_hash_with_signatures() {
        let mut tx = Transaction::default();
        tx.inputs.push(TransactionInput::new(
            TransactionOutpoint::new(Hash::from_bytes([1u8; 32]), 0),
            vec![1, 2, 3],
            0,
            0,
        ));

        let id1 = tx_id(&tx);
        let hash1 = tx_hash(&tx);

        // Modifying signature script changes tx_hash but NOT tx_id
        tx.inputs[0].signature_script = vec![4, 5, 6];
        let id2 = tx_id(&tx);
        let hash2 = tx_hash(&tx);

        assert_eq!(id1, id2);
        assert_ne!(hash1, hash2);
    }
}
