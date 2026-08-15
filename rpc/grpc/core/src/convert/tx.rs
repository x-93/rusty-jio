use jio_rpc_core::model::tx::RpcTransaction;

pub fn rpc_tx_to_proto(tx: RpcTransaction) -> Vec<u8> {
    serde_json::to_vec(&tx).unwrap_or_default()
}

pub fn proto_to_rpc_tx(bytes: &[u8]) -> Option<RpcTransaction> {
    serde_json::from_slice(bytes).ok()
}
