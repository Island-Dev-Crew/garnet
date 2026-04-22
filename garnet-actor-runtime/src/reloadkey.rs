//! ReloadKey — Ed25519-signed hot-reload authorisation (v3.5 Security Layer 3).
//!
//! Closes the "unauthenticated hot-reload = RCE" threat class described in
//! `GARNET_v3_3_SECURITY_THREAT_MODEL.md` §2.2 and required by the master
//! plan Phase 3-SEC item #1 (ReloadKey, 12h budget).
//!
//! Every actor may carry a verifying key. When a reload is issued, the
//! caller supplies a signature over a canonical reload-request byte string.
//! The runtime verifies the signature against the actor's key BEFORE running
//! the migrator — a correctly-signed request is honored; an unsigned or
//! wrong-signed request is refused with `ReloadOutcome::SignatureInvalid`.
//!
//! ## Threat closed
//!
//! Without ReloadKey, a v3.5 actor reachable from the network (MVPs 7, 8)
//! could be hot-reloaded by any caller that knew the actor's address —
//! arbitrary code execution in the actor's address space. ReloadKey binds
//! reload authority to a cryptographic key; knowledge of the address is
//! no longer sufficient.
//!
//! ## Design
//!
//! Each actor, at spawn time, is optionally associated with an Ed25519
//! `VerifyingKey`. The associated `SigningKey` is held by whoever is
//! authorised to perform reloads (typically a separate deploy-agent
//! process on a different machine).
//!
//! The canonical signed bytes are:
//!
//!   reload-v1 || actor_addr_hash || target_version || allow_downgrade_byte
//!
//! where `actor_addr_hash` is a BLAKE3 hash of the address's channel identity
//! (stable across the address's lifetime but distinct per actor instance).
//!
//! ## Defense-in-depth
//!
//! Every reload carries a monotonically-increasing sequence number to
//! defeat replay attacks. The actor tracks the last-honored sequence; any
//! lower number is refused even with a valid signature.

use blake3::Hasher;
use ed25519_dalek::{Signature, SigningKey, Verifier, VerifyingKey, SECRET_KEY_LENGTH};
use rand::rngs::OsRng;

/// The canonical magic string prefix. Distinguishes reload signatures
/// from any other Ed25519 signature the same key may produce.
pub const RELOAD_SIGNATURE_MAGIC: &[u8] = b"garnet-reload-v1";

/// A reload authorisation request. The caller constructs this, signs the
/// `to_signing_bytes()`, and passes the signature to `Runtime::reload_signed`.
#[derive(Debug, Clone)]
pub struct ReloadAuth {
    pub actor_id: [u8; 32],
    pub target_version: u32,
    pub allow_downgrade: bool,
    pub sequence: u64,
}

impl ReloadAuth {
    /// The exact byte string the Ed25519 signer must sign. Callers MUST
    /// use this function rather than concatenating fields themselves —
    /// canonicalisation is load-bearing for cross-machine signing.
    pub fn to_signing_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(RELOAD_SIGNATURE_MAGIC.len() + 32 + 4 + 1 + 8);
        out.extend_from_slice(RELOAD_SIGNATURE_MAGIC);
        out.extend_from_slice(&self.actor_id);
        out.extend_from_slice(&self.target_version.to_le_bytes());
        out.push(self.allow_downgrade as u8);
        out.extend_from_slice(&self.sequence.to_le_bytes());
        out
    }

    /// Sign this auth with a secret key. Returns the signature bytes.
    pub fn sign(&self, signing_key: &SigningKey) -> Signature {
        let bytes = self.to_signing_bytes();
        use ed25519_dalek::Signer;
        signing_key.sign(&bytes)
    }

    /// Verify this auth's signature against a verifying key.
    pub fn verify(&self, vk: &VerifyingKey, sig: &Signature) -> bool {
        let bytes = self.to_signing_bytes();
        vk.verify(&bytes, sig).is_ok()
    }
}

/// Derive a stable 32-byte actor identity from a per-actor seed. The
/// seed is typically the BLAKE3 hash of the spawn timestamp + a
/// random nonce, captured at `spawn_keyed` time.
pub fn derive_actor_id(seed: &[u8]) -> [u8; 32] {
    let mut h = Hasher::new();
    h.update(b"garnet-actor-id-v1");
    h.update(seed);
    *h.finalize().as_bytes()
}

/// Generate a fresh (SigningKey, VerifyingKey) pair.
pub fn generate_keypair() -> (SigningKey, VerifyingKey) {
    let sk = SigningKey::generate(&mut OsRng);
    let vk = sk.verifying_key();
    (sk, vk)
}

/// Parse a 32-byte secret from hex, returning the SigningKey.
pub fn signing_key_from_hex(hex: &str) -> Result<SigningKey, String> {
    if hex.len() != SECRET_KEY_LENGTH * 2 {
        return Err(format!(
            "ed25519 secret key hex must be {} chars, got {}",
            SECRET_KEY_LENGTH * 2,
            hex.len()
        ));
    }
    let mut bytes = [0u8; SECRET_KEY_LENGTH];
    for i in 0..SECRET_KEY_LENGTH {
        let byte = u8::from_str_radix(&hex[i * 2..i * 2 + 2], 16)
            .map_err(|e| format!("invalid hex at byte {i}: {e}"))?;
        bytes[i] = byte;
    }
    Ok(SigningKey::from_bytes(&bytes))
}

