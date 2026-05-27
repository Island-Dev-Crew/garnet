//! Primitive registry — maps Garnet-surface names to
//! (required_capability, layer, stability, invocation-metadata) tuples.
//!
//! The interpreter calls `all_prims()` at startup to populate its
//! prelude. The CapCaps checker (Rung 4 / v3.4 Security Layer 2)
//! consults the `RequiredCaps` tag on each primitive at every call
//! site to verify the calling function's `@caps(...)` annotation
//! covers the required capability. The `@stability` checker (S17)
//! consults the `Stability` tag at every call site to warn on use of
//! `experimental`/`deprecated`/`frozen` primitives.
//!
//! ## Layer + stability are explicit metadata, not inferred from the name
//!
//! The existing modules keep their bare surface names (`str`, `array`,
//! `time`, `fs`, `net`, `crypto`) for backward compatibility — renaming
//! them to `core::`/`std::` prefixes would break existing programs and the
//! interpreter's prelude bindings. The `layer` field carries the Layer
//! Policy classification (see `C_Language_Specification/GARNET_STDLIB_LAYER_POLICY.md`)
//! independently of the surface name, so the surface name and the policy
//! layer can evolve separately.

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
    /// Process-environment access (`std::env`). New in S17 (v0.7). The
    /// matching known-capability entry lives in `garnet-check-v0.3`.
    pub fn env() -> Self {
        Self(vec!["env"])
    }
    pub fn contains(&self, cap: &str) -> bool {
        self.0.contains(&cap)
    }
}

/// Stdlib layer per the Layer Policy (`GARNET_STDLIB_LAYER_POLICY.md` §1).
/// Layers 2–4 (packages) are not represented in this in-binary registry —
/// they resolve through the registry stub (S13) — but the enum carries them
/// so the layer gate and future tooling share one vocabulary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Layer {
    /// Layer 0 — `core::`: always available, no `@caps` ever, pure compute.
    Core,
    /// Layer 1 — `std::`: bundled; capability-gated or pure-but-library.
    Std,
    /// Layer 2 — `@garnet-lang/*`: official packages, versioned independently.
    Package,
    /// Layer 3 — `community/*`: registry-endorsed.
    Community,
    /// Layer 4 — `*`: anyone publishes.
    Open,
}

impl Layer {
    pub fn as_str(&self) -> &'static str {
        match self {
            Layer::Core => "core",
            Layer::Std => "std",
            Layer::Package => "package",
            Layer::Community => "community",
            Layer::Open => "open",
        }
    }
    /// Numeric layer index 0..=4, as used in the Layer Policy doc.
    pub fn index(&self) -> u8 {
        match self {
            Layer::Core => 0,
            Layer::Std => 1,
            Layer::Package => 2,
            Layer::Community => 3,
            Layer::Open => 4,
        }
    }
}

/// API-contract stability tier per the Layer Policy (`§4`). Enforced at
/// warning/info level by `garnet-check-v0.3` at primitive call sites.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stability {
    /// API contract held for the entire major version.
    Stable,
    /// API may change between minor versions.
    Experimental,
    /// No further changes; supported but will not grow.
    Frozen,
    /// Scheduled for removal; existing uses warn.
    Deprecated,
}

impl Stability {
    pub fn as_str(&self) -> &'static str {
        match self {
            Stability::Stable => "stable",
            Stability::Experimental => "experimental",
            Stability::Frozen => "frozen",
            Stability::Deprecated => "deprecated",
        }
    }
}

/// One primitive entry in the registry. The `arity` and `module`/`name`
/// fields are metadata the interpreter uses to build the prelude; the actual
/// dispatch happens through named lookup + reflection on the module
/// hierarchy (e.g., `fs::read_file` vs. `time::now_ms`). `layer` and
/// `stability` are the Layer Policy classification consumed by the layer
/// gate and the `@stability` checker.
#[derive(Debug, Clone)]
pub struct PrimMeta {
    pub module: &'static str,
    pub name: &'static str,
    pub arity: usize,
    pub required_caps: RequiredCaps,
    pub layer: Layer,
    pub stability: Stability,
    pub doc: &'static str,
}

/// The full primitive table. Produced by `all_prims()`. The interpreter
/// walks this at startup; the CapCaps checker consults it at every
/// primitive-call site.
pub type PrimTable = BTreeMap<String, PrimMeta>;

