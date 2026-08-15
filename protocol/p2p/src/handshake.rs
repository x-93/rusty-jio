use crate::common::{PROTOCOL_VERSION, USER_AGENT};
use crate::convert::model::version::VersionMessage;
use std::net::SocketAddr;

pub struct Handshake;

impl Handshake {
    pub fn build_version_message(
        services: u64,
        address: Option<SocketAddr>,
        id: Vec<u8>,
    ) -> VersionMessage {
        VersionMessage {
            protocol_version: PROTOCOL_VERSION,
            services,
            timestamp: jio_core::time::unix_now(),
            address,
            id,
            user_agent: USER_AGENT.to_string(),
            disable_relay_tx: false,
            subnetwork_id: None,
        }
    }
}
