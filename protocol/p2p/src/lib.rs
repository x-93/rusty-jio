pub mod common;
pub mod convert;
pub mod core;
pub mod echo;
pub mod handshake;

pub use common::*;
pub use convert::*;
pub use core::*;
pub use echo::*;
pub use handshake::*;

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};

    #[test]
    fn test_handshake_version_message() {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 16111);
        let version = Handshake::build_version_message(1, Some(addr), vec![1, 2, 3]);
        assert_eq!(version.protocol_version, PROTOCOL_VERSION);
        assert_eq!(version.user_agent, USER_AGENT);
    }
}
