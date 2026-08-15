use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VersionMessage {
    pub protocol_version: u32,
    pub services: u64,
    pub timestamp: u64,
    pub address: Option<SocketAddr>,
    pub id: Vec<u8>,
    pub user_agent: String,
    pub disable_relay_tx: bool,
    pub subnetwork_id: Option<jio_consensus_core::subnets::SubnetworkId>,
}
