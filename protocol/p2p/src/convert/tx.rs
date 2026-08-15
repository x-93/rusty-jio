use jio_consensus_core::tx::Transaction;

pub fn serialize_tx(tx: &Transaction) -> Vec<u8> {
    serde_json::to_vec(tx).unwrap_or_default()
}

pub fn deserialize_tx(bytes: &[u8]) -> Option<Transaction> {
    serde_json::from_slice(bytes).ok()
}
