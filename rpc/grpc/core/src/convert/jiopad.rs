use crate::protowire::JioResponse;

pub fn to_response_bytes(resp: &JioResponse) -> Vec<u8> {
    serde_json::to_vec(resp).unwrap_or_default()
}
