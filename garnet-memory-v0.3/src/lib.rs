//! # Garnet Memory Core (v0.3 stub)
//!
//! Reference implementations of the four memory primitives specified in
//! Mini-Spec v0.3 §4 and `GARNET_Memory_Manager_Architecture.md`. These
//! implementations are intentionally simple — they provide the behavioural
//! contract for the interpreter to target, and serve as the baseline that
//! Rung 5's production Memory Manager will replace with allocator-aware
//! backends.

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
