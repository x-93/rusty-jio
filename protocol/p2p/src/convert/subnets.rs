use jio_consensus_core::subnets::SubnetworkId;

pub fn serialize_subnetwork_id(subnetwork_id: &SubnetworkId) -> Vec<u8> {
    subnetwork_id.as_bytes().to_vec()
}

pub fn deserialize_subnetwork_id(bytes: &[u8]) -> Option<SubnetworkId> {
    if bytes.len() == 20 {
        let mut arr = [0u8; 20];
        arr.copy_from_slice(bytes);
        Some(SubnetworkId::from_bytes(arr))
    } else {
        None
    }
}
