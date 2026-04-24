//! KindGuard tests — v3.3 Security Layer 1 (hardening #13).
//!
//! Defense-in-depth against post-codegen memory-kind confusion. The
//! `MemoryBackend` enum already provides compile-time kind safety in
//! v3.2, but a future IR lowering could drop the enum discriminant
//! and leave the handle as an undifferentiated pointer. The `KindTag`
//! tag + `ensure_kind_matches` check survives that lowering: a
//! mismatched `Value::MemoryStore` (declared kind disagrees with
//! backend) fails with a clear error at dispatch time.

use garnet_interp::value::{KindTag, MemoryBackend, Value};
use garnet_interp::Interpreter;
use garnet_parser::ast::MemoryKind;
use std::rc::Rc;

// ── Happy path: matching declared kind + backend dispatches normally ──

#[test]
fn matching_kind_and_backend_dispatches_as_usual() {
    // Baseline sanity: the kind-aware dispatch still works for well-
    // formed MemoryStores constructed via `MemoryBackend::for_kind`.
    let mut interp = Interpreter::new();
    interp
        .load_source(
            r#"
        memory working scratch : Buffer
        def main() {
            scratch.push(1)
            scratch.push(2)
            scratch.len()
        }
    "#,
        )
        .unwrap();
    let r = interp.call("main", vec![]).unwrap();
    assert!(matches!(r, Value::Int(2)));
}

// ── Tag sanity ──────────────────────────────────────────────────────

#[test]
fn kind_tag_values_are_non_sequential() {
    // Non-sequential u8 values make random/zero-byte corruption loud.
    assert_eq!(KindTag::Working as u8, 0x57);
    assert_eq!(KindTag::Episodic as u8, 0x45);
    assert_eq!(KindTag::Semantic as u8, 0x53);
    assert_eq!(KindTag::Procedural as u8, 0x50);
    // Distinctness:
    let tags = [
        KindTag::Working as u8,
        KindTag::Episodic as u8,
        KindTag::Semantic as u8,
        KindTag::Procedural as u8,
    ];
    let unique: std::collections::HashSet<_> = tags.iter().copied().collect();
    assert_eq!(unique.len(), 4, "all 4 tag values must be distinct");
}

#[test]
fn kind_tag_name_matches_backend_kind_name() {
    // kind_tag().name() must match the v3.2 kind_name string so tests
    // keyed on kind_name continue to work.
    let working = MemoryBackend::for_kind(MemoryKind::Working);
    assert_eq!(working.kind_tag().name(), working.kind_name());

    let episodic = MemoryBackend::for_kind(MemoryKind::Episodic);
    assert_eq!(episodic.kind_tag().name(), episodic.kind_name());

    let semantic = MemoryBackend::for_kind(MemoryKind::Semantic);
    assert_eq!(semantic.kind_tag().name(), semantic.kind_name());

    let procedural = MemoryBackend::for_kind(MemoryKind::Procedural);
    assert_eq!(procedural.kind_tag().name(), procedural.kind_name());
}

// ── ensure_kind_matches: happy paths ────────────────────────────────

#[test]
fn ensure_kind_matches_accepts_correct_pairing() {
    let backend = MemoryBackend::for_kind(MemoryKind::Working);
    assert!(backend.ensure_kind_matches(MemoryKind::Working).is_ok());

    let backend = MemoryBackend::for_kind(MemoryKind::Episodic);
    assert!(backend.ensure_kind_matches(MemoryKind::Episodic).is_ok());
}

// ── ensure_kind_matches: adversarial paths (all 4x4 = 12 mismatches) ──

#[test]
fn ensure_kind_matches_rejects_working_backend_declared_as_episodic() {
    let backend = MemoryBackend::for_kind(MemoryKind::Working);
    let err = backend
        .ensure_kind_matches(MemoryKind::Episodic)
        .expect_err("must reject Working backend declared as Episodic");
    assert_eq!(err.actual, KindTag::Working);
    assert_eq!(err.expected, KindTag::Episodic);
}

#[test]
fn ensure_kind_matches_rejects_all_off_diagonal_pairings() {
    let kinds = [
        MemoryKind::Working,
        MemoryKind::Episodic,
        MemoryKind::Semantic,
        MemoryKind::Procedural,
    ];
    for &backend_kind in &kinds {
        for &declared_kind in &kinds {
            let backend = MemoryBackend::for_kind(backend_kind);
            let result = backend.ensure_kind_matches(declared_kind);
            if backend_kind == declared_kind {
                assert!(
                    result.is_ok(),
                    "matching kinds must accept: backend={backend_kind:?} declared={declared_kind:?}"
                );
            } else {
                assert!(
                    result.is_err(),
                    "mismatched kinds must reject: backend={backend_kind:?} declared={declared_kind:?}"
                );
            }
        }
    }
}

// ── Dispatch-level: adversarial Value::MemoryStore catches mismatch ──

#[test]
fn dispatch_rejects_mismatched_kind_and_backend_with_clear_error() {
    // Construct a Value::MemoryStore via direct struct init (bypassing
    // `for_kind`) with a mismatched kind tag vs. backend. This is the
    // exact shape of the threat KindGuard defends against: if any
    // future IR lowering drops the enum discriminant on the `backend`
    // field, or if reflection/FFI constructs such a value, the tag
    // check at dispatch time must reject it.
    let bad_store = Value::MemoryStore {
        kind: MemoryKind::Working,
        name: "imposter".to_string(),
        backend: MemoryBackend::Semantic(Rc::new(garnet_memory::VectorIndex::new())),
    };

    // Register the adversarial value into a fresh interpreter's global scope.
    let mut interp = Interpreter::new();
    interp.global.define("imposter", bad_store);

    // Invoke a method on it via the interpreter. The KindGuard at
    // eval.rs:574 should catch the mismatch before any dispatch arm runs.
    interp
        .load_source("def touch_imposter() { imposter.push(1) }")
        .unwrap();
    let err = interp
        .call("touch_imposter", vec![])
        .expect_err("KindGuard must reject the mismatched MemoryStore");
    let err_text = format!("{err:?}");
    assert!(
        err_text.contains("kind mismatch"),
        "expected 'kind mismatch' error, got: {err_text}"
    );
    // Both the declared and actual names should appear in the diagnostic.
    assert!(
        err_text.contains("WorkingStore"),
        "diagnostic must name declared kind"
    );
    assert!(
        err_text.contains("VectorIndex"),
        "diagnostic must name actual backend"
    );
}

#[test]
fn dispatch_rejects_episodic_declared_as_procedural() {
    // Another permutation: episodic backend declared as procedural
    // would otherwise let `.register()` reach an EpisodeStore that has
    // no such method — KindGuard kicks in first.
    let bad_store = Value::MemoryStore {
        kind: MemoryKind::Procedural,
        name: "imposter".to_string(),
        backend: MemoryBackend::Episodic(Rc::new(garnet_memory::EpisodeStore::new())),
    };

    let mut interp = Interpreter::new();
    interp.global.define("imposter", bad_store);
    interp
        .load_source(r#"def touch() { imposter.register("w", "s") }"#)
        .unwrap();
    let err = interp
        .call("touch", vec![])
        .expect_err("must reject mismatch");
    let err_text = format!("{err:?}");
    assert!(err_text.contains("kind mismatch"));
}
