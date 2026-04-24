//! Primitive registry — maps Garnet-surface names to
//! (required_capability, invocation-metadata) tuples.
//!
//! The interpreter calls `all_prims()` at startup to populate its
//! prelude. The CapCaps checker (Rung 4 / v3.4 Security Layer 2)
//! consults the `RequiredCaps` tag on each primitive at every call
//! site to verify the calling function's `@caps(...)` annotation
//! covers the required capability.

use std::collections::BTreeMap;

/// Capabilities a primitive requires at the source layer. An empty set
/// means "pure computation, no OS authority required."
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequiredCaps(pub Vec<&'static str>);

impl RequiredCaps {
    pub const fn none() -> Self {
        Self(Vec::new())
    }
    pub fn fs() -> Self {
        Self(vec!["fs"])
    }
    pub fn net() -> Self {
        Self(vec!["net"])
    }
    pub fn time() -> Self {
        Self(vec!["time"])
    }
    pub fn proc() -> Self {
        Self(vec!["proc"])
    }
    pub fn contains(&self, cap: &str) -> bool {
        self.0.contains(&cap)
    }
}

/// One primitive entry in the registry. The `arity` and `mode` fields
/// are metadata the interpreter uses to build the prelude; the actual
/// dispatch happens through named lookup + reflection on the module
/// hierarchy (e.g., `fs::read_file` vs. `time::now_ms`).
#[derive(Debug, Clone)]
pub struct PrimMeta {
    pub module: &'static str,
    pub name: &'static str,
    pub arity: usize,
    pub required_caps: RequiredCaps,
    pub doc: &'static str,
}

/// The full primitive table. Produced by `all_prims()`. The interpreter
/// walks this at startup; the CapCaps checker consults it at every
/// primitive-call site.
pub type PrimTable = BTreeMap<String, PrimMeta>;

/// Produce the v3.4 P0 stdlib primitive table.
///
/// Canonical primitive list per Mini-Spec v1.0 §11.2 + Security V2 spec
/// §1.6. Each primitive's required capability is defensible per the
/// threat-model rationale documented in v3.4 Security V2 spec §2 (net),
/// §3 (fs via BoundedMail's actor-discipline companion), etc.
pub fn all_prims() -> PrimTable {
    let mut t = BTreeMap::new();
    for p in static_prims() {
        t.insert(format!("{}::{}", p.module, p.name), p.clone());
    }
    t
}

fn static_prims() -> &'static [PrimMeta] {
    // Built lazily at first call for test stability; in the binary this
    // becomes a const-initialised table.
    PRIMS.get_or_init(build_prims)
}

use std::sync::OnceLock;
static PRIMS: OnceLock<Vec<PrimMeta>> = OnceLock::new();

