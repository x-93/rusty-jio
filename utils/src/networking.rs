//! Networking IP classification and routing validation.

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};

/// Returns true if an IP address is publicly routable on the global Internet.
pub fn is_routable(addr: &IpAddr) -> bool {
    match addr {
        IpAddr::V4(ip) => is_routable_v4(ip),
        IpAddr::V6(ip) => is_routable_v6(ip),
    }
}

pub fn is_routable_v4(ip: &Ipv4Addr) -> bool {
    let octets = ip.octets();
    // Loopback 127.0.0.0/8
    if octets[0] == 127 {
        return false;
    }
    // Private RFC1918: 10.0.0.0/8, 172.16.0.0/12, 192.168.0.0/16
    if octets[0] == 10 {
        return false;
    }
    if octets[0] == 172 && (16..=31).contains(&octets[1]) {
        return false;
    }
    if octets[0] == 192 && octets[1] == 168 {
        return false;
    }
    // Link-local: 169.254.0.0/16
    if octets[0] == 169 && octets[1] == 254 {
        return false;
    }
    // Broadcast / unspecified
    if ip.is_broadcast() || ip.is_unspecified() {
        return false;
    }
    true
}

pub fn is_routable_v6(ip: &Ipv6Addr) -> bool {
    if ip.is_loopback() || ip.is_unspecified() {
        return false;
    }
    // Unique local (fc00::/7) or link-local (fe80::/10)
    let segments = ip.segments();
    if (segments[0] & 0xfe00) == 0xfc00 || (segments[0] & 0xffc0) == 0xfe80 {
        return false;
    }
    true
}

pub fn canonical_socket_addr(addr: SocketAddr) -> SocketAddr {
    addr
}
