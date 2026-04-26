//! # Mnemos — reference implementation of Garnet's Memory Core
//!
//! **Memory Core** is the architectural noun for Garnet's first-class,
//! cognitively-inspired memory subsystem (Mini-Spec v1.0 §4; Paper IV
//! "One Memory Core, Many Harnesses"). It is one of Garnet's two
//! load-bearing differentiators against mainstream PLs (the other being
//! the dual-mode managed/safe boundary).
//!
//! **Mnemos** is the codename for the v0.4.x reference *implementation*
//! of the Memory Core — the crate you are reading. Mnemos provides
//! behavioural-contract reference stores for the four memory kinds:
//!
//! | Kind          | Purpose                                  | Reference store                     |
//! |---------------|------------------------------------------|-------------------------------------|
//! | [`WorkingStore`]   | Bulk-allocated scratch tied to a scope   | Arena `RefCell<Vec<T>>`             |
//! | [`EpisodeStore`]   | Append-only timestamped event log        | `RefCell<Vec<Episode<T>>>`          |
//! | [`VectorIndex`]    | Cosine-similarity semantic recall        | `RefCell<Vec<(Vec<f32>, T)>>`       |
//! | [`WorkflowStore`]  | Copy-on-write versioned procedure store  | `RefCell<BTreeMap<Version, T>>`     |
//!
//! ## What "reference implementation" means
//!
//! Each store is the **simplest correct implementation** of the
//! Mini-Spec contract. They are designed for the interpreter
//! (`garnet-interp-v0.3`) to target so the language semantics stay
//! testable end-to-end while the production allocator work proceeds
//! in parallel. They are NOT designed for production agent workloads:
//!
//! - No allocator integration (everything goes through `Vec` / `BTreeMap`).
//! - No persistence (state is in-process only).
//! - No production-grade vector index (cosine over a flat `Vec`, not
//!   HNSW / IVF / PolarQuant).
//! - No eviction beyond what `MemoryPolicy` exposes as scoring API.
//! - No ARC + Bacon–Rajan cycle detection (Mini-Spec §4.5 is deferred).
//!
//! ## What's planned
//!
//! The production Memory Core is sequenced in
//! `C_Language_Specification/MEMORY_CORE_ROADMAP.md`. Same crate name,
//! same public types — Mnemos matures from "behavioural contract
//! reference" to "production allocator with persistence and vector
//! indexing" without churning the call surface.
//!
//! ## Quoting / citing this subsystem
//!
//! - Architectural / spec / paper text → "the Memory Core" or
//!   "Garnet's Memory Core".
//! - Implementation / crate / docs / talks → "Mnemos" (e.g. "Mnemos
//!   ships the v0.4.x reference stores; production backends land in
//!   v0.5.x per the Memory Core Roadmap").
//! - Mini-Spec section reference → §4 (declaration, semantics, kinds)
//!   and the deferred §4.4 (generics over kinds) / §4.5 (ARC).

pub mod episodic;
pub mod policy;
pub mod procedural;
pub mod semantic;
pub mod working;

pub use episodic::EpisodeStore;
pub use policy::{MemoryKind, MemoryPolicy};
pub use procedural::WorkflowStore;
pub use semantic::VectorIndex;
pub use working::WorkingStore;

/// A handle that ties a named memory unit to its runtime store. Interpreters
/// and harnesses hold these; the language-level `memory` declaration produces
/// one at scope entry.
pub struct MemoryHandle<S> {
    pub name: String,
    pub kind: MemoryKind,
    pub store: S,
    pub policy: MemoryPolicy,
}

impl<S: Default> MemoryHandle<S> {
    pub fn new(name: impl Into<String>, kind: MemoryKind) -> Self {
        let policy = MemoryPolicy::default_for(kind);
        Self {
            name: name.into(),
            kind,
            store: S::default(),
            policy,
        }
    }
}
