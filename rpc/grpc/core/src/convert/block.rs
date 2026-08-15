use jio_rpc_core::model::block::RpcBlock;

pub fn rpc_block_to_proto(block: RpcBlock) -> Vec<u8> {
    serde_json::to_vec(&block).unwrap_or_default()
}

pub fn proto_to_rpc_block(bytes: &[u8]) -> Option<RpcBlock> {
    serde_json::from_slice(bytes).ok()
}
