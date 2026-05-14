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

#[test]
fn working_store_routes_pushes_through_kind_allocator() {
    let s: WorkingStore<i32> = WorkingStore::new();

    s.push(10);
    s.push(20);
    let stats = s.allocator_stats();

    assert_eq!(stats.kind, MemoryKind::Working);
    assert_eq!(stats.allocations, 2);
    assert_eq!(stats.allocated_items, 2);
    assert!(stats.bytes_reserved >= 2 * std::mem::size_of::<i32>());

    s.clear();
    assert_eq!(s.allocator_stats().resets, 1);
}

#[test]
fn all_reference_stores_route_allocations_to_their_kind_allocator() {
    let working: WorkingStore<i32> = WorkingStore::new();
    let episodic: EpisodeStore<i32> = EpisodeStore::new();
    let semantic: VectorIndex<i32> = VectorIndex::new();
    let procedural: WorkflowStore<i32> = WorkflowStore::new();

    working.push(1);
    episodic.append_at(1, 2);
    semantic.insert(vec![1.0, 0.0], 3);
    procedural.register("wf", 4);

    assert_eq!(working.allocator_stats().kind, MemoryKind::Working);
    assert_eq!(episodic.allocator_stats().kind, MemoryKind::Episodic);
    assert_eq!(semantic.allocator_stats().kind, MemoryKind::Semantic);
    assert_eq!(procedural.allocator_stats().kind, MemoryKind::Procedural);
    assert_eq!(working.allocator_stats().allocated_items, 1);
    assert_eq!(episodic.allocator_stats().allocated_items, 1);
    assert_eq!(semantic.allocator_stats().allocated_items, 1);
    assert_eq!(procedural.allocator_stats().allocated_items, 1);
}

#[test]
fn cycle_aware_allocators_record_store_roots_for_each_memory_kind() {
    let working_alloc = CycleAwareKindAllocator::shared(MemoryKind::Working, 8);
    let episodic_alloc = CycleAwareKindAllocator::shared(MemoryKind::Episodic, 8);
    let semantic_alloc = CycleAwareKindAllocator::shared(MemoryKind::Semantic, 8);
    let procedural_alloc = CycleAwareKindAllocator::shared(MemoryKind::Procedural, 8);
    let working: WorkingStore<i32> = WorkingStore::with_allocator(working_alloc);
    let episodic: EpisodeStore<i32> = EpisodeStore::with_allocator(episodic_alloc);
    let semantic: VectorIndex<i32> = VectorIndex::with_allocator(semantic_alloc);
    let procedural: WorkflowStore<i32> = WorkflowStore::with_allocator(procedural_alloc);

    working.push(1);
    episodic.append(2);
    semantic.insert(vec![1.0, 0.0], 3);
    procedural.register("wf", 4);

    assert_eq!(working.allocator_root_stats().roots_created, 1);
    assert_eq!(episodic.allocator_root_stats().roots_created, 1);
    assert_eq!(semantic.allocator_root_stats().roots_created, 1);
    assert_eq!(procedural.allocator_root_stats().roots_created, 1);
    assert_eq!(working.allocator_root_stats().active_roots, 1);
    assert_eq!(episodic.allocator_root_stats().active_roots, 1);
    assert_eq!(semantic.allocator_root_stats().active_roots, 1);
    assert_eq!(procedural.allocator_root_stats().active_roots, 1);
}

#[test]
fn default_stores_route_roots_to_cycle_aware_allocator_backend() {
    let working: WorkingStore<i32> = WorkingStore::new();
    let episodic: EpisodeStore<i32> = EpisodeStore::new();
    let semantic: VectorIndex<i32> = VectorIndex::new();
    let procedural: WorkflowStore<i32> = WorkflowStore::new();

    working.push(1);
    episodic.append(2);
    semantic.insert(vec![1.0, 0.0], 3);
    procedural.register("wf", 4);

    assert_eq!(working.allocator_root_stats().roots_created, 1);
    assert_eq!(working.allocator_root_stats().active_roots, 1);
    assert_eq!(episodic.allocator_root_stats().roots_created, 1);
    assert_eq!(episodic.allocator_root_stats().active_roots, 1);
    assert_eq!(semantic.allocator_root_stats().roots_created, 1);
    assert_eq!(semantic.allocator_root_stats().active_roots, 1);
    assert_eq!(procedural.allocator_root_stats().roots_created, 1);
    assert_eq!(procedural.allocator_root_stats().active_roots, 1);
}