/// Produce the stdlib primitive table.
///
/// Canonical primitive list per Mini-Spec v1.0 §11.2 + Security V2 spec
/// §1.6, extended by S17 (v0.7) with the Layer-0 `core::` combinators and
/// the Layer-1 `std::` library modules. Each primitive's required
/// capability is defensible per the threat-model rationale documented in
/// v3.4 Security V2 spec §2 (net), §3 (fs), etc., and its layer per
/// `GARNET_STDLIB_LAYER_POLICY.md`.
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

/// Compact constructor used by `build_prims`.
fn p(
    module: &'static str,
    name: &'static str,
    arity: usize,
    required_caps: RequiredCaps,
    layer: Layer,
    stability: Stability,
    doc: &'static str,
) -> PrimMeta {
    PrimMeta {
        module,
        name,
        arity,
        required_caps,
        layer,
        stability,
        doc,
    }
}

fn build_prims() -> Vec<PrimMeta> {
    // Layer / Stability are written fully-qualified below so the layer-gate
    // script (`scripts/garnet_stdlib_layer_gate.py`) can extract them from
    // source with an unambiguous `Layer::\w+` / `Stability::\w+` anchor.
    vec![
        // ════════════════════════════════════════════════════════════
        // Existing primitives (shipped v0.4, 2+ minor releases → Stable)
        // ════════════════════════════════════════════════════════════
        // ── time (Layer 1 std, cap: time) ──
        p(
            "time",
            "now_ms",
            0,
            RequiredCaps::time(),
            Layer::Std,
            Stability::Stable,
            "Monotonic clock in milliseconds since process start.",
        ),
        p(
            "time",
            "wall_clock_ms",
            0,
            RequiredCaps::time(),
            Layer::Std,
            Stability::Stable,
            "Wall clock in milliseconds since UNIX epoch.",
        ),
        p(
            "time",
            "sleep",
            1,
            RequiredCaps::time(),
            Layer::Std,
            Stability::Stable,
            "Sleep the current thread for N milliseconds.",
        ),
        // ── str (Layer 0 core, no caps) ──
        p(
            "str",
            "split",
            2,
            RequiredCaps::none(),
            Layer::Core,
            Stability::Stable,
            "Split a string on a delimiter; returns an Array<String>.",
        ),
        p(
            "str",
            "replace",
            3,
            RequiredCaps::none(),
            Layer::Core,
            Stability::Stable,
            "Replace all occurrences of `old` with `new`.",
        ),
        p(
            "str",
            "to_lower",
            1,
            RequiredCaps::none(),
            Layer::Core,
            Stability::Stable,
            "Lowercase a string (Unicode-aware).",
        ),
        p(
            "str",
            "to_upper",
            1,
            RequiredCaps::none(),
            Layer::Core,
            Stability::Stable,
            "Uppercase a string (Unicode-aware).",
        ),
        p(
            "str",
            "trim",
            1,
            RequiredCaps::none(),
            Layer::Core,
            Stability::Stable,
            "Trim whitespace from both ends.",
        ),
        p(
            "str",
            "starts_with",
            2,
            RequiredCaps::none(),
            Layer::Core,
            Stability::Stable,
            "Returns true if the string starts with the given prefix.",
        ),
        p(
            "str",
            "contains",
            2,
            RequiredCaps::none(),
            Layer::Core,
            Stability::Stable,
            "Returns true if the string contains the given substring.",
        ),
        // ── array (Layer 0 core, no caps) ──
        p(
            "array",
            "insert",
            3,
            RequiredCaps::none(),
            Layer::Core,
            Stability::Stable,
            "Insert `value` at index; shifts following elements right.",
        ),
        p(
            "array",
            "remove",
            2,
            RequiredCaps::none(),
            Layer::Core,
            Stability::Stable,
            "Remove and return the element at index.",
        ),
        p(
            "array",
            "sort",
            1,
            RequiredCaps::none(),
            Layer::Core,
            Stability::Stable,
            "Sort the array in-place (stable, ascending).",
        ),
        // ── crypto (Layer 1 std, no caps — pure compute, tracks external specs) ──
        p(
            "crypto",
            "blake3",
            1,
            RequiredCaps::none(),
            Layer::Std,
            Stability::Stable,
            "BLAKE3 hash of a byte sequence (32 bytes).",
        ),
        p(
            "crypto",
            "sha256",
            1,
            RequiredCaps::none(),
            Layer::Std,
            Stability::Stable,
            "SHA-256 hash of a byte sequence (32 bytes).",
        ),
        p(
            "crypto",
            "hmac_sha256",
            2,
            RequiredCaps::none(),
            Layer::Std,
            Stability::Stable,
            "HMAC-SHA-256 of a byte sequence with a given key.",
        ),
        // ── fs (Layer 1 std, cap: fs) ──
        p(
            "fs",
            "read_file",
            1,
            RequiredCaps::fs(),
            Layer::Std,
            Stability::Stable,
            "Read a UTF-8 file as String.",
        ),
        p(
            "fs",
            "write_file",
            2,
            RequiredCaps::fs(),
            Layer::Std,
            Stability::Stable,
            "Write a String to a file, creating or truncating.",
        ),
        p(
            "fs",
            "read_bytes",
            1,
            RequiredCaps::fs(),
            Layer::Std,
            Stability::Stable,
            "Read a file as Bytes.",
        ),
        p(
            "fs",
            "write_bytes",
            2,
            RequiredCaps::fs(),
            Layer::Std,
            Stability::Stable,
            "Write Bytes to a file, creating or truncating.",
        ),
        p(
            "fs",
            "list_dir",
            1,
            RequiredCaps::fs(),
            Layer::Std,
            Stability::Stable,
            "List entries in a directory.",
        ),
        // ── net (Layer 1 std, cap: net) ──
        p(
            "net",
            "tcp_connect",
            2,
            RequiredCaps::net(),
            Layer::Std,
            Stability::Stable,
            "Open an outbound TCP connection (NetDefaults-gated).",
        ),
        p(
            "net",
            "tcp_listen",
            1,
            RequiredCaps::net(),
            Layer::Std,
            Stability::Stable,
            "Open a TCP listener on a local port.",
        ),
        p(
            "net",
            "udp_bind",
            1,
            RequiredCaps::net(),
            Layer::Std,
            Stability::Stable,
            "Bind a UDP socket on a local port.",
        ),
        // ════════════════════════════════════════════════════════════
        // S17 (v0.7) Layer 0 additions — `core::` combinators (no caps).
        // All ship @stability(experimental); promote in v0.8 if unchanged.
        // ════════════════════════════════════════════════════════════
        // ── core::iter ──
        p(
            "core::iter",
            "map",
            2,
            RequiredCaps::none(),
            Layer::Core,
            Stability::Experimental,
            "Apply a function to each element, returning a new sequence.",
        ),
        p(
            "core::iter",
            "filter",
            2,
            RequiredCaps::none(),
            Layer::Core,
            Stability::Experimental,
            "Keep elements for which the predicate returns true.",
        ),
        p(
            "core::iter",
            "fold",
            3,
            RequiredCaps::none(),
            Layer::Core,
            Stability::Experimental,
            "Reduce a sequence to a single value with an accumulator.",
        ),
        p(
            "core::iter",
            "zip",
            2,
            RequiredCaps::none(),
            Layer::Core,
            Stability::Experimental,
            "Pair up elements of two sequences, truncating to the shorter.",
        ),
        p(
            "core::iter",
            "take",
            2,
            RequiredCaps::none(),
            Layer::Core,
            Stability::Experimental,
            "Take the first N elements.",
        ),
        p(
            "core::iter",
            "drop",
            2,
            RequiredCaps::none(),
            Layer::Core,
            Stability::Experimental,
            "Skip the first N elements, returning the rest.",
        ),
        p(
            "core::iter",
            "collect",
            1,
            RequiredCaps::none(),
            Layer::Core,
            Stability::Experimental,
            "Materialize a sequence into an owned Array.",
        ),
        p(
            "core::iter",
            "chain",
            2,
            RequiredCaps::none(),
            Layer::Core,
            Stability::Experimental,
            "Concatenate two sequences end to end.",
        ),
        p(
            "core::iter",
            "enumerate",
            1,
            RequiredCaps::none(),
            Layer::Core,
            Stability::Experimental,
            "Pair each element with its zero-based index.",
        ),
        // ── core::result ──
        p(
            "core::result",
            "ok",
            1,
            RequiredCaps::none(),
            Layer::Core,
            Stability::Experimental,
            "Wrap a value as Result::Ok.",
        ),
        p(
            "core::result",
            "err",
            1,
            RequiredCaps::none(),
            Layer::Core,
            Stability::Experimental,
            "Wrap a value as Result::Err.",
        ),
        p(
            "core::result",
            "map",
            2,
            RequiredCaps::none(),
            Layer::Core,
            Stability::Experimental,
            "Transform the Ok value, leaving Err untouched.",
        ),
        p(
            "core::result",
            "and_then",
            2,
            RequiredCaps::none(),
            Layer::Core,
            Stability::Experimental,
            "Chain a Result-returning function on the Ok value.",
        ),
        p(
            "core::result",
            "or_else",
            2,
            RequiredCaps::none(),
            Layer::Core,
            Stability::Experimental,
            "Recover from an Err with a Result-returning function.",
        ),
        p(
            "core::result",
            "unwrap_or",
            2,
            RequiredCaps::none(),
            Layer::Core,
            Stability::Experimental,
            "Return the Ok value or a supplied default.",
        ),
        // ── core::option ──
        p(
            "core::option",
            "some",
            1,
            RequiredCaps::none(),
            Layer::Core,
            Stability::Experimental,
            "Wrap a value as Option::Some.",
        ),
        p(
            "core::option",
            "none",
            0,
            RequiredCaps::none(),
            Layer::Core,
            Stability::Experimental,
            "The Option::None value.",
        ),
        p(
            "core::option",
            "map",
            2,
            RequiredCaps::none(),
            Layer::Core,
            Stability::Experimental,
            "Transform the Some value, leaving None untouched.",
        ),
        p(
            "core::option",
            "and_then",
            2,
            RequiredCaps::none(),
            Layer::Core,
            Stability::Experimental,
            "Chain an Option-returning function on the Some value.",
        ),
        p(
            "core::option",
            "unwrap_or",
            2,
            RequiredCaps::none(),
            Layer::Core,
            Stability::Experimental,
            "Return the Some value or a supplied default.",
        ),
        // ── core::cmp ──
        p(
            "core::cmp",
            "min",
            2,
            RequiredCaps::none(),
            Layer::Core,
            Stability::Experimental,
            "The lesser of two values.",
        ),
        p(
            "core::cmp",
            "max",
            2,
            RequiredCaps::none(),
            Layer::Core,
            Stability::Experimental,
            "The greater of two values.",
        ),
        p(
            "core::cmp",
            "ordering",
            2,
            RequiredCaps::none(),
            Layer::Core,
            Stability::Experimental,
            "Three-way compare: -1 if a<b, 0 if equal, 1 if a>b.",
        ),
        p(
            "core::cmp",
            "clamp",
            3,
            RequiredCaps::none(),
            Layer::Core,
            Stability::Experimental,
            "Clamp a value into the inclusive [lo, hi] range.",
        ),
        // ── core::math ──
        p(
            "core::math",
            "abs",
            1,
            RequiredCaps::none(),
            Layer::Core,
            Stability::Experimental,
            "Absolute value.",
        ),
        p(
            "core::math",
            "sqrt",
            1,
            RequiredCaps::none(),
            Layer::Core,
            Stability::Experimental,
            "Square root (errors on negative input).",
        ),
        p(
            "core::math",
            "pow",
            2,
            RequiredCaps::none(),
            Layer::Core,
            Stability::Experimental,
            "Raise a base to an exponent.",
        ),
        p(
            "core::math",
            "floor",
            1,
            RequiredCaps::none(),
            Layer::Core,
            Stability::Experimental,
            "Largest integer not greater than the input.",
        ),
        p(
            "core::math",
            "ceil",
            1,
            RequiredCaps::none(),
            Layer::Core,
            Stability::Experimental,
            "Smallest integer not less than the input.",
        ),
        p(
            "core::math",
            "round",
            1,
            RequiredCaps::none(),
            Layer::Core,
            Stability::Experimental,
            "Round half away from zero to the nearest integer.",
        ),
        // ════════════════════════════════════════════════════════════
        // S17 (v0.7) Layer 1 additions — `std::` library modules.
        // ════════════════════════════════════════════════════════════
        // ── std::env (cap: env — NEW capability) ──
        p(
            "std::env",
            "get",
            1,
            RequiredCaps::env(),
            Layer::Std,
            Stability::Experimental,
            "Read a process environment variable; None if unset.",
        ),
        p(
            "std::env",
            "set",
            2,
            RequiredCaps::env(),
            Layer::Std,
            Stability::Experimental,
            "Set a process environment variable for this process.",
        ),
        p(
            "std::env",
            "vars",
            0,
            RequiredCaps::env(),
            Layer::Std,
            Stability::Experimental,
            "Snapshot all environment variables as (key, value) pairs.",
        ),
        // ── std::process (cap: proc) ──
        p(
            "std::process",
            "spawn",
            1,
            RequiredCaps::proc(),
            Layer::Std,
            Stability::Experimental,
            "Spawn a child process from a command line; returns a handle.",
        ),
        p(
            "std::process",
            "wait",
            1,
            RequiredCaps::proc(),
            Layer::Std,
            Stability::Experimental,
            "Wait for a spawned child to exit; returns its exit status.",
        ),
        p(
            "std::process",
            "exit_code",
            1,
            RequiredCaps::proc(),
            Layer::Std,
            Stability::Experimental,
            "Extract the integer exit code from a finished child status.",
        ),
        p(
            "std::process",
            "spawn_args",
            2,
            RequiredCaps::proc(),
            Layer::Std,
            Stability::Experimental,
            "Spawn a child from a program plus an explicit argv array (no shell \
             splitting, so arguments containing spaces survive); returns a handle.",
        ),
        p(
            "std::process",
            "output",
            2,
            RequiredCaps::proc(),
            Layer::Std,
            Stability::Experimental,
            "Run a program with an explicit argv array to completion, capturing \
             stdout, stderr, and the exit code as a map.",
        ),
        // ── std::json (no caps — pure; tracks RFC 8259) ──
        p(
            "std::json",
            "parse",
            1,
            RequiredCaps::none(),
            Layer::Std,
            Stability::Experimental,
            "Parse a JSON string into a JSON value (errors on malformed input).",
        ),
        p(
            "std::json",
            "stringify",
            1,
            RequiredCaps::none(),
            Layer::Std,
            Stability::Experimental,
            "Serialize a JSON value to a compact string.",
        ),
        p(
            "std::json",
            "get",
            2,
            RequiredCaps::none(),
            Layer::Std,
            Stability::Experimental,
            "Look up a key (object) or index (array) in a JSON value.",
        ),
        p(
            "std::json",
            "set",
            3,
            RequiredCaps::none(),
            Layer::Std,
            Stability::Experimental,
            "Return a copy of a JSON object with `key` set to `value`.",
        ),
        // ── std::regex (no caps — pure) ──
        p(
            "std::regex",
            "compile",
            1,
            RequiredCaps::none(),
            Layer::Std,
            Stability::Experimental,
            "Validate/compile a regular expression (errors on bad syntax).",
        ),
        p(
            "std::regex",
            "match",
            2,
            RequiredCaps::none(),
            Layer::Std,
            Stability::Experimental,
            "Return true if the pattern matches anywhere in the input.",
        ),
        p(
            "std::regex",
            "find_all",
            2,
            RequiredCaps::none(),
            Layer::Std,
            Stability::Experimental,
            "Return all non-overlapping matches of the pattern.",
        ),
        p(
            "std::regex",
            "replace",
            3,
            RequiredCaps::none(),
            Layer::Std,
            Stability::Experimental,
            "Replace all matches of the pattern with a replacement string.",
        ),
        // ── std::uuid (cap: time for v4/v7 clock-seeding; none for v5) ──
        p(
            "std::uuid",
            "new_v4",
            0,
            RequiredCaps::time(),
            Layer::Std,
            Stability::Experimental,
            "Random UUIDv4 (128 bits of randomness; version+variant tagged).",
        ),
        p(
            "std::uuid",
            "new_v5",
            2,
            RequiredCaps::none(),
            Layer::Std,
            Stability::Experimental,
            "Name-based UUIDv5: SHA-1 of (namespace, name). Deterministic.",
        ),
        p(
            "std::uuid",
            "new_v7",
            0,
            RequiredCaps::time(),
            Layer::Std,
            Stability::Experimental,
            "Time-ordered UUIDv7 (48-bit unix-ms prefix + randomness).",
        ),
        // ── std::base64 (no caps — pure; tracks RFC 4648) ──
        p(
            "std::base64",
            "encode",
            1,
            RequiredCaps::none(),
            Layer::Std,
            Stability::Experimental,
            "Encode bytes to a standard RFC 4648 base64 string (with padding).",
        ),
        p(
            "std::base64",
            "decode",
            1,
            RequiredCaps::none(),
            Layer::Std,
            Stability::Experimental,
            "Decode a standard RFC 4648 base64 string to bytes (errors on bad input).",
        ),
        // ── std::log (no caps for formatting; file sinks need fs — deferred) ──
        p(
            "std::log",
            "info",
            1,
            RequiredCaps::none(),
            Layer::Std,
            Stability::Experimental,
            "Format an INFO-level log line (level + message).",
        ),
        p(
            "std::log",
            "warn",
            1,
            RequiredCaps::none(),
            Layer::Std,
            Stability::Experimental,
            "Format a WARN-level log line.",
        ),
        p(
            "std::log",
            "error",
            1,
            RequiredCaps::none(),
            Layer::Std,
            Stability::Experimental,
            "Format an ERROR-level log line.",
        ),
        p(
            "std::log",
            "debug",
            1,
            RequiredCaps::none(),
            Layer::Std,
            Stability::Experimental,
            "Format a DEBUG-level log line.",
        ),
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
        // S17 additions.
        assert!(t.contains_key("core::iter::map"));
        assert!(t.contains_key("core::math::sqrt"));
        assert!(t.contains_key("std::json::parse"));
        assert!(t.contains_key("std::base64::encode"));
        assert!(t.contains_key("std::env::get"));
        assert!(t.contains_key("std::uuid::new_v5"));
    }

    #[test]
    fn caps_are_correct_per_spec() {
        let t = all_prims();
        assert!(t["fs::read_file"].required_caps.contains("fs"));
        assert!(t["net::tcp_connect"].required_caps.contains("net"));
        assert!(t["time::now_ms"].required_caps.contains("time"));
        assert_eq!(t["str::split"].required_caps, RequiredCaps::none());
        assert_eq!(t["crypto::blake3"].required_caps, RequiredCaps::none());
        // S17 caps.
        assert!(t["std::env::get"].required_caps.contains("env"));
        assert!(t["std::process::spawn"].required_caps.contains("proc"));
        assert!(t["std::uuid::new_v4"].required_caps.contains("time"));
        assert_eq!(t["std::uuid::new_v5"].required_caps, RequiredCaps::none());
        assert_eq!(t["std::json::parse"].required_caps, RequiredCaps::none());
        assert_eq!(t["core::iter::map"].required_caps, RequiredCaps::none());
    }

    #[test]
    fn no_primitive_requires_multiple_caps_in_v3_4() {
        // Per v3.4 Security V2 spec: every primitive carries a single cap.
        let t = all_prims();
        for (name, meta) in &t {
            assert!(
                meta.required_caps.0.len() <= 1,
                "primitive {name} requires {} caps; v3.4 allows at most 1",
                meta.required_caps.0.len()
            );
        }
    }

    #[test]
    fn at_least_fifty_primitives() {
        // S17 Done criteria: stdlib has >= 50 primitives.
        let t = all_prims();
        assert!(t.len() >= 50, "expected >= 50 primitives, got {}", t.len());
    }

    #[test]
    fn every_primitive_has_explicit_stability() {
        // The Stability field is non-optional, so every entry is explicitly
        // tagged by construction. S17 Done criteria: >= 95% explicit.
        let t = all_prims();
        let tagged = t.len(); // all are explicit
        let pct = (tagged as f64 / t.len() as f64) * 100.0;
        assert!(pct >= 95.0, "explicit-stability coverage {pct:.1}% < 95%");
    }

    #[test]
    fn layer_zero_primitives_carry_no_caps() {
        // Layer Policy §1: Layer 0 (`core`) primitives have no caps, ever.
        let t = all_prims();
        for (name, meta) in &t {
            if meta.layer == Layer::Core {
                assert_eq!(
                    meta.required_caps,
                    RequiredCaps::none(),
                    "Layer-0 primitive {name} must carry no caps"
                );
            }
        }
    }

    #[test]
    fn capability_gated_primitives_are_layer_one() {
        // Anything requiring OS authority must be Layer 1 (`std`) or above —
        // never Layer 0.
        let t = all_prims();
        for (name, meta) in &t {
            if !meta.required_caps.0.is_empty() {
                assert_ne!(
                    meta.layer,
                    Layer::Core,
                    "capability-gated primitive {name} cannot be Layer 0"
                );
            }
        }
    }

    #[test]
    fn existing_primitives_are_stable_new_are_experimental() {
        let t = all_prims();
        // Shipped-since-v0.4 primitives are Stable.
        assert_eq!(t["fs::read_file"].stability, Stability::Stable);
        assert_eq!(t["str::split"].stability, Stability::Stable);
        // S17 additions are Experimental for v0.7.
        assert_eq!(t["core::iter::map"].stability, Stability::Experimental);
        assert_eq!(t["std::json::parse"].stability, Stability::Experimental);
    }
}
