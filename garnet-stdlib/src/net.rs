//! Networking primitives (cap: `net`) — gated on CapCaps + NetDefaults.
//!
//! The CapCaps gate is enforced at the source layer (Mini-Spec v1.0 §16
//! / Security V2 §1). The NetDefaults gate (Security V2 §2) is enforced
//! INSIDE this module: every connect call walks the destination address
//! through `is_allowed()` BEFORE reaching `std::net`. DNS rebinding is
//! defended by re-validating the resolved peer at connect time.

use crate::StdError;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, TcpStream, ToSocketAddrs, UdpSocket};
use std::time::Duration;

/// Default read/idle timeout for new TCP connections (Security V2 §2.5).
const DEFAULT_TIMEOUT_MS: u64 = 30_000;

/// Maximum response size as a multiple of the request size for UDP
/// amplification defense (Security V2 §2.4).
const UDP_AMP_MAX_RATIO: usize = 3;

/// NetDefaults policy. Default = strict (deny RFC1918 + loopback +
/// link-local + cloud metadata). `permit_internal = true` lifts the
/// strict denial — corresponding source-layer requirement is
/// `@caps(net_internal)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct NetPolicy {
    pub permit_internal: bool,
}

/// Returns true iff `ip` may be connected to under `policy`.
pub fn is_allowed(ip: &IpAddr, policy: NetPolicy) -> bool {
    if policy.permit_internal {
        return !is_unconditionally_denied(ip);
    }
    !is_internal_or_unconditional(ip)
}

/// IPs that are denied even with `net_internal` (broadcast, multicast,
/// reserved, unspecified). Connecting to these never makes sense.
fn is_unconditionally_denied(ip: &IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            v4.is_unspecified()              // 0.0.0.0
                || v4.is_broadcast()         // 255.255.255.255
                || v4.is_multicast()         // 224/4
                || v4.is_documentation()     // 192.0.2/24, 198.51.100/24, 203.0.113/24
                || is_v4_reserved(v4)
        }
        IpAddr::V6(v6) => v6.is_unspecified() || v6.is_multicast() || is_v6_documentation(v6),
    }
}

/// IPs that are denied without `net_internal` (private + loopback +
/// link-local + cloud-metadata).
fn is_internal_or_unconditional(ip: &IpAddr) -> bool {
    if is_unconditionally_denied(ip) {
        return true;
    }
    match ip {
        IpAddr::V4(v4) => {
            v4.is_loopback()
                || v4.is_private()
                || v4.is_link_local()        // 169.254/16 (covers AWS metadata)
                || is_v4_cgnat(v4)           // 100.64/10
                || is_v4_benchmarking(v4) // 198.18/15
        }
        IpAddr::V6(v6) => {
            v6.is_loopback()
                || is_v6_unique_local(v6)    // fc00::/7
                || is_v6_link_local(v6) // fe80::/10
        }
    }
}

fn is_v4_reserved(v4: &Ipv4Addr) -> bool {
    let o = v4.octets();
    // 0.0.0.0/8 (current network, except 0.0.0.0 itself which is unspec)
    if o[0] == 0 && (o[1] != 0 || o[2] != 0 || o[3] != 0) {
        return true;
    }
    // 192.0.0.0/24 reserved
    if o[0] == 192 && o[1] == 0 && o[2] == 0 {
        return true;
    }
    // 240.0.0.0/4 reserved (excluding 255.255.255.255 which is broadcast)
    if o[0] >= 240 && !v4.is_broadcast() {
        return true;
    }
    false
}

fn is_v4_cgnat(v4: &Ipv4Addr) -> bool {
    // 100.64.0.0/10 = 100.64.0.0 .. 100.127.255.255
    let o = v4.octets();
    o[0] == 100 && (64..=127).contains(&o[1])
}

fn is_v4_benchmarking(v4: &Ipv4Addr) -> bool {
    let o = v4.octets();
    o[0] == 198 && (o[1] == 18 || o[1] == 19)
}

fn is_v6_unique_local(v6: &Ipv6Addr) -> bool {
    // fc00::/7
    (v6.octets()[0] & 0xfe) == 0xfc
}

