//! Property-based tests for the four memory primitives.

use garnet_memory::*;
use proptest::prelude::*;

// ── WorkingStore: push then len equals the count of pushes ─────────

proptest! {
    #[test]
    fn working_store_len_equals_push_count(items in proptest::collection::vec(0i32..1000, 0..200)) {
        let s: WorkingStore<i32> = WorkingStore::new();
        for v in &items {
            s.push(*v);
        }
        prop_assert_eq!(s.len(), items.len());
    }
}

proptest! {
    #[test]
    fn working_store_clear_idempotent(items in proptest::collection::vec(0i32..1000, 0..50)) {
        let s: WorkingStore<i32> = WorkingStore::new();
        for v in items {
            s.push(v);
        }
        s.clear();
        s.clear();
        s.clear();
        prop_assert_eq!(s.len(), 0);
        prop_assert!(s.is_empty());
    }
}

proptest! {
    #[test]
    fn working_store_push_returns_dense_indices(items in proptest::collection::vec(0i32..1000, 0..100)) {
        let s: WorkingStore<i32> = WorkingStore::new();
        for (i, v) in items.iter().enumerate() {
            let idx = s.push(*v);
            prop_assert_eq!(idx, i);
        }
    }
}

// ── EpisodeStore: recent(N) returns last N in order ────────────────

proptest! {
    #[test]
    fn episode_recent_n_equals_tail(items in proptest::collection::vec(0i32..1000, 0..200), n in 0usize..200) {
        let s: EpisodeStore<i32> = EpisodeStore::new();
        for (i, v) in items.iter().enumerate() {
            s.append_at(i as u64, *v);
        }
        let recent = s.recent(n);
        let expected = if items.len() < n { items.len() } else { n };
        prop_assert_eq!(recent.len(), expected);
        // Returned items must be the LAST `expected` items in order.
        let tail = &items[items.len().saturating_sub(expected)..];
        for (i, ep) in recent.iter().enumerate() {
            prop_assert_eq!(ep.value, tail[i]);
        }
    }
}

proptest! {
    #[test]
    fn episode_since_filters_by_timestamp(
        events in proptest::collection::vec((0u64..10_000, 0i32..1000), 0..50),
        threshold in 0u64..10_000,
    ) {
        let s: EpisodeStore<i32> = EpisodeStore::new();
        for (ts, v) in &events {
            s.append_at(*ts, *v);
        }
        let since = s.since(threshold);
        prop_assert!(since.iter().all(|e| e.timestamp_unix >= threshold));
    }
}

// ── VectorIndex: top-k ordering invariant ───────────────────────────

proptest! {
    #[test]
    fn vector_index_top_k_returns_at_most_k(
        vectors in proptest::collection::vec(proptest::collection::vec(-10.0f32..10.0, 4..8), 0..30),
        k in 0usize..50,
    ) {
        let idx: VectorIndex<usize> = VectorIndex::new();
        let dim = if let Some(first) = vectors.first() { first.len() } else { 4 };
        let aligned: Vec<Vec<f32>> = vectors
            .into_iter()
            .map(|mut v| {
                v.resize(dim, 0.0);
                v
            })
            .collect();
        for (i, v) in aligned.iter().enumerate() {
            idx.insert(v.clone(), i);
        }
        let query = vec![1.0; dim];
        let r = idx.search(&query, k);
        prop_assert!(r.len() <= k);
        prop_assert!(r.len() <= aligned.len());
    }
}

proptest! {
    #[test]
    fn vector_index_results_are_sorted_descending(
        vectors in proptest::collection::vec(proptest::collection::vec(-1.0f32..1.0, 4..6), 1..30),
    ) {
        let idx: VectorIndex<usize> = VectorIndex::new();
        let dim = vectors[0].len();
        let aligned: Vec<Vec<f32>> = vectors.into_iter().map(|mut v| { v.resize(dim, 0.0); v }).collect();
        for (i, v) in aligned.iter().enumerate() {
            idx.insert(v.clone(), i);
        }
        let query = vec![0.5; dim];
        let r = idx.search(&query, aligned.len());
        for w in r.windows(2) {
            prop_assert!(w[0].0 >= w[1].0, "results must be sorted descending by score");
        }
    }
}

// ── WorkflowStore: register then find returns latest ────────────────

proptest! {
    #[test]
    fn workflow_store_find_returns_latest_after_updates(updates in 1usize..20) {
        let ws: WorkflowStore<i32> = WorkflowStore::new();
        ws.register("counter", 0);
        for _ in 0..updates {
            ws.update("counter", |n| n + 1);
        }
        let w = ws.find("counter").unwrap();
        prop_assert_eq!(*w.current().unwrap(), updates as i32);
    }
}

proptest! {
    #[test]
    fn workflow_store_replay_returns_correct_version(updates in 1usize..15, version in 0usize..15) {
        let ws: WorkflowStore<i32> = WorkflowStore::new();
        ws.register("c", 0);
        for _ in 0..updates {
            ws.update("c", |n| n + 1);
        }
        let r = ws.replay("c", version);
        if version > updates {
            prop_assert!(r.is_none());
        } else {
            prop_assert_eq!(r.unwrap(), version as i32);
        }
    }
}

// ── MemoryPolicy: R+R+I total over the kind set ─────────────────────

proptest! {
    #[test]
    fn memory_policy_score_in_unit_interval(
        relevance in 0.0f64..1.0, age in 0.0f64..86_400.0, importance in 0.0f64..1.0
    ) {
        for kind in [MemoryKind::Working, MemoryKind::Episodic, MemoryKind::Semantic, MemoryKind::Procedural] {
            let p = MemoryPolicy::default_for(kind);
            let s = p.score(relevance, age, importance);
            prop_assert!((0.0..=1.0).contains(&s), "score out of [0,1]: {s}");
        }
    }
}

proptest! {
    #[test]
    fn memory_policy_score_monotone_in_age(
        relevance in 0.1f64..1.0, age in 1.0f64..1000.0, delta in 1.0f64..100.0,
        importance in 0.1f64..1.0
    ) {
        let p = MemoryPolicy::default_for(MemoryKind::Working);
        let young = p.score(relevance, age, importance);
        let older = p.score(relevance, age + delta, importance);
        prop_assert!(young >= older, "score must not increase with age: young={young}, older={older}");
    }
}
