use std::net::SocketAddr;

pub fn serialize_net_address(addr: &SocketAddr) -> String {
    addr.to_string()
}

pub fn deserialize_net_address(s: &str) -> Option<SocketAddr> {
    s.parse().ok()
}
