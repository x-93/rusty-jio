use std::net::SocketAddr;

pub struct PortMappingExtender {
    default_port: u16,
}

impl PortMappingExtender {
    pub fn new(default_port: u16) -> Self {
        Self { default_port }
    }

    pub fn extend_port(&self, mut addr: SocketAddr) -> SocketAddr {
        if addr.port() == 0 {
            addr.set_port(self.default_port);
        }
        addr
    }
}