#[test]
fn working_store_clear_releases_cycle_aware_roots() {
    let alloc = CycleAwareKindAllocator::shared(MemoryKind::Working, 8);
    let s = WorkingStore::with_allocator(alloc);

    s.push(10);
    s.push(20);
    assert_eq!(s.allocator_root_stats().active_roots, 2);

    s.clear();
    let stats = s.allocator_root_stats();

    assert_eq!(stats.active_roots, 0);
    assert_eq!(stats.roots_released, 2);
    assert_eq!(stats.buffered_roots, 0);
}

#[test]
fn cycle_aware_allocator_surface_reports_root_finalization_without_collecting_safe_allocations() {
    let alloc = CycleAwareKindAllocator::new(MemoryKind::Working, 1);
    let root = alloc.retain_root("root").unwrap();
    let child = alloc.allocate_arc("child");
    let safe = alloc.allocate_safe("safe_affine");

    alloc.add_edge(root, child).unwrap();
    alloc.add_edge(child, root).unwrap();
    alloc.add_edge(safe, safe).unwrap();

    let report = alloc
        .release_root(root)
        .expect("root release should report collection");

    assert_eq!(
        report
            .finalization_order
            .iter()
            .map(|id| alloc.label(*id).unwrap())
            .collect::<Vec<_>>(),
        vec!["child", "root"]
    );
    assert_eq!(alloc.root_stats().collected_roots, 2);
    assert!(alloc.contains(safe));
    assert_eq!(
        alloc.allocation_mode(safe),
        Some(CycleAllocationMode::SafeAffine)
    );
}

#[test]
fn episodic_policy_eviction_releases_cycle_aware_roots() {
    let mut policy = MemoryPolicy::default_for(MemoryKind::Episodic);
    policy.compaction_high_water = 2;
    policy.retention_threshold = 0.0;
    let alloc = CycleAwareKindAllocator::shared(MemoryKind::Episodic, 8);
    let s = EpisodeStore::with_policy_and_allocator(policy, alloc);

    for i in 0..5 {
        s.append_at(i, i as i32);
    }
    assert_eq!(s.allocator_root_stats().active_roots, 5);

    let values: Vec<_> = s
        .recent(10)
        .into_iter()
        .map(|episode| episode.value)
        .collect();
    let stats = s.allocator_root_stats();

    assert_eq!(values, vec![3, 4]);
    assert_eq!(stats.active_roots, 2);
    assert_eq!(stats.roots_released, 3);
    assert_eq!(stats.buffered_roots, 0);
}

#[test]
fn semantic_policy_eviction_releases_cycle_aware_roots() {
    let mut policy = MemoryPolicy::default_for(MemoryKind::Semantic);
    policy.compaction_high_water = 1;
    policy.retention_threshold = 0.0;
    let alloc = CycleAwareKindAllocator::shared(MemoryKind::Semantic, 8);
    let idx: VectorIndex<&str> = VectorIndex::with_policy_and_allocator(policy, alloc);

    idx.insert(vec![1.0, 0.0], "x");
    idx.insert(vec![0.0, 1.0], "y");
    idx.insert(vec![0.5, 0.5], "mid");
    assert_eq!(idx.allocator_root_stats().active_roots, 3);

    let values: Vec<_> = idx
        .search(&[1.0, 0.0], 3)
        .into_iter()
        .map(|(_score, value)| value)
        .collect();
    let stats = idx.allocator_root_stats();

    assert_eq!(values, vec!["x"]);
    assert_eq!(stats.active_roots, 1);
    assert_eq!(stats.roots_released, 2);
    assert_eq!(stats.buffered_roots, 0);
}

