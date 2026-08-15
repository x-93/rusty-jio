use jio_consensus_core::block::Block;

pub fn serialize_block(block: &Block) -> Vec<u8> {
    serde_json::to_vec(block).unwrap_or_default()
}

pub fn deserialize_block(bytes: &[u8]) -> Option<Block> {
    serde_json::from_slice(bytes).ok()
}
