//! StateCert — schema-fingerprinted hot-reload state extraction.
//! v3.3 Security Layer 1 (hardening pattern #2).
//!
//! ## The threat
//!
//! v3.3 Fix #2 added `Actor::extract_state()` returning
//! `Option<Box<dyn Any + Send>>`. A migrator `.downcast()`s and
//! silently panics on type mismatch. With hot-reload over any external
//! channel (CLI today, RPC tomorrow), a malicious migrator:
//!
//! 1. **Crashes** the actor process via a deliberate mismatched downcast.
//! 2. **Coerces state** through a type with a compatible layout (same
//!    size/align, different semantics) — `Rust`'s `Any` downcast is
//!    name-based via `TypeId`, but `TypeId` is not stable across rustc
//!    versions and gives no information to the operator inspecting a
//!    failed reload.
//!
//! ## The fix
//!
//! Every state payload carries a **32-byte BLAKE3 fingerprint** derived
//! from the Rust type's name + size + alignment. `TaggedState::downcast<T>`
//! computes the expected fingerprint and refuses the downcast on mismatch,
//! returning a structured `FingerprintMismatch` error instead of panicking.
//!
//! ## Why not just TypeId?
//!
//! `std::any::TypeId` is:
//! - Not stable across compiler versions (so two compilers disagree).
//! - Only 64 bits (collision-resistance is a stretch).
//! - Opaque (cannot be serialized, printed, or compared outside the
//!   running binary).
//!
//! BLAKE3 of `type_name() || size || align` gives us a stable,
//! inspectable, 256-bit identifier that survives compiler bumps and
//! cross-binary hot-reload.

use std::any::Any;

/// 32-byte BLAKE3 fingerprint of a Rust type's identity. Used by
/// `TaggedState::downcast` to refuse type-confused reloads.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypeFingerprint([u8; 32]);

impl TypeFingerprint {
    /// Compute the fingerprint for type `T`. Derives from
    /// `type_name::<T>() || size_of::<T>() || align_of::<T>()` —
    /// captures both identity and ABI.
    pub fn of<T: 'static>() -> Self {
        let mut hasher = blake3::Hasher::new();
        hasher.update(std::any::type_name::<T>().as_bytes());
        hasher.update(&std::mem::size_of::<T>().to_le_bytes());
        hasher.update(&std::mem::align_of::<T>().to_le_bytes());
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(hasher.finalize().as_bytes());
        Self(bytes)
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Hex representation for logs / manifests.
    pub fn to_hex(&self) -> String {
        let mut out = String::with_capacity(64);
        for byte in &self.0 {
            out.push_str(&format!("{byte:02x}"));
        }
        out
    }
}

/// State extracted from an actor for hot-reload, with a
/// schema fingerprint so migrators can verify type compatibility
/// before the downcast.
///
/// Produced via `TaggedState::new(concrete_state)`; consumed via
/// `tagged.downcast::<T>()` which returns `FingerprintMismatch` if
/// `T`'s fingerprint doesn't match.
pub struct TaggedState {
    pub fingerprint: TypeFingerprint,
    pub state: Box<dyn Any + Send>,
}

impl TaggedState {
    /// Build a `TaggedState` from a concrete value, capturing its
    /// fingerprint. The common construction pattern inside an
    /// `Actor::extract_state` implementation.
    pub fn new<T: Any + Send + 'static>(state: T) -> Self {
        Self {
            fingerprint: TypeFingerprint::of::<T>(),
            state: Box::new(state),
        }
    }

    /// Fingerprint-verified downcast. Returns `Err(FingerprintMismatch)`
    /// if the caller's expected type `T` does not match the recorded
    /// fingerprint — never panics. On match, returns the `Box<T>`.
    pub fn downcast<T: Any + Send + 'static>(self) -> Result<Box<T>, FingerprintMismatch> {
        let expected = TypeFingerprint::of::<T>();
        if self.fingerprint != expected {
            return Err(FingerprintMismatch {
                expected,
                actual: self.fingerprint,
            });
        }
        // Fingerprints matched; the underlying TypeId should agree.
        // If it somehow doesn't (a would-be compiler bug), surface it
        // as a FingerprintMismatch rather than panicking.
        self.state.downcast::<T>().map_err(|_| FingerprintMismatch {
            expected,
            actual: self.fingerprint,
        })
    }

    /// Inspect fingerprint without consuming.
    pub fn fingerprint(&self) -> TypeFingerprint {
        self.fingerprint
    }
}

impl std::fmt::Debug for TaggedState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TaggedState")
            .field("fingerprint", &self.fingerprint.to_hex())
            .finish_non_exhaustive()
    }
}

/// Returned by `TaggedState::downcast` when the caller's type
/// fingerprint does not match the recorded fingerprint. Structured
/// error — no panic, no silent data corruption.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FingerprintMismatch {
    pub expected: TypeFingerprint,
    pub actual: TypeFingerprint,
}

impl std::fmt::Display for FingerprintMismatch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "type fingerprint mismatch on hot-reload state extraction (expected {}, got {})",
            self.expected.to_hex(),
            self.actual.to_hex()
        )
    }
}

impl std::error::Error for FingerprintMismatch {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_type_produces_same_fingerprint() {
        assert_eq!(TypeFingerprint::of::<i64>(), TypeFingerprint::of::<i64>());
        assert_eq!(
            TypeFingerprint::of::<(i64, String)>(),
            TypeFingerprint::of::<(i64, String)>()
        );
    }

    #[test]
    fn different_types_produce_different_fingerprints() {
        assert_ne!(TypeFingerprint::of::<i64>(), TypeFingerprint::of::<u64>());
        assert_ne!(TypeFingerprint::of::<i32>(), TypeFingerprint::of::<i64>());
        assert_ne!(
            TypeFingerprint::of::<String>(),
            TypeFingerprint::of::<&'static str>()
        );
    }

    #[test]
    fn tagged_state_roundtrips_correct_type() {
        let tagged = TaggedState::new(42i64);
        let boxed = tagged.downcast::<i64>().expect("correct downcast");
        assert_eq!(*boxed, 42);
    }

    #[test]
    fn tagged_state_rejects_wrong_type() {
        let tagged = TaggedState::new(42i64);
        let err = tagged.downcast::<u64>().expect_err("must reject u64");
        assert_ne!(err.expected, err.actual);
    }

    #[test]
    fn tagged_state_rejects_layout_compatible_but_semantically_distinct() {
        // `i64` and `u64` have the same size and alignment but different
        // names — the name goes into the fingerprint so they don't
        // accidentally alias.
        let tagged = TaggedState::new(42i64);
        assert!(tagged.downcast::<u64>().is_err());
    }

    #[test]
    fn fingerprint_hex_is_64_chars() {
        let fp = TypeFingerprint::of::<i64>();
        assert_eq!(fp.to_hex().len(), 64);
        assert!(fp.to_hex().chars().all(|c| c.is_ascii_hexdigit()));
    }
}