#[test]
fn procedural_register_replacement_releases_previous_cycle_aware_root() {
    let alloc = CycleAwareKindAllocator::shared(MemoryKind::Procedural, 8);
    let ws = WorkflowStore::with_allocator(alloc);

    ws.register("build", 1);
    ws.register("build", 2);
    let stats = ws.allocator_root_stats();

    assert_eq!(stats.roots_created, 2);
    assert_eq!(stats.active_roots, 1);
    assert_eq!(stats.roots_released, 1);
    assert_eq!(stats.buffered_roots, 0);
}

#[test]
fn dropping_stores_releases_cycle_aware_roots() {
    let working_alloc = CycleAwareKindAllocator::shared(MemoryKind::Working, 8);
    let episodic_alloc = CycleAwareKindAllocator::shared(MemoryKind::Episodic, 8);
    let semantic_alloc = CycleAwareKindAllocator::shared(MemoryKind::Semantic, 8);
    let procedural_alloc = CycleAwareKindAllocator::shared(MemoryKind::Procedural, 8);

    {
        let working: WorkingStore<i32> = WorkingStore::with_allocator(working_alloc.clone());
        let episodic: EpisodeStore<i32> = EpisodeStore::with_allocator(episodic_alloc.clone());
        let semantic: VectorIndex<i32> = VectorIndex::with_allocator(semantic_alloc.clone());
        let procedural: WorkflowStore<i32> =
            WorkflowStore::with_allocator(procedural_alloc.clone());

        working.push(1);
        episodic.append(2);
        semantic.insert(vec![1.0, 0.0], 3);
        procedural.register("wf", 4);
    }

    assert_eq!(working_alloc.root_stats().active_roots, 0);
    assert_eq!(episodic_alloc.root_stats().active_roots, 0);
    assert_eq!(semantic_alloc.root_stats().active_roots, 0);
    assert_eq!(procedural_alloc.root_stats().active_roots, 0);
    assert_eq!(working_alloc.root_stats().roots_released, 1);
    assert_eq!(episodic_alloc.root_stats().roots_released, 1);
    assert_eq!(semantic_alloc.root_stats().roots_released, 1);
    assert_eq!(procedural_alloc.root_stats().roots_released, 1);
    assert_eq!(working_alloc.root_stats().buffered_roots, 0);
    assert_eq!(episodic_alloc.root_stats().buffered_roots, 0);
    assert_eq!(semantic_alloc.root_stats().buffered_roots, 0);
    assert_eq!(procedural_alloc.root_stats().buffered_roots, 0);
}

