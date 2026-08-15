use std::net::SocketAddr;

pub struct GrpcAdaptor {
    pub listen_addr: SocketAddr,
}

impl GrpcAdaptor {
    pub fn new(listen_addr: SocketAddr) -> Self {
        Self { listen_addr }
    }
}
