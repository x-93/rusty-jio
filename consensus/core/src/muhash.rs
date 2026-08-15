use crate::tx::TransactionOutpoint;
use crate::utxo::UtxoEntry;
use jio_hashes::Hash;
use jio_muhash::MuHash;

pub fn calc_utxo_commitment(utxos: &[(TransactionOutpoint, UtxoEntry)]) -> Hash {
    let mut muhash = MuHash::new();
    for (outpoint, entry) in utxos {
        let mut data = Vec::with_capacity(64);
        data.extend_from_slice(&outpoint.transaction_id.as_bytes());
        data.extend_from_slice(&outpoint.index.to_le_bytes());
        data.extend_from_slice(&entry.amount.to_le_bytes());
        data.extend_from_slice(&entry.block_daa_score.to_le_bytes());
        data.extend_from_slice(&[entry.is_coinbase as u8]);
        data.extend_from_slice(entry.script_public_key.script());
        muhash.add_element(&data);
    }
    muhash.finalize()
}

#[cfg(test)]
mod tests {
    use super::*;
    use jio_txscript::ScriptPublicKey;

    #[test]
    fn test_utxo_commitment_calculation() {
        let op = TransactionOutpoint::new(Hash::from_bytes([1u8; 32]), 0);
        let entry = UtxoEntry::new(100, ScriptPublicKey::default(), 1, false);
        let hash = calc_utxo_commitment(&[(op, entry)]);
        assert_ne!(hash, Hash::default());
    }
}