#[test]
fn cycle_aware_allocator_boundary_methods_flush_all_store_root_buffers() {
    let mut episodic_policy = MemoryPolicy::default_for(MemoryKind::Episodic);
    episodic_policy.compaction_high_water = 1;
    episodic_policy.retention_threshold = 0.0;
    let mut semantic_policy = MemoryPolicy::default_for(MemoryKind::Semantic);
    semantic_policy.compaction_high_water = 1;
    semantic_policy.retention_threshold = 0.0;

    let working_alloc = CycleAwareKindAllocator::shared(MemoryKind::Working, 8);
    let episodic_alloc = CycleAwareKindAllocator::shared(MemoryKind::Episodic, 8);
    let semantic_alloc = CycleAwareKindAllocator::shared(MemoryKind::Semantic, 8);
    let procedural_alloc = CycleAwareKindAllocator::shared(MemoryKind::Procedural, 8);

    let working: WorkingStore<i32> = WorkingStore::with_allocator(working_alloc.clone());
    let episodic: EpisodeStore<i32> =
        EpisodeStore::with_policy_and_allocator(episodic_policy, episodic_alloc.clone());
    let semantic: VectorIndex<i32> =
        VectorIndex::with_policy_and_allocator(semantic_policy, semantic_alloc.clone());
    let procedural: WorkflowStore<i32> = WorkflowStore::with_allocator(procedural_alloc.clone());

    working.push(1);
    working.push(2);
    episodic.append(10);
    episodic.append(20);
    episodic.append(30);
    episodic.append(40);
    semantic.insert(vec![1.0, 0.0], 1);
    semantic.insert(vec![0.0, 1.0], 2);
    semantic.insert(vec![0.5, 0.5], 3);
    procedural.register("workflow", 1);
    procedural.register("workflow", 2);

    let _ = episodic.recent(1);
    let _ = semantic.search(&[1.0, 0.0], 1);

    working.clear();
    procedural.find("workflow");

    {
        let working_stats = working.allocator_root_stats();
        assert_eq!(working_stats.active_roots, 0);
        assert_eq!(working_stats.buffered_roots, 0);
        assert_eq!(working_stats.roots_released, 2);
    }
    {
        let episodic_stats = episodic.allocator_root_stats();
        assert_eq!(episodic_stats.active_roots, 1);
        assert_eq!(episodic_stats.buffered_roots, 0);
        assert_eq!(episodic_stats.roots_released, 3);
    }
    {
        let semantic_stats = semantic.allocator_root_stats();
        assert_eq!(semantic_stats.active_roots, 1);
        assert_eq!(semantic_stats.buffered_roots, 0);
        assert_eq!(semantic_stats.roots_released, 2);
    }
    {
        let procedural_stats = procedural.allocator_root_stats();
        assert_eq!(procedural_stats.active_roots, 1);
        assert_eq!(procedural_stats.buffered_roots, 0);
        assert_eq!(procedural_stats.roots_released, 1);
    }

    drop(working);
    drop(episodic);
    drop(semantic);
    drop(procedural);

    assert_eq!(working_alloc.root_stats().active_roots, 0);
    assert_eq!(episodic_alloc.root_stats().active_roots, 0);
    assert_eq!(semantic_alloc.root_stats().active_roots, 0);
    assert_eq!(procedural_alloc.root_stats().active_roots, 0);
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
    assert_eq!(s.allocator_stats().allocated_items, 3);
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

#[test]
fn episode_store_lazy_eviction_enforces_policy_cap_on_reads() {
    let mut policy = MemoryPolicy::default_for(MemoryKind::Episodic);
    policy.compaction_high_water = 3;
    policy.retention_threshold = 0.0;
    let s: EpisodeStore<i32> = EpisodeStore::with_policy(policy);

    for i in 0..10 {
        s.append_at(i, i as i32);
    }

    let recent = s.recent(10);
    let values: Vec<_> = recent.into_iter().map(|episode| episode.value).collect();

    assert_eq!(values, vec![7, 8, 9]);
    assert_eq!(s.len(), 3);
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

#[test]
fn vector_index_lazy_eviction_keeps_top_policy_matches() {
    let mut policy = MemoryPolicy::default_for(MemoryKind::Semantic);
    policy.compaction_high_water = 2;
    policy.retention_threshold = 0.0;
    let idx: VectorIndex<&str> = VectorIndex::with_policy(policy);

    idx.insert(vec![1.0, 0.0], "x");
    idx.insert(vec![0.95, 0.05], "near_x");
    idx.insert(vec![0.0, 1.0], "y");
    idx.insert(vec![0.0, 0.0], "zero");

    let r = idx.search(&[1.0, 0.0], 4);
    let values: Vec<_> = r.into_iter().map(|(_score, value)| value).collect();

    assert_eq!(values, vec!["x", "near_x"]);
    assert_eq!(idx.len(), 2);
}

#[test]
fn vector_index_lazy_eviction_drops_low_relevance_matches() {
    let mut policy = MemoryPolicy::default_for(MemoryKind::Semantic);
    policy.retention_threshold = 0.5;
    let idx: VectorIndex<&str> = VectorIndex::with_policy(policy);

    idx.insert(vec![1.0, 0.0], "x");
    idx.insert(vec![0.0, 1.0], "y");

    let r = idx.search(&[1.0, 0.0], 2);
    let values: Vec<_> = r.into_iter().map(|(_score, value)| value).collect();

    assert_eq!(values, vec!["x"]);
    assert_eq!(idx.len(), 1);
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

#[test]
fn cycle_aware_allocator_edge_removal_collection_is_deterministic_across_kinds() {
    // Threshold-crossing edge removal at the allocator wrapper layer must
    // produce consistent collection behavior for every MemoryKind, with
    // root_stats invariants preserved across kinds.
    let kinds = [
        MemoryKind::Working,
        MemoryKind::Episodic,
        MemoryKind::Semantic,
        MemoryKind::Procedural,
    ];

    for kind in kinds {
        let allocator = CycleAwareKindAllocator::new(kind, 1);
        let root = allocator
            .retain_root("root")
            .unwrap_or_else(|| panic!("retain_root for {:?}", kind));
        let cycle_a = allocator.allocate_arc("cycle_a");
        let cycle_b = allocator.allocate_arc("cycle_b");

        allocator.add_edge(root, cycle_a).unwrap();
        allocator.add_edge(cycle_a, cycle_b).unwrap();
        allocator.add_edge(cycle_b, cycle_a).unwrap();

        let report = allocator
            .remove_edge(root, cycle_a)
            .unwrap_or_else(|err| panic!("remove_edge for {:?}: {:?}", kind, err))
            .unwrap_or_else(|| {
                panic!(
                    "threshold-crossing remove_edge should collect for {:?}",
                    kind
                )
            });

        assert_eq!(report.collected.len(), 2, "collected count for {:?}", kind);
        assert_eq!(
            report.finalization_order.len(),
            2,
            "finalization_order length for {:?}",
            kind
        );
        assert!(allocator.contains(root), "root retained for {:?}", kind);
        assert!(
            !allocator.contains(cycle_a),
            "cycle_a collected for {:?}",
            kind
        );
        assert!(
            !allocator.contains(cycle_b),
            "cycle_b collected for {:?}",
            kind
        );

        let stats = allocator.root_stats();
        assert_eq!(stats.roots_created, 1, "roots_created for {:?}", kind);
        assert_eq!(stats.active_roots, 1, "active_roots for {:?}", kind);
        assert_eq!(stats.buffered_roots, 0, "buffered_roots for {:?}", kind);
        assert_eq!(stats.collected_roots, 2, "collected_roots for {:?}", kind);
        assert_eq!(
            stats.active_roots,
            stats.roots_created.saturating_sub(stats.roots_released),
            "active_roots invariant for {:?}",
            kind
        );
    }
}

#[test]
fn cycle_aware_allocator_collect_roots_drains_buffered_candidates() {
    let allocator = CycleAwareKindAllocator::new(MemoryKind::Working, 4);
    let root = allocator
        .retain_root("root")
        .expect("retain_root should allocate a managed root");
    let cycle_a = allocator.allocate_arc("cycle_a");
    let cycle_b = allocator.allocate_arc("cycle_b");
    let safe = allocator.allocate_safe("safe_affine");

    allocator
        .add_edge(root, cycle_a)
        .expect("add_edge should link root -> cycle_a");
    allocator
        .add_edge(cycle_a, cycle_b)
        .expect("add_edge should link cycle_a -> cycle_b");
    allocator
        .add_edge(cycle_b, cycle_a)
        .expect("add_edge should close the cycle");
    allocator
        .add_edge(cycle_b, root)
        .expect("add_edge should keep root in the cycle");
    allocator
        .add_edge(safe, safe)
        .expect("safe-affine self-edge should be tracked");

    assert_eq!(allocator.release_root(root), None);

    let buffered = allocator.buffered_roots();
    assert_eq!(buffered, vec![root]);

    let collect_report = allocator
        .collect_roots()
        .expect("collect_roots should drain buffered candidates");

    assert_eq!(collect_report.trial_candidates, vec![root]);
    assert_eq!(
        collect_report.finalization_order,
        vec![cycle_b, cycle_a, root]
    );
    assert_eq!(collect_report.collected, vec![root, cycle_a, cycle_b]);

    assert!(allocator.buffered_roots().is_empty());
    assert!(!allocator.contains(cycle_a));
    assert!(!allocator.contains(cycle_b));
    assert!(!allocator.contains(root));
    assert!(allocator.contains(safe));
    assert_eq!(
        allocator.allocation_mode(safe),
        Some(CycleAllocationMode::SafeAffine)
    );

    let stats = allocator.root_stats();
    assert_eq!(stats.buffered_roots, 0);
    assert_eq!(stats.collected_roots, 3);
    assert_eq!(stats.active_roots, 0);
}
