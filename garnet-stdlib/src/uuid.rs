//! `std::uuid` — UUID generation (Layer 1).
//!
//! - `new_v4` — random (cap: `time`, to acknowledge clock-seeded entropy).
//! - `new_v7` — time-ordered: 48-bit unix-ms prefix + randomness (cap: `time`).
//! - `new_v5` — name-based, SHA-1 of (namespace, name); deterministic (no cap).
//!
//! Hand-rolled bit-tagging over `rand` (entropy) and `sha1` (v5), both in the
//! RustCrypto / rand families already vetted in the workspace. RFC 4122
//! layout. `@stability(experimental)`.

use std::time::{SystemTime, UNIX_EPOCH};

/// Format 16 bytes as a canonical lowercase hyphenated UUID string.
fn format_uuid(b: &[u8; 16]) -> String {
    let mut s = String::with_capacity(36);
    for (i, byte) in b.iter().enumerate() {
        if matches!(i, 4 | 6 | 8 | 10) {
            s.push('-');
        }
        s.push_str(&format!("{byte:02x}"));
    }
    s
}

/// Set the 4-bit version (high nibble of byte 6) and the 2-bit RFC 4122
/// variant (high bits of byte 8).
fn tag(b: &mut [u8; 16], version: u8) {
    b[6] = (b[6] & 0x0f) | (version << 4);
    b[8] = (b[8] & 0x3f) | 0x80;
}

fn unix_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// Random UUIDv4.
pub fn new_v4() -> String {
    let mut b: [u8; 16] = rand::random();
    tag(&mut b, 4);
    format_uuid(&b)
}

/// Time-ordered UUIDv7: 48-bit big-endian unix-ms prefix, then randomness.
pub fn new_v7() -> String {
    let mut b: [u8; 16] = rand::random();
    let ms = unix_millis();
    // High 48 bits = timestamp, big-endian, in bytes 0..6.
    b[0] = (ms >> 40) as u8;
    b[1] = (ms >> 32) as u8;
    b[2] = (ms >> 24) as u8;
    b[3] = (ms >> 16) as u8;
    b[4] = (ms >> 8) as u8;
    b[5] = ms as u8;
    tag(&mut b, 7);
    format_uuid(&b)
}

/// Name-based UUIDv5: SHA-1 of (namespace ‖ name), first 16 bytes tagged.
/// Deterministic — equal inputs always yield the same UUID.
pub fn new_v5(namespace: &[u8; 16], name: &str) -> String {
    use sha1::{Digest, Sha1};
    let mut h = Sha1::new();
    h.update(namespace);
    h.update(name.as_bytes());
    let digest = h.finalize(); // 20 bytes
    let mut b = [0u8; 16];
    b.copy_from_slice(&digest[..16]);
    tag(&mut b, 5);
    format_uuid(&b)
}

#[cfg(test)]
mod tests {
    use super::*;

    // RFC 4122 DNS namespace, for v5 tests.
    const NS_DNS: [u8; 16] = [
        0x6b, 0xa7, 0xb8, 0x10, 0x9d, 0xad, 0x11, 0xd1, 0x80, 0xb4, 0x00, 0xc0, 0x4f, 0xd4, 0x30,
        0xc8,
    ];

    fn well_formed(u: &str) -> bool {
        let bytes = u.as_bytes();
        u.len() == 36
            && bytes[8] == b'-'
            && bytes[13] == b'-'
            && bytes[18] == b'-'
            && bytes[23] == b'-'
            && u.chars().all(|c| c.is_ascii_hexdigit() || c == '-')
    }

    /// Version nibble is the first char of the 3rd group (index 14).
    fn version_char(u: &str) -> char {
        u.chars().nth(14).unwrap()
    }
    /// Variant char is the first char of the 4th group (index 19); RFC 4122
    /// variant means it is one of 8,9,a,b.
    fn variant_char(u: &str) -> char {
        u.chars().nth(19).unwrap()
    }

    #[test]
    fn v4_is_well_formed_versioned_and_unique() {
        let a = new_v4();
        let b = new_v4();
        assert!(well_formed(&a), "malformed: {a}");
        assert_eq!(version_char(&a), '4');
        assert!(matches!(variant_char(&a), '8' | '9' | 'a' | 'b'));
        assert_ne!(a, b, "two v4 uuids should (essentially always) differ");
    }

    #[test]
    fn v7_is_well_formed_and_versioned() {
        let u = new_v7();
        assert!(well_formed(&u), "malformed: {u}");
        assert_eq!(version_char(&u), '7');
        assert!(matches!(variant_char(&u), '8' | '9' | 'a' | 'b'));
    }

    #[test]
    fn v7_is_time_ordered() {
        let first = new_v7();
        std::thread::sleep(std::time::Duration::from_millis(2));
        let second = new_v7();
        // The 48-bit timestamp prefix (first 12 hex chars, skipping the dash
        // at index 8) is non-decreasing over time.
        let prefix = |u: &str| {
            let h: String = u.chars().filter(|c| *c != '-').take(12).collect();
            u64::from_str_radix(&h, 16).unwrap()
        };
        assert!(prefix(&second) >= prefix(&first));
    }

    #[test]
    fn v5_is_deterministic() {
        let a = new_v5(&NS_DNS, "garnet-lang.org");
        let b = new_v5(&NS_DNS, "garnet-lang.org");
        assert_eq!(a, b, "v5 must be deterministic for equal inputs");
        assert!(well_formed(&a));
        assert_eq!(version_char(&a), '5');
        assert!(matches!(variant_char(&a), '8' | '9' | 'a' | 'b'));
    }

    #[test]
    fn v5_differs_on_different_names() {
        assert_ne!(new_v5(&NS_DNS, "a"), new_v5(&NS_DNS, "b"));
    }
}
