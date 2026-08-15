use jio_rpc_core::model::message::*;

pub struct RpcConverter;

impl RpcConverter {
    pub fn build_info(mempool_size: u64, is_synced: bool) -> GetInfoResponse {
        GetInfoResponse {
            p2p_id: "jio-node-1".to_string(),
            mempool_size,
            server_version: "0.1.0".to_string(),
            is_utxo_indexed: true,
            is_synced,
            has_notify_command: true,
        }
    }
}
