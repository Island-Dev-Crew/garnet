//! Kind-aware allocation dispatch — Paper VI Contribution 4.
//!
//! Each `memory <kind> <name> : <type>` declaration must produce a
//! `MemoryStore` value whose `backend` is the purpose-built store for
//! that kind. These tests prove every kind dispatches correctly and that
//! the resulting backend honours the kind-appropriate methods (and only
//! those).

use garnet_interp::{Interpreter, Value};

fn load(src: &str) -> Interpreter {
    let mut interp = Interpreter::new();
    interp.load_source(src).expect("load");
    interp
}

fn fetch(interp: &Interpreter, name: &str) -> Value {
    interp.global.get(name).expect("binding")
}

// ── Backend kind dispatch ───────────────────────────────────────────

#[test]
fn working_kind_produces_workingstore_backend() {
    let interp = load("memory working scratch : Buffer");
    let v = fetch(&interp, "scratch");
    if let Value::MemoryStore { backend, .. } = v {
        assert_eq!(backend.kind_name(), "WorkingStore");
    } else {
        panic!("expected MemoryStore");
    }
}

#[test]
fn episodic_kind_produces_episodestore_backend() {
    let interp = load("memory episodic events : EpisodeStore<Event>");
    let v = fetch(&interp, "events");
    if let Value::MemoryStore { backend, .. } = v {
        assert_eq!(backend.kind_name(), "EpisodeStore");
    } else {
        panic!("expected MemoryStore");
    }
}

#[test]
fn semantic_kind_produces_vectorindex_backend() {
    let interp = load("memory semantic facts : VectorIndex<Fact>");
    let v = fetch(&interp, "facts");
    if let Value::MemoryStore { backend, .. } = v {
        assert_eq!(backend.kind_name(), "VectorIndex");
    } else {
        panic!("expected MemoryStore");
    }
}

#[test]
fn procedural_kind_produces_workflowstore_backend() {
    let interp = load("memory procedural workflows : WorkflowStore<Trace>");
    let v = fetch(&interp, "workflows");
    if let Value::MemoryStore { backend, .. } = v {
        assert_eq!(backend.kind_name(), "WorkflowStore");
    } else {
        panic!("expected MemoryStore");
    }
}

// ── Each kind exposes only its appropriate operations ───────────────

#[test]
fn working_store_supports_push_len_clear() {
    let mut interp = Interpreter::new();
    interp
        .load_source(
            r#"
        memory working scratch : Buffer
        def main() {
            scratch.push(1)
            scratch.push(2)
            scratch.push(3)
            scratch.len()
        }
    "#,
        )
        .unwrap();
    let r = interp.call("main", vec![]).unwrap();
    assert!(matches!(r, Value::Int(3)));
}

#[test]
fn episodic_store_supports_append_recent() {
    let mut interp = Interpreter::new();
    interp
        .load_source(
            r#"
        memory episodic log : EpisodeStore<Event>
        def main() {
            log.append("e1")
            log.append("e2")
            log.append("e3")
            let last = log.recent(2)
            last.len()
        }
    "#,
        )
        .unwrap();
    let r = interp.call("main", vec![]).unwrap();
    assert!(matches!(r, Value::Int(2)));
}

#[test]
fn semantic_store_supports_insert_search() {
    let mut interp = Interpreter::new();
    interp
        .load_source(
            r#"
        memory semantic facts : VectorIndex<Fact>
        def main() {
            facts.insert([1.0, 0.0, 0.0], "x-axis")
            facts.insert([0.0, 1.0, 0.0], "y-axis")
            let r = facts.search([1.0, 0.0, 0.0], 1)
            r.len()
        }
    "#,
        )
        .unwrap();
    let r = interp.call("main", vec![]).unwrap();
    assert!(matches!(r, Value::Int(1)));
}

#[test]
fn procedural_store_supports_register_find() {
    let mut interp = Interpreter::new();
    interp
        .load_source(
            r#"
        memory procedural workflows : WorkflowStore<Trace>
        def main() {
            workflows.register("build", "step1")
            workflows.find("build")
        }
    "#,
        )
        .unwrap();
    let r = interp.call("main", vec![]).unwrap();
    if let Value::Str(s) = r {
        assert_eq!(s.as_str(), "step1");
    } else {
        panic!("expected string");
    }
}

// ── Kind isolation: wrong kind for method errors ────────────────────

#[test]
fn working_store_rejects_recent_method() {
    let mut interp = Interpreter::new();
    interp
        .load_source(
            r#"
        memory working scratch : Buffer
        def main() { scratch.recent(5) }
    "#,
        )
        .unwrap();
    let r = interp.call("main", vec![]);
    assert!(r.is_err(), "WorkingStore should not have .recent() — that's an EpisodeStore-only API");
}

#[test]
fn semantic_store_rejects_recent_method() {
    let mut interp = Interpreter::new();
    interp
        .load_source(
            r#"
        memory semantic kb : VectorIndex<Fact>
        def main() { kb.recent(5) }
    "#,
        )
        .unwrap();
    let r = interp.call("main", vec![]);
    assert!(r.is_err());
}

#[test]
fn procedural_store_rejects_search_method() {
    let mut interp = Interpreter::new();
    interp
        .load_source(
            r#"
        memory procedural wf : WorkflowStore<Trace>
        def main() { wf.search([1.0, 0.0], 1) }
    "#,
        )
        .unwrap();
    let r = interp.call("main", vec![]);
    assert!(r.is_err(), "WorkflowStore should not have .search()");
}

// ── Multiple bindings to same memory unit share backing (ARC) ───────

#[test]
fn rebind_shares_backend_so_writes_are_visible() {
    let mut interp = Interpreter::new();
    interp
        .load_source(
            r#"
        memory working scratch : Buffer
        def writer() {
            scratch.push(42)
        }
        def main() {
            writer()
            writer()
            writer()
            scratch.len()
        }
    "#,
        )
        .unwrap();
    let r = interp.call("main", vec![]).unwrap();
    assert!(matches!(r, Value::Int(3)));
}