/// Per-actor replay-defense state. Tracks the highest sequence number
/// honored by this actor.
#[derive(Debug, Default)]
pub struct ReloadReplayGuard {
    last_sequence: u64,
}

impl ReloadReplayGuard {
    /// Returns Ok(()) if the sequence is strictly greater than the last
    /// honored one; Err with the conflict details otherwise.
    pub fn check_and_record(&mut self, seq: u64) -> Result<(), (u64, u64)> {
        if seq <= self.last_sequence {
            return Err((self.last_sequence, seq));
        }
        self.last_sequence = seq;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sign_and_verify_roundtrip() {
        let (sk, vk) = generate_keypair();
        let auth = ReloadAuth {
            actor_id: [1u8; 32],
            target_version: 7,
            allow_downgrade: false,
            sequence: 100,
        };
        let sig = auth.sign(&sk);
        assert!(auth.verify(&vk, &sig));
    }

    #[test]
    fn wrong_key_rejects_signature() {
        let (sk1, _) = generate_keypair();
        let (_, vk2) = generate_keypair();
        let auth = ReloadAuth {
            actor_id: [1u8; 32],
            target_version: 7,
            allow_downgrade: false,
            sequence: 100,
        };
        let sig = auth.sign(&sk1);
        assert!(!auth.verify(&vk2, &sig));
    }

    #[test]
    fn modified_version_breaks_signature() {
        let (sk, vk) = generate_keypair();
        let auth = ReloadAuth {
            actor_id: [1u8; 32],
            target_version: 7,
            allow_downgrade: false,
            sequence: 100,
        };
        let sig = auth.sign(&sk);
        let tampered = ReloadAuth {
            target_version: 8, // changed
            ..auth
        };
        assert!(!tampered.verify(&vk, &sig));
    }

    #[test]
    fn modified_downgrade_flag_breaks_signature() {
        let (sk, vk) = generate_keypair();
        let auth = ReloadAuth {
            actor_id: [1u8; 32],
            target_version: 7,
            allow_downgrade: false,
            sequence: 100,
        };
        let sig = auth.sign(&sk);
        let tampered = ReloadAuth {
            allow_downgrade: true,
            ..auth
        };
        assert!(!tampered.verify(&vk, &sig));
    }

    #[test]
    fn modified_actor_id_breaks_signature() {
        let (sk, vk) = generate_keypair();
        let auth = ReloadAuth {
            actor_id: [1u8; 32],
            target_version: 7,
            allow_downgrade: false,
            sequence: 100,
        };
        let sig = auth.sign(&sk);
        let tampered = ReloadAuth {
            actor_id: [2u8; 32],
            ..auth
        };
        assert!(!tampered.verify(&vk, &sig));
    }

    #[test]
    fn modified_sequence_breaks_signature() {
        let (sk, vk) = generate_keypair();
        let auth = ReloadAuth {
            actor_id: [1u8; 32],
            target_version: 7,
            allow_downgrade: false,
            sequence: 100,
        };
        let sig = auth.sign(&sk);
        let tampered = ReloadAuth {
            sequence: 101,
            ..auth
        };
        assert!(!tampered.verify(&vk, &sig));
    }

    #[test]
    fn signing_bytes_are_canonical_and_stable() {
        let auth = ReloadAuth {
            actor_id: [7u8; 32],
            target_version: 42,
            allow_downgrade: true,
            sequence: 9999,
        };
        let a = auth.to_signing_bytes();
        let b = auth.to_signing_bytes();
        assert_eq!(a, b);
        // Check length: magic(17) + id(32) + ver(4) + downgrade(1) + seq(8) = 62
        assert_eq!(a.len(), RELOAD_SIGNATURE_MAGIC.len() + 32 + 4 + 1 + 8);
        // Magic prefix is preserved
        assert!(a.starts_with(RELOAD_SIGNATURE_MAGIC));
    }

    #[test]
    fn replay_guard_rejects_repeats() {
        let mut g = ReloadReplayGuard::default();
        assert!(g.check_and_record(1).is_ok());
        assert!(g.check_and_record(2).is_ok());
        assert!(g.check_and_record(2).is_err()); // same — reject
        assert!(g.check_and_record(1).is_err()); // lower — reject
        assert!(g.check_and_record(100).is_ok());
        assert!(g.check_and_record(50).is_err()); // backward — reject
    }

    #[test]
    fn derive_actor_id_is_stable() {
        let a = derive_actor_id(b"seed-one");
        let b = derive_actor_id(b"seed-one");
        let c = derive_actor_id(b"seed-two");
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn signing_key_from_hex_roundtrip() {
        let (sk, _) = generate_keypair();
        let bytes = sk.to_bytes();
        let hex: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
        let sk2 = signing_key_from_hex(&hex).unwrap();
        assert_eq!(sk.to_bytes(), sk2.to_bytes());
    }

    #[test]
    fn signing_key_from_malformed_hex_rejected() {
        assert!(signing_key_from_hex("not-hex").is_err());
        assert!(signing_key_from_hex("ff").is_err()); // too short
    }
}
