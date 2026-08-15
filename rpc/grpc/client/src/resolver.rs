use std::net::SocketAddr;

pub struct GrpcResolver {
    pub default_port: u16,
}

impl GrpcResolver {
    pub fn new(default_port: u16) -> Self {
        Self { default_port }
    }

    pub fn resolve(&self, target: &str) -> Option<SocketAddr> {
        target.parse().ok()
    }
}