fn is_v6_link_local(v6: &Ipv6Addr) -> bool {
    // fe80::/10
    let octs = v6.octets();
    octs[0] == 0xfe && (octs[1] & 0xc0) == 0x80
}

fn is_v6_documentation(v6: &Ipv6Addr) -> bool {
    // 2001:db8::/32
    let s = v6.segments();
    s[0] == 0x2001 && s[1] == 0x0db8
}

/// Open an outbound TCP connection with the given policy.
///
/// Algorithm (per Security V2 §2.3):
/// 1. Resolve `host:port` to a list of candidate addresses.
/// 2. For each candidate, check `is_allowed`; skip if denied.
/// 3. Connect to the first allowed candidate.
/// 4. Re-check the actual peer address after connect (DNS rebinding defense).
pub fn tcp_connect(host: &str, port: u16, policy: NetPolicy) -> Result<TcpStream, StdError> {
    let addrs = (host, port)
        .to_socket_addrs()
        .map_err(|e| StdError::Io(format!("resolve {host}:{port}: {e}")))?;
    let mut last_err: Option<StdError> = None;
    for addr in addrs {
        if !is_allowed(&addr.ip(), policy) {
            last_err = Some(StdError::NetDenied(format!(
                "address {} denied by NetDefaults",
                addr.ip()
            )));
            continue;
        }
        let stream =
            match TcpStream::connect_timeout(&addr, Duration::from_millis(DEFAULT_TIMEOUT_MS)) {
                Ok(s) => s,
                Err(e) => {
                    last_err = Some(StdError::Io(format!("connect {addr}: {e}")));
                    continue;
                }
            };
        // Defense in depth: re-check the actual peer after connect.
        let peer = stream
            .peer_addr()
            .map_err(|e| StdError::Io(format!("peer_addr: {e}")))?;
        if !is_allowed(&peer.ip(), policy) {
            return Err(StdError::NetDenied(format!(
                "peer {} differed from resolved {} (DNS rebinding?) and is denied",
                peer.ip(),
                addr.ip()
            )));
        }
        // Apply default read + write timeouts (slowloris defense).
        let _ = stream.set_read_timeout(Some(Duration::from_millis(DEFAULT_TIMEOUT_MS)));
        let _ = stream.set_write_timeout(Some(Duration::from_millis(DEFAULT_TIMEOUT_MS)));
        return Ok(stream);
    }
    Err(last_err
        .unwrap_or_else(|| StdError::Io(format!("no addresses resolved for {host}:{port}"))))
}

