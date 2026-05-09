//! Persistence hardening tests for Mnemos stores.

use garnet_memory::{CycleAwareKindAllocator, EpisodeStore, MemoryKind};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_file(name: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time after epoch")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!(
        "garnet-memory-persistence-{}-{nonce}",
        std::process::id()
    ));
    std::fs::create_dir_all(&dir).expect("create temp persistence dir");
    dir.join(name)
}

#[test]
fn episodic_text_persistence_round_trips_and_rehydrates_cycle_roots() {
    let path = temp_file("episodes.mnemos");
    let alloc = CycleAwareKindAllocator::shared(MemoryKind::Episodic, 8);

    {
        let store: EpisodeStore<String> = EpisodeStore::with_allocator(Arc::clone(&alloc));
        store.append_at(10, "alpha".to_string());
        store.append_at(20, "beta\twith\ncontrols".to_string());

        store.save_text(&path).expect("persist episodes");
        assert_eq!(store.allocator_root_stats().active_roots, 2);
    }

    assert_eq!(alloc.root_stats().active_roots, 0);
    assert_eq!(alloc.root_stats().roots_released, 2);

    let recovered: EpisodeStore<String> = EpisodeStore::with_allocator(Arc::clone(&alloc));
    recovered.load_text(&path).expect("recover episodes");

    let snapshot = recovered.snapshot();
    let timestamps: Vec<_> = snapshot
        .iter()
        .map(|episode| episode.timestamp_unix)
        .collect();
    let values: Vec<_> = snapshot.into_iter().map(|episode| episode.value).collect();

    assert_eq!(timestamps, vec![10, 20]);
    assert_eq!(
        values,
        vec!["alpha".to_string(), "beta\twith\ncontrols".to_string()]
    );
    assert_eq!(recovered.allocator_root_stats().active_roots, 2);
    assert_eq!(recovered.allocator_root_stats().roots_created, 4);
}

#[test]
fn episodic_text_persistence_rejects_malformed_files_without_mutating_store() {
    let path = temp_file("bad-episodes.mnemos");
    std::fs::write(&path, "garnet-episodic-v1\nnot-a-timestamp\t616c706861\n")
        .expect("write malformed persistence file");

    let store: EpisodeStore<String> = EpisodeStore::new();
    store.append_at(7, "keep".to_string());

    let result = store.load_text(&path);

    assert!(result.is_err());
    let snapshot = store.snapshot();
    assert_eq!(snapshot.len(), 1);
    assert_eq!(snapshot[0].timestamp_unix, 7);
    assert_eq!(snapshot[0].value, "keep");
}
