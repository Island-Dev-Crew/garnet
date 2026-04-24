//! Property-style tests for the four reference memory stores. These are
//! deterministic small-input "property" checks (not randomised) that pin the
//! invariants documented in `GARNET_Memory_Manager_Architecture.md`.

use garnet_memory::*;

// ════════════════════════════════════════════════════════════════════
// WorkingStore — arena semantics
// ════════════════════════════════════════════════════════════════════

#[test]
fn working_store_starts_empty() {
    let s: WorkingStore<i32> = WorkingStore::new();
    assert!(s.is_empty());
    assert_eq!(s.len(), 0);
}

#[test]
fn working_store_push_returns_dense_index() {
    let s: WorkingStore<i32> = WorkingStore::new();
    assert_eq!(s.push(10), 0);
    assert_eq!(s.push(20), 1);
    assert_eq!(s.push(30), 2);
}

#[test]
fn working_store_with_reads_pushed_value() {
    let s: WorkingStore<String> = WorkingStore::new();
    s.push("hello".to_string());
    let r = s.with(0, |v| v.len());
    assert_eq!(r, Some(5));
}

#[test]
fn working_store_with_out_of_bounds_returns_none() {
    let s: WorkingStore<i32> = WorkingStore::new();
    s.push(1);
    assert!(s.with(99, |v| *v).is_none());
}

#[test]
fn working_store_clear_resets() {
    let s: WorkingStore<i32> = WorkingStore::new();
    s.push(1);
    s.push(2);
    s.push(3);
    s.clear();
    assert!(s.is_empty());
}

#[test]
fn working_store_snapshot_clones() {
    let s: WorkingStore<i32> = WorkingStore::new();
    s.push(1);
    s.push(2);
    let snap = s.snapshot();
    assert_eq!(snap, vec![1, 2]);
}

#[test]
fn working_store_thousand_pushes() {
    let s: WorkingStore<i32> = WorkingStore::new();
    for i in 0..1000 {
        s.push(i);
    }
    assert_eq!(s.len(), 1000);
}

// ════════════════════════════════════════════════════════════════════
// EpisodeStore — append-only log
// ════════════════════════════════════════════════════════════════════

#[test]
fn episode_store_starts_empty() {
    let s: EpisodeStore<i32> = EpisodeStore::new();
    assert!(s.is_empty());
}

#[test]
fn episode_store_append_grows_len() {
    let s: EpisodeStore<i32> = EpisodeStore::new();
    s.append(1);
    s.append(2);
    s.append(3);
    assert_eq!(s.len(), 3);
}

#[test]
fn episode_store_append_at_uses_explicit_timestamp() {
    let s: EpisodeStore<&str> = EpisodeStore::new();
    s.append_at(100, "first");
    s.append_at(200, "second");
    let snap = s.snapshot();
    assert_eq!(snap[0].timestamp_unix, 100);
    assert_eq!(snap[1].timestamp_unix, 200);
}

#[test]
fn episode_store_recent_n_returns_n_most_recent() {
    let s: EpisodeStore<i32> = EpisodeStore::new();
    for i in 0..10 {
        s.append_at(i as u64, i);
    }
    let recent = s.recent(3);
    assert_eq!(recent.len(), 3);
    assert_eq!(recent[0].value, 7);
    assert_eq!(recent[1].value, 8);
    assert_eq!(recent[2].value, 9);
}

#[test]
fn episode_store_recent_with_n_larger_than_size() {
    let s: EpisodeStore<i32> = EpisodeStore::new();
    s.append_at(1, 100);
    s.append_at(2, 200);
    let recent = s.recent(99);
    assert_eq!(recent.len(), 2);
}

#[test]
fn episode_store_since_filters_by_timestamp() {
    let s: EpisodeStore<i32> = EpisodeStore::new();
    s.append_at(10, 1);
    s.append_at(20, 2);
    s.append_at(30, 3);
    s.append_at(40, 4);
    let since = s.since(25);
    assert_eq!(since.len(), 2);
    assert_eq!(since[0].value, 3);
    assert_eq!(since[1].value, 4);
}

#[test]
fn episode_store_since_zero_returns_all() {
    let s: EpisodeStore<i32> = EpisodeStore::new();
    s.append_at(100, 1);
    s.append_at(200, 2);
    assert_eq!(s.since(0).len(), 2);
}

// ════════════════════════════════════════════════════════════════════
// VectorIndex — cosine search
// ════════════════════════════════════════════════════════════════════

