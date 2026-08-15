use std::net::SocketAddr;

pub struct WrpcServer {
    pub listen_addr: SocketAddr,
}

impl WrpcServer {
    pub fn new(listen_addr: SocketAddr) -> Self {
        Self { listen_addr }
    }
}
