use jio_consensus_core::header::Header;

pub fn serialize_header(header: &Header) -> Vec<u8> {
    serde_json::to_vec(header).unwrap_or_default()
}

pub fn deserialize_header(bytes: &[u8]) -> Option<Header> {
    serde_json::from_slice(bytes).ok()
}
