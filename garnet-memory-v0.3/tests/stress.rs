//! Stress tests for the memory primitives. `#[ignore]` by default so the
//! default `cargo test` stays fast; run with `cargo test --workspace -- --ignored`.

use garnet_memory::*;

#[test]
#[ignore]
fn vector_index_100k_top10() {
    let idx: VectorIndex<usize> = VectorIndex::new();
    for i in 0..100_000usize {
        let frac = i as f32 / 100_000.0;
        idx.insert(vec![frac, 1.0 - frac], i);
    }
    let r = idx.search(&[0.5, 0.5], 10);
    assert_eq!(r.len(), 10);
}

#[test]
#[ignore]
fn episode_store_one_million_appends() {
    let s: EpisodeStore<u32> = EpisodeStore::new();
    for i in 0..1_000_000u32 {
        s.append_at(i as u64, i);
    }
    assert_eq!(s.len(), 1_000_000);
    let recent = s.recent(50);
    assert_eq!(recent.len(), 50);
    assert_eq!(recent[49].value, 999_999);
}

#[test]
#[ignore]
fn working_store_50k_pushes_then_clear() {
    let s: WorkingStore<u32> = WorkingStore::new();
    for i in 0..50_000u32 {
        s.push(i);
    }
    assert_eq!(s.len(), 50_000);
    s.clear();
    assert!(s.is_empty());
}

#[test]
#[ignore]
fn workflow_store_thousand_versions() {
    let ws: WorkflowStore<u32> = WorkflowStore::new();
    ws.register("counter", 0);
    for _ in 0..1000 {
        ws.update("counter", |n| n + 1);
    }
    let w = ws.find("counter").unwrap();
    assert_eq!(w.versions.len(), 1001);
    assert_eq!(*w.current().unwrap(), 1000);
    let v500 = ws.replay("counter", 500).unwrap();
    assert_eq!(v500, 500);
}
