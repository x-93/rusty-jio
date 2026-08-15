use crate::convert::model::trusted::TrustedDataPackage;

pub fn serialize_trusted_data(data: &TrustedDataPackage) -> Vec<u8> {
    serde_json::to_vec(data).unwrap_or_default()
}

pub fn deserialize_trusted_data(bytes: &[u8]) -> Option<TrustedDataPackage> {
    serde_json::from_slice(bytes).ok()
}
