use jio_consensus_core::utxo::UtxoEntry;

pub fn serialize_utxo_entry(entry: &UtxoEntry) -> Vec<u8> {
    serde_json::to_vec(entry).unwrap_or_default()
}

pub fn deserialize_utxo_entry(bytes: &[u8]) -> Option<UtxoEntry> {
    serde_json::from_slice(bytes).ok()
}
