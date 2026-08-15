use jio_rpc_core::model::header::RpcHeader;

pub fn rpc_header_to_proto(header: RpcHeader) -> Vec<u8> {
    serde_json::to_vec(&header).unwrap_or_default()
}

pub fn proto_to_rpc_header(bytes: &[u8]) -> Option<RpcHeader> {
    serde_json::from_slice(bytes).ok()
}