#[test]
fn vector_index_starts_empty() {
    let idx: VectorIndex<&str> = VectorIndex::new();
    assert!(idx.is_empty());
    assert_eq!(idx.len(), 0);
}

#[test]
fn vector_index_insert_grows_len() {
    let idx: VectorIndex<&str> = VectorIndex::new();
    idx.insert(vec![1.0, 0.0, 0.0], "x");
    idx.insert(vec![0.0, 1.0, 0.0], "y");
    assert_eq!(idx.len(), 2);
}

#[test]
fn vector_index_search_orders_by_cosine_descending() {
    let idx: VectorIndex<&str> = VectorIndex::new();
    idx.insert(vec![1.0, 0.0, 0.0], "x_axis");
    idx.insert(vec![0.0, 1.0, 0.0], "y_axis");
    idx.insert(vec![0.0, 0.0, 1.0], "z_axis");
    idx.insert(vec![0.95, 0.05, 0.0], "near_x");

    let r = idx.search(&[1.0, 0.0, 0.0], 4);
    assert_eq!(r[0].1, "x_axis");
    assert_eq!(r[1].1, "near_x");
}

#[test]
fn vector_index_search_top_k_truncates() {
    let idx: VectorIndex<i32> = VectorIndex::new();
    for i in 0..5 {
        idx.insert(vec![i as f32, 0.0], i);
    }
    let r = idx.search(&[1.0, 0.0], 2);
    assert_eq!(r.len(), 2);
}

#[test]
fn vector_index_orthogonal_yields_zero_cosine() {
    let idx: VectorIndex<&str> = VectorIndex::new();
    idx.insert(vec![1.0, 0.0], "right");
    let r = idx.search(&[0.0, 1.0], 1);
    assert!(r[0].0.abs() < 1e-6);
}

#[test]
fn vector_index_identical_vectors_yield_one_cosine() {
    let idx: VectorIndex<&str> = VectorIndex::new();
    idx.insert(vec![3.0, 4.0], "v");
    let r = idx.search(&[3.0, 4.0], 1);
    assert!((r[0].0 - 1.0).abs() < 1e-6);
}

#[test]
fn vector_index_dim_mismatch_yields_zero() {
    let idx: VectorIndex<&str> = VectorIndex::new();
    idx.insert(vec![1.0, 0.0, 0.0], "3d");
    let r = idx.search(&[1.0, 0.0], 1);
    assert!(r[0].0.abs() < 1e-9);
}

// ════════════════════════════════════════════════════════════════════
// WorkflowStore — copy-on-write
// ════════════════════════════════════════════════════════════════════

#[test]
fn workflow_store_register_then_find() {
    let ws: WorkflowStore<Vec<i32>> = WorkflowStore::new();
    ws.register("build", vec![1, 2, 3]);
    let w = ws.find("build").unwrap();
    assert_eq!(w.versions.len(), 1);
}

#[test]
fn workflow_store_update_appends_version() {
    let ws: WorkflowStore<Vec<i32>> = WorkflowStore::new();
    ws.register("p", vec![]);
    ws.update("p", |mut v| {
        v.push(1);
        v
    });
    ws.update("p", |mut v| {
        v.push(2);
        v
    });
    ws.update("p", |mut v| {
        v.push(3);
        v
    });
    let w = ws.find("p").unwrap();
    assert_eq!(w.versions.len(), 4); // initial + 3 updates
}

#[test]
fn workflow_store_old_versions_preserved() {
    let ws: WorkflowStore<Vec<i32>> = WorkflowStore::new();
    ws.register("p", vec![10]);
    ws.update("p", |mut v| {
        v.push(20);
        v
    });
    let v0 = ws.replay("p", 0).unwrap();
    let v1 = ws.replay("p", 1).unwrap();
    assert_eq!(v0, vec![10]);
    assert_eq!(v1, vec![10, 20]);
}

#[test]
fn workflow_store_replay_out_of_bounds_returns_none() {
    let ws: WorkflowStore<Vec<i32>> = WorkflowStore::new();
    ws.register("p", vec![1]);
    assert!(ws.replay("p", 99).is_none());
}

#[test]
fn workflow_store_find_unknown_returns_none() {
    let ws: WorkflowStore<i32> = WorkflowStore::new();
    assert!(ws.find("missing").is_none());
}

#[test]
fn workflow_store_current_returns_latest() {
    let ws: WorkflowStore<i32> = WorkflowStore::new();
    ws.register("counter", 0);
    ws.update("counter", |n| n + 1);
    ws.update("counter", |n| n + 1);
    let w = ws.find("counter").unwrap();
    assert_eq!(*w.current().unwrap(), 2);
}

