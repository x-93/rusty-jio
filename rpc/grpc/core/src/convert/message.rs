use jio_rpc_core::model::message::*;

pub fn rpc_info_to_proto(info: GetInfoResponse) -> Vec<u8> {
    serde_json::to_vec(&info).unwrap_or_default()
}
