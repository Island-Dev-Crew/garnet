//! # Garnet Standard Library (v0.4)
//!
//! P0 stdlib primitives implemented as Rust host functions. Each module
//! corresponds to one Mini-Spec §11.2 / Paper VII §2 primitive category:
//!
//! - [`time`] — wall clock + monotonic clock + sleep (gated on `@caps(time)`)
//! - [`strings`] — split, replace, trim, case conversion (no caps required)
//! - [`collections`] — array ops, map subscript, set operations (no caps)
//! - [`crypto`] — BLAKE3, SHA-256, HMAC-SHA-256 (no caps — pure compute)
//! - [`fs`] — read/write/list files (gated on `@caps(fs)`)
//! - [`net`] — TCP/UDP primitives (gated on `@caps(net)` + NetDefaults)
//!
//! Each primitive is a Rust function that returns `Result<T, StdError>`.
//! The interpreter bridge (Rung 3) wires these into the prelude by
//! registering each function with its required capability set, which the
//! v3.4 CapCaps checker (Security Layer 2) validates against the caller's
//! `@caps(...)` annotation.
//!
//! ## Sequencing — why this crate exists
//!
//! v3.3 shipped the interpreter but not the OS-I/O primitives because
//! shipping `read_file` and `tcp_connect` without their paired security
//! gates would create the "Pegasus default" the user explicitly rejected.
//! v3.4 adds those primitives, but ONLY after v3.4 Security Layer 2 lands
//! in the same release.
//!
//! ## Integration with the interpreter
//!
//! The [`registry`] module exposes a [`PrimTable`] that maps each primitive
//! name to `(required_caps, fn_pointer)`. The interpreter bridge in
//! `garnet-interp` can register this table at startup to expose all
//! primitives to user programs.

pub mod collections;
pub mod crypto;
pub mod error;
pub mod fs;
pub mod net;
pub mod ratelimit;
pub mod registry;
pub mod sandbox;
pub mod strings;
pub mod time;

pub use error::StdError;
pub use ratelimit::{apply_dp_noise, gate_search, IndexPolicy, RateLimitError, RateLimiter};
pub use registry::{PrimTable, RequiredCaps};
pub use sandbox::{
    active_profile, check_cap_permitted, parse_sandbox_arg, reject_unsafe_constructs,
    SandboxProfile, SandboxStatus,
};