// ════════════════════════════════════════════════════════════════════
// MemoryPolicy — R+R+I scoring
// ════════════════════════════════════════════════════════════════════

#[test]
fn policy_working_decays_fastest() {
    let w = MemoryPolicy::default_for(MemoryKind::Working);
    let e = MemoryPolicy::default_for(MemoryKind::Episodic);
    let s = MemoryPolicy::default_for(MemoryKind::Semantic);
    let p = MemoryPolicy::default_for(MemoryKind::Procedural);
    assert!(w.decay_lambda_per_sec > e.decay_lambda_per_sec);
    assert!(w.decay_lambda_per_sec > s.decay_lambda_per_sec);
    assert!(w.decay_lambda_per_sec > p.decay_lambda_per_sec);
}

#[test]
fn policy_semantic_retains_most() {
    let s = MemoryPolicy::default_for(MemoryKind::Semantic);
    let w = MemoryPolicy::default_for(MemoryKind::Working);
    assert!(s.retention_threshold > w.retention_threshold);
}

#[test]
fn policy_score_is_zero_for_zero_relevance() {
    let p = MemoryPolicy::default_for(MemoryKind::Episodic);
    assert!((p.score(0.0, 1.0, 1.0)).abs() < 1e-9);
}

#[test]
fn policy_score_is_zero_for_zero_importance() {
    let p = MemoryPolicy::default_for(MemoryKind::Episodic);
    assert!((p.score(1.0, 1.0, 0.0)).abs() < 1e-9);
}

#[test]
fn policy_score_decays_with_age() {
    let p = MemoryPolicy::default_for(MemoryKind::Working);
    assert!(p.score(1.0, 0.0, 1.0) > p.score(1.0, 100.0, 1.0));
}

#[test]
fn policy_score_clamps_relevance_to_unit_interval() {
    let p = MemoryPolicy::default_for(MemoryKind::Working);
    // Above-1 relevance should not amplify beyond the clamp.
    assert!((p.score(2.0, 0.0, 1.0) - p.score(1.0, 0.0, 1.0)).abs() < 1e-9);
}

#[test]
fn policy_score_clamps_importance_to_unit_interval() {
    let p = MemoryPolicy::default_for(MemoryKind::Working);
    assert!((p.score(1.0, 0.0, 5.0) - p.score(1.0, 0.0, 1.0)).abs() < 1e-9);
}

#[test]
fn policy_should_retain_above_threshold() {
    let p = MemoryPolicy::default_for(MemoryKind::Working);
    assert!(p.should_retain(0.99));
}

#[test]
fn policy_should_evict_below_threshold() {
    let p = MemoryPolicy::default_for(MemoryKind::Working);
    assert!(!p.should_retain(0.0));
}

// ════════════════════════════════════════════════════════════════════
// MemoryHandle — typed wrapper
// ════════════════════════════════════════════════════════════════════

#[test]
fn memory_handle_carries_kind_and_default_policy() {
    let h: MemoryHandle<WorkingStore<i32>> = MemoryHandle::new("scratch", MemoryKind::Working);
    assert_eq!(h.name, "scratch");
    assert_eq!(h.kind, MemoryKind::Working);
    let default = MemoryPolicy::default_for(MemoryKind::Working);
    assert!((h.policy.decay_lambda_per_sec - default.decay_lambda_per_sec).abs() < 1e-12);
}

#[test]
fn memory_handle_works_with_episode_store() {
    let h: MemoryHandle<EpisodeStore<String>> =
        MemoryHandle::new("session_log", MemoryKind::Episodic);
    h.store.append("event1".to_string());
    h.store.append("event2".to_string());
    assert_eq!(h.store.len(), 2);
}

#[test]
fn memory_handle_works_with_vector_index() {
    let h: MemoryHandle<VectorIndex<i32>> = MemoryHandle::new("kb", MemoryKind::Semantic);
    h.store.insert(vec![1.0, 0.0], 1);
    h.store.insert(vec![0.0, 1.0], 2);
    let r = h.store.search(&[1.0, 0.0], 1);
    assert_eq!(r[0].1, 1);
}

#[test]
fn memory_handle_works_with_workflow_store() {
    let h: MemoryHandle<WorkflowStore<i32>> = MemoryHandle::new("wf", MemoryKind::Procedural);
    h.store.register("counter", 0);
    h.store.update("counter", |n| n + 1);
    let w = h.store.find("counter").unwrap();
    assert_eq!(*w.current().unwrap(), 1);
}