fn build_prims() -> Vec<PrimMeta> {
    vec![
        // ── time (cap: time) ──
        PrimMeta {
            module: "time",
            name: "now_ms",
            arity: 0,
            required_caps: RequiredCaps::time(),
            doc: "Monotonic clock in milliseconds since process start.",
        },
        PrimMeta {
            module: "time",
            name: "wall_clock_ms",
            arity: 0,
            required_caps: RequiredCaps::time(),
            doc: "Wall clock in milliseconds since UNIX epoch.",
        },
        PrimMeta {
            module: "time",
            name: "sleep",
            arity: 1,
            required_caps: RequiredCaps::time(),
            doc: "Sleep the current thread for N milliseconds.",
        },
        // ── strings (no caps) ──
        PrimMeta {
            module: "str",
            name: "split",
            arity: 2,
            required_caps: RequiredCaps::none(),
            doc: "Split a string on a delimiter; returns an Array<String>.",
        },
        PrimMeta {
            module: "str",
            name: "replace",
            arity: 3,
            required_caps: RequiredCaps::none(),
            doc: "Replace all occurrences of `old` with `new`.",
        },
        PrimMeta {
            module: "str",
            name: "to_lower",
            arity: 1,
            required_caps: RequiredCaps::none(),
            doc: "Lowercase a string (Unicode-aware).",
        },
        PrimMeta {
            module: "str",
            name: "to_upper",
            arity: 1,
            required_caps: RequiredCaps::none(),
            doc: "Uppercase a string (Unicode-aware).",
        },
        PrimMeta {
            module: "str",
            name: "trim",
            arity: 1,
            required_caps: RequiredCaps::none(),
            doc: "Trim whitespace from both ends.",
        },
        PrimMeta {
            module: "str",
            name: "starts_with",
            arity: 2,
            required_caps: RequiredCaps::none(),
            doc: "Returns true if the string starts with the given prefix.",
        },
        PrimMeta {
            module: "str",
            name: "contains",
            arity: 2,
            required_caps: RequiredCaps::none(),
            doc: "Returns true if the string contains the given substring.",
        },
        // ── collections (no caps) ──
        PrimMeta {
            module: "array",
            name: "insert",
            arity: 3,
            required_caps: RequiredCaps::none(),
            doc: "Insert `value` at index; shifts following elements right.",
        },
        PrimMeta {
            module: "array",
            name: "remove",
            arity: 2,
            required_caps: RequiredCaps::none(),
            doc: "Remove and return the element at index.",
        },
        PrimMeta {
            module: "array",
            name: "sort",
            arity: 1,
            required_caps: RequiredCaps::none(),
            doc: "Sort the array in-place (stable, ascending).",
        },
        // ── crypto (no caps — pure compute) ──
        PrimMeta {
            module: "crypto",
            name: "blake3",
            arity: 1,
            required_caps: RequiredCaps::none(),
            doc: "BLAKE3 hash of a byte sequence (32 bytes).",
        },
        PrimMeta {
            module: "crypto",
            name: "sha256",
            arity: 1,
            required_caps: RequiredCaps::none(),
            doc: "SHA-256 hash of a byte sequence (32 bytes).",
        },
        PrimMeta {
            module: "crypto",
            name: "hmac_sha256",
            arity: 2,
            required_caps: RequiredCaps::none(),
            doc: "HMAC-SHA-256 of a byte sequence with a given key.",
        },
        // ── fs (cap: fs) ──
        PrimMeta {
            module: "fs",
            name: "read_file",
            arity: 1,
            required_caps: RequiredCaps::fs(),
            doc: "Read a UTF-8 file as String.",
        },
        PrimMeta {
            module: "fs",
            name: "write_file",
            arity: 2,
            required_caps: RequiredCaps::fs(),
            doc: "Write a String to a file, creating or truncating.",
        },
        PrimMeta {
            module: "fs",
            name: "read_bytes",
            arity: 1,
            required_caps: RequiredCaps::fs(),
            doc: "Read a file as Bytes.",
        },
        PrimMeta {
            module: "fs",
            name: "write_bytes",
            arity: 2,
            required_caps: RequiredCaps::fs(),
            doc: "Write Bytes to a file, creating or truncating.",
        },
        PrimMeta {
            module: "fs",
            name: "list_dir",
            arity: 1,
            required_caps: RequiredCaps::fs(),
            doc: "List entries in a directory.",
        },
        // ── net (cap: net) ──
        PrimMeta {
            module: "net",
            name: "tcp_connect",
            arity: 2,
            required_caps: RequiredCaps::net(),
            doc: "Open an outbound TCP connection (NetDefaults-gated).",
        },
        PrimMeta {
            module: "net",
            name: "tcp_listen",
            arity: 1,
            required_caps: RequiredCaps::net(),
            doc: "Open a TCP listener on a local port.",
        },
        PrimMeta {
            module: "net",
            name: "udp_bind",
            arity: 1,
            required_caps: RequiredCaps::net(),
            doc: "Bind a UDP socket on a local port.",
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_contains_all_expected_primitives() {
        let t = all_prims();
        // Smoke: each module has its marquee entries.
        assert!(t.contains_key("time::now_ms"));
        assert!(t.contains_key("str::split"));
        assert!(t.contains_key("crypto::blake3"));
        assert!(t.contains_key("fs::read_file"));
        assert!(t.contains_key("net::tcp_connect"));
        assert!(t.contains_key("array::sort"));
    }

    #[test]
    fn caps_are_correct_per_spec() {
        let t = all_prims();
        assert!(t["fs::read_file"].required_caps.contains("fs"));
        assert!(t["net::tcp_connect"].required_caps.contains("net"));
        assert!(t["time::now_ms"].required_caps.contains("time"));
        assert_eq!(t["str::split"].required_caps, RequiredCaps::none());
        assert_eq!(t["crypto::blake3"].required_caps, RequiredCaps::none());
    }

    #[test]
    fn no_primitive_requires_multiple_caps_in_v3_4() {
        // Per v3.4 Security V2 spec: every primitive carries a single cap.
        // Composite caps (e.g., read-file-AND-open-socket) will be a v3.5
        // extension once the call-graph propagator is fully wired.
        let t = all_prims();
        for (name, meta) in &t {
            assert!(
                meta.required_caps.0.len() <= 1,
                "primitive {name} requires {} caps; v3.4 allows at most 1",
                meta.required_caps.0.len()
            );
        }
    }
}
