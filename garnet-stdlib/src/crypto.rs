//! Cryptographic hashes + HMAC (no caps — pure compute).
//!
//! All outputs are 32 bytes (256 bits). Suitable for content addressing,
//! integrity checks, and the v3.3 CacheHMAC / ManifestSig pipelines.

/// BLAKE3 hash of `data`. Returns 32 bytes.
pub fn blake3_hash(data: &[u8]) -> [u8; 32] {
    *blake3::hash(data).as_bytes()
}

/// SHA-256 hash of `data`. Returns 32 bytes.
pub fn sha256_hash(data: &[u8]) -> [u8; 32] {
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    h.update(data);
    let out = h.finalize();
    let mut arr = [0u8; 32];
    arr.copy_from_slice(&out);
    arr
}

/// HMAC-SHA-256 of `msg` with `key`. Returns 32 bytes.
pub fn hmac_sha256(key: &[u8], msg: &[u8]) -> [u8; 32] {
    use hmac::{Hmac, Mac};
    type HmacSha256 = Hmac<sha2::Sha256>;
    let mut mac = HmacSha256::new_from_slice(key)
        .expect("HMAC-SHA-256 accepts any key length");
    mac.update(msg);
    let out = mac.finalize().into_bytes();
    let mut arr = [0u8; 32];
    arr.copy_from_slice(&out);
    arr
}

/// BLAKE3 keyed hash (cryptographically equivalent to HMAC-BLAKE3). Faster
/// than HMAC-SHA-256 when the BLAKE3 cost amortises across many messages
/// signed with the same key.
///
/// Used by v3.3 CacheHMAC for per-machine episode signing.
pub fn blake3_keyed(key: &[u8; 32], msg: &[u8]) -> [u8; 32] {
    *blake3::keyed_hash(key, msg).as_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blake3_deterministic() {
        let a = blake3_hash(b"garnet");
        let b = blake3_hash(b"garnet");
        assert_eq!(a, b);
    }

    #[test]
    fn blake3_distinct_inputs_produce_distinct_outputs() {
        let a = blake3_hash(b"garnet");
        let b = blake3_hash(b"garnet!");
        assert_ne!(a, b);
    }

    #[test]
    fn sha256_matches_published_empty_vector() {
        // SHA-256("") = e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
        let h = sha256_hash(b"");
        let hex: String = h.iter().map(|b| format!("{b:02x}")).collect();
        assert_eq!(
            hex,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn hmac_deterministic_same_key() {
        let a = hmac_sha256(b"key", b"msg");
        let b = hmac_sha256(b"key", b"msg");
        assert_eq!(a, b);
    }

    #[test]
    fn hmac_key_sensitive() {
        let a = hmac_sha256(b"key1", b"msg");
        let b = hmac_sha256(b"key2", b"msg");
        assert_ne!(a, b);
    }

    #[test]
    fn hmac_msg_sensitive() {
        let a = hmac_sha256(b"key", b"msg1");
        let b = hmac_sha256(b"key", b"msg2");
        assert_ne!(a, b);
    }

    #[test]
    fn blake3_keyed_distinct_keys_distinct_outputs() {
        let k1 = [1u8; 32];
        let k2 = [2u8; 32];
        assert_ne!(blake3_keyed(&k1, b"m"), blake3_keyed(&k2, b"m"));
    }
}