/// Send a UDP response, enforcing the amplification cap (Security V2 §2.4).
pub fn udp_send_response(
    sock: &UdpSocket,
    peer: SocketAddr,
    request_size: usize,
    response: &[u8],
) -> Result<usize, StdError> {
    if response.len() > request_size.saturating_mul(UDP_AMP_MAX_RATIO) {
        return Err(StdError::NetDenied(format!(
            "udp response {} bytes exceeds {}× request size {}",
            response.len(),
            UDP_AMP_MAX_RATIO,
            request_size
        )));
    }
    sock.send_to(response, peer).map_err(StdError::from)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    fn ip(s: &str) -> IpAddr {
        IpAddr::from_str(s).unwrap()
    }

    // ─── Strict policy denies all internal/reserved ─────────────────────
    #[test]
    fn strict_policy_denies_loopback_v4() {
        assert!(!is_allowed(&ip("127.0.0.1"), NetPolicy::default()));
    }
    #[test]
    fn strict_policy_denies_rfc1918_10() {
        assert!(!is_allowed(&ip("10.0.0.1"), NetPolicy::default()));
    }
    #[test]
    fn strict_policy_denies_rfc1918_192_168() {
        assert!(!is_allowed(&ip("192.168.1.1"), NetPolicy::default()));
    }
    #[test]
    fn strict_policy_denies_rfc1918_172_16() {
        assert!(!is_allowed(&ip("172.16.0.1"), NetPolicy::default()));
    }
    #[test]
    fn strict_policy_denies_link_local_aws_metadata() {
        // 169.254.169.254 is the AWS / GCP / Azure metadata service.
        assert!(!is_allowed(&ip("169.254.169.254"), NetPolicy::default()));
    }
    #[test]
    fn strict_policy_denies_cgnat() {
        assert!(!is_allowed(&ip("100.64.0.1"), NetPolicy::default()));
    }
    #[test]
    fn strict_policy_denies_v6_loopback() {
        assert!(!is_allowed(&ip("::1"), NetPolicy::default()));
    }
    #[test]
    fn strict_policy_denies_v6_link_local() {
        assert!(!is_allowed(&ip("fe80::1"), NetPolicy::default()));
    }
    #[test]
    fn strict_policy_denies_v6_unique_local() {
        assert!(!is_allowed(&ip("fc00::1"), NetPolicy::default()));
    }

    // ─── Strict policy permits public IPs ───────────────────────────────
    #[test]
    fn strict_policy_permits_public_v4() {
        assert!(is_allowed(&ip("8.8.8.8"), NetPolicy::default()));
        assert!(is_allowed(&ip("1.1.1.1"), NetPolicy::default()));
    }
    #[test]
    fn strict_policy_permits_public_v6() {
        assert!(is_allowed(
            &ip("2606:4700:4700::1111"),
            NetPolicy::default()
        ));
    }

    // ─── net_internal lifts ONLY the internal denials ───────────────────
    #[test]
    fn permit_internal_allows_loopback() {
        let p = NetPolicy {
            permit_internal: true,
        };
        assert!(is_allowed(&ip("127.0.0.1"), p));
        assert!(is_allowed(&ip("10.0.0.1"), p));
        assert!(is_allowed(&ip("192.168.1.1"), p));
    }
    #[test]
    fn permit_internal_still_denies_unspecified() {
        let p = NetPolicy {
            permit_internal: true,
        };
        assert!(!is_allowed(&ip("0.0.0.0"), p));
        assert!(!is_allowed(&ip("255.255.255.255"), p));
    }
    #[test]
    fn permit_internal_still_denies_multicast() {
        let p = NetPolicy {
            permit_internal: true,
        };
        assert!(!is_allowed(&ip("224.0.0.1"), p));
    }
    #[test]
    fn permit_internal_still_denies_v4_documentation() {
        let p = NetPolicy {
            permit_internal: true,
        };
        assert!(!is_allowed(&ip("192.0.2.1"), p));
        assert!(!is_allowed(&ip("198.51.100.1"), p));
        assert!(!is_allowed(&ip("203.0.113.1"), p));
    }

    // ─── UDP amplification cap ──────────────────────────────────────────
    #[test]
    fn udp_amp_cap_rejects_oversize_response() {
        // Bind ephemeral, attempt to send oversized to ourselves.
        let sock = UdpSocket::bind("127.0.0.1:0").unwrap();
        let peer = sock.local_addr().unwrap();
        let request_size = 10;
        let oversize = vec![0u8; request_size * UDP_AMP_MAX_RATIO + 1];
        match udp_send_response(&sock, peer, request_size, &oversize) {
            Err(StdError::NetDenied(_)) => {}
            other => panic!("expected NetDenied, got {other:?}"),
        }
    }

    #[test]
    fn udp_amp_cap_accepts_within_3x() {
        let sock = UdpSocket::bind("127.0.0.1:0").unwrap();
        let peer = sock.local_addr().unwrap();
        let request_size = 10;
        let within = vec![0u8; request_size * UDP_AMP_MAX_RATIO];
        // This may fail (e.g., no recv on the other side) but it MUST NOT
        // be NetDenied — that's what we're checking.
        let r = udp_send_response(&sock, peer, request_size, &within);
        if let Err(StdError::NetDenied(msg)) = r {
            panic!("expected accept, got NetDenied: {msg}");
        }
    }

    // ─── tcp_connect against a denied target without net_internal ───────
    #[test]
    fn tcp_connect_to_loopback_strict_returns_netdenied() {
        // 127.0.0.1:1 — strict policy should refuse before attempting.
        match tcp_connect("127.0.0.1", 1, NetPolicy::default()) {
            Err(StdError::NetDenied(_)) => {}
            other => panic!("expected NetDenied, got {other:?}"),
        }
    }
}
