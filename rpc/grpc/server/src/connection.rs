use std::net::SocketAddr;

pub struct GrpcConnection {
    pub peer_addr: SocketAddr,
}

impl GrpcConnection {
    pub fn new(peer_addr: SocketAddr) -> Self {
        Self { peer_addr }
    }
}
