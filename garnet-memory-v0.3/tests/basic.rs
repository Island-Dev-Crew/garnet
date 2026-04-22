//! Smoke tests for the four memory primitives.

use garnet_memory::*;

#[test]
fn working_store_bulk_free() {
    let store: WorkingStore<String> = WorkingStore::new();
    store.push("a".to_string());
    store.push("b".to_string());
    assert_eq!(store.len(), 2);
    store.clear();
    assert!(store.is_empty());
}

#[test]
fn episode_store_recent() {
    let store: EpisodeStore<i32> = EpisodeStore::new();
    for i in 0..10 {
        store.append_at(i as u64, i);
    }
    let latest = store.recent(3);
    assert_eq!(latest.len(), 3);
    assert_eq!(latest[0].value, 7);
    assert_eq!(latest[2].value, 9);
}

#[test]
fn vector_index_search() {
    let idx: VectorIndex<String> = VectorIndex::new();
    idx.insert(vec![1.0, 0.0, 0.0], "x-axis".to_string());
    idx.insert(vec![0.0, 1.0, 0.0], "y-axis".to_string());
    idx.insert(vec![0.9, 0.1, 0.0], "near-x".to_string());
    let results = idx.search(&[1.0, 0.0, 0.0], 2);
    assert_eq!(results[0].1, "x-axis");
    assert_eq!(results[1].1, "near-x");
}

#[test]
fn workflow_store_versioning() {
    let ws: WorkflowStore<Vec<String>> = WorkflowStore::new();
    ws.register("build", vec!["step1".to_string()]);
    ws.update("build", |mut steps| {
        steps.push("step2".to_string());
        steps
    });
    ws.update("build", |mut steps| {
        steps.push("step3".to_string());
        steps
    });
    let current = ws.find("build").unwrap();
    assert_eq!(current.versions.len(), 3);
    assert_eq!(current.current().unwrap().len(), 3);
    let v0 = ws.replay("build", 0).unwrap();
    assert_eq!(v0.len(), 1);
}

#[test]
fn policy_defaults_are_kind_specific() {
    let w = MemoryPolicy::default_for(MemoryKind::Working);
    let s = MemoryPolicy::default_for(MemoryKind::Semantic);
    // Working decays much faster than semantic.
    assert!(w.decay_lambda_per_sec > s.decay_lambda_per_sec);
    // Semantic retains more (higher threshold is more selective).
    assert!(s.retention_threshold > w.retention_threshold);
}

#[test]
fn policy_score_decays_with_age() {
    let p = MemoryPolicy::default_for(MemoryKind::Working);
    let fresh = p.score(1.0, 0.0, 1.0);
    let old = p.score(1.0, 60.0, 1.0);
    assert!(fresh > old);
    assert!(fresh > 0.5);
}
