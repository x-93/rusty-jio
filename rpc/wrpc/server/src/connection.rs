use std::net::SocketAddr;

pub struct WrpcConnection {
    pub peer_addr: SocketAddr,
}

impl WrpcConnection {
    pub fn new(peer_addr: SocketAddr) -> Self {
        Self { peer_addr }
    }
}
