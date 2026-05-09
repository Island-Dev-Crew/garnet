//! Persistence hardening tests for Mnemos stores.

use garnet_memory::{
    CycleAwareKindAllocator, EpisodePersistenceError, EpisodeStore, MemoryKind,
    EPISODIC_TEXT_LOG_MAX_BYTES,
};
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

#[test]
fn episodic_text_append_extends_log_and_live_store() {
    let path = temp_file("append-episodes.mnemos");
    let store: EpisodeStore<String> = EpisodeStore::new();

    store
        .append_text(&path, 11, "first".to_string())
        .expect("append first persisted episode");
    store
        .append_text(&path, 12, "second\twith\ncontrols".to_string())
        .expect("append second persisted episode");

    let live_values: Vec<_> = store
        .snapshot()
        .into_iter()
        .map(|episode| episode.value)
        .collect();
    assert_eq!(
        live_values,
        vec!["first".to_string(), "second\twith\ncontrols".to_string()]
    );

    let recovered: EpisodeStore<String> = EpisodeStore::new();
    recovered
        .load_text(&path)
        .expect("load append-style text log");
    let recovered_values: Vec<_> = recovered
        .snapshot()
        .into_iter()
        .map(|episode| episode.value)
        .collect();
    assert_eq!(recovered_values, live_values);
}

#[test]
fn episodic_text_append_rejects_corrupt_existing_log_without_mutating_store() {
    let path = temp_file("corrupt-append-episodes.mnemos");
    let original_log = "garnet-episodic-v1\nbroken-line\n";
    std::fs::write(&path, original_log).expect("write corrupt append log");

    let store: EpisodeStore<String> = EpisodeStore::new();
    store.append_at(9, "keep".to_string());

    let result = store.append_text(&path, 10, "new".to_string());

    assert!(result.is_err());
    let snapshot = store.snapshot();
    assert_eq!(snapshot.len(), 1);
    assert_eq!(snapshot[0].timestamp_unix, 9);
    assert_eq!(snapshot[0].value, "keep");
    assert_eq!(
        std::fs::read_to_string(&path).expect("read corrupt append log"),
        original_log
    );
}

#[test]
fn episodic_text_append_rejects_empty_existing_log_without_mutating_store() {
    let path = temp_file("empty-append-episodes.mnemos");
    std::fs::write(&path, "").expect("write empty append log");

    let store: EpisodeStore<String> = EpisodeStore::new();
    store.append_at(9, "keep".to_string());

    let result = store.append_text(&path, 10, "new".to_string());

    assert!(matches!(
        result,
        Err(EpisodePersistenceError::MissingHeader)
    ));
    assert_eq!(std::fs::read_to_string(&path).expect("read empty log"), "");
    let snapshot = store.snapshot();
    assert_eq!(snapshot.len(), 1);
    assert_eq!(snapshot[0].timestamp_unix, 9);
    assert_eq!(snapshot[0].value, "keep");
}

#[test]
fn episodic_text_append_validates_existing_values_against_store_type() {
    let path = temp_file("typed-append-episodes.mnemos");
    let original_log = "garnet-episodic-v1\n1\t616263\n";
    std::fs::write(&path, original_log).expect("write typed-invalid append log");

    let store: EpisodeStore<u64> = EpisodeStore::new();
    store.append_at(9, 7);

    let result = store.append_text(&path, 10, 42);

    assert!(matches!(
        result,
        Err(EpisodePersistenceError::InvalidValue {
            line: 2,
            value,
            ..
        }) if value == "abc"
    ));
    assert_eq!(
        std::fs::read_to_string(&path).expect("read typed-invalid log"),
        original_log
    );
    let snapshot = store.snapshot();
    assert_eq!(snapshot.len(), 1);
    assert_eq!(snapshot[0].timestamp_unix, 9);
    assert_eq!(snapshot[0].value, 7);
}

#[test]
fn episodic_text_append_rejects_oversized_existing_log_without_mutating_store() {
    let path = temp_file("oversized-append-episodes.mnemos");
    let payload_hex_len = (EPISODIC_TEXT_LOG_MAX_BYTES as usize) + 2;
    let oversized_log = format!(
        "garnet-episodic-v1\n1\t{}\n",
        "61".repeat(payload_hex_len / 2)
    );
    std::fs::write(&path, oversized_log).expect("write oversized append log");

    let store: EpisodeStore<String> = EpisodeStore::new();
    store.append_at(9, "keep".to_string());

    let result = store.append_text(&path, 10, "new".to_string());

    assert!(matches!(
        result,
        Err(EpisodePersistenceError::LogTooLarge { .. })
    ));
    let snapshot = store.snapshot();
    assert_eq!(snapshot.len(), 1);
    assert_eq!(snapshot[0].timestamp_unix, 9);
    assert_eq!(snapshot[0].value, "keep");
}

#[test]
fn episodic_text_append_rejects_record_that_would_exceed_log_limit() {
    let path = temp_file("record-too-large-episodes.mnemos");
    let store: EpisodeStore<String> = EpisodeStore::new();
    store.append_at(9, "keep".to_string());
    let oversized_value = "x".repeat((EPISODIC_TEXT_LOG_MAX_BYTES as usize / 2) + 1);

    let result = store.append_text(&path, 10, oversized_value);

    assert!(matches!(
        result,
        Err(EpisodePersistenceError::LogTooLarge { .. })
    ));
    assert!(!path.exists());
    let snapshot = store.snapshot();
    assert_eq!(snapshot.len(), 1);
    assert_eq!(snapshot[0].timestamp_unix, 9);
    assert_eq!(snapshot[0].value, "keep");
}
