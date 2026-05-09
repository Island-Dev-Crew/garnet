//! Persistence hardening tests for Mnemos stores.

use garnet_memory::{
    episodic_cache_log_path_for, CycleAwareKindAllocator, EpisodePersistenceError, EpisodeStore,
    MemoryKind, EPISODIC_CACHE_DIR, EPISODIC_CACHE_EPISODIC_DIR, EPISODIC_CACHE_LOG_FILE,
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

fn temp_dir(name: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time after epoch")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!(
        "garnet-memory-{name}-{}-{nonce}",
        std::process::id()
    ));
    std::fs::create_dir_all(&dir).expect("create temp persistence dir");
    dir
}

fn cache_header_for_project(project: &PathBuf) -> String {
    let store: EpisodeStore<String> = EpisodeStore::new();
    store
        .append_cache_text(project, 1, "bootstrap".to_string())
        .expect("bootstrap cache header");
    let path = episodic_cache_log_path_for(project).expect("construct backend path");
    let raw = std::fs::read_to_string(&path).expect("read bootstrap cache");
    let mut lines = raw.lines();
    let header = lines.next().expect("cache format header");
    let source_tree = lines.next().expect("cache source-tree binding");
    format!("{header}\n{source_tree}\n")
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

#[test]
fn episodic_text_load_rejects_oversized_file_without_mutating_store() {
    let path = temp_file("oversized-load-episodes.mnemos");
    let oversized_log = "x".repeat((EPISODIC_TEXT_LOG_MAX_BYTES as usize) + 1);
    std::fs::write(&path, oversized_log).expect("write oversized load log");

    let store: EpisodeStore<String> = EpisodeStore::new();
    store.append_at(9, "keep".to_string());

    let result = store.load_text(&path);

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
fn episodic_cache_path_uses_fixed_garnet_cache_episodic_log() {
    let project = temp_dir("cache-path");

    let path = episodic_cache_log_path_for(&project).expect("construct backend path");

    assert_eq!(
        path,
        std::fs::canonicalize(&project)
            .expect("canonical project path")
            .join(EPISODIC_CACHE_DIR)
            .join(EPISODIC_CACHE_EPISODIC_DIR)
            .join(EPISODIC_CACHE_LOG_FILE)
    );
}

#[test]
fn episodic_cache_append_and_load_round_trips_under_default_cache_dir() {
    let project = temp_dir("cache-roundtrip");
    let path = episodic_cache_log_path_for(&project).expect("construct backend path");
    let store: EpisodeStore<String> = EpisodeStore::new();

    store
        .append_cache_text(&project, 11, "first".to_string())
        .expect("append first backend episode");
    store
        .append_cache_text(&project, 12, "second\twith\ncontrols".to_string())
        .expect("append second backend episode");

    assert!(path.exists());
    let raw = std::fs::read_to_string(&path).expect("read backend log");
    assert!(
        raw.lines()
            .nth(1)
            .is_some_and(|line| line.starts_with("source-tree\t")),
        "typed backend log must carry a source-tree binding"
    );
    let recovered: EpisodeStore<String> = EpisodeStore::new();
    recovered
        .load_cache_text(&project)
        .expect("load backend episodes");

    let values: Vec<_> = recovered
        .snapshot()
        .into_iter()
        .map(|episode| episode.value)
        .collect();
    assert_eq!(
        values,
        vec!["first".to_string(), "second\twith\ncontrols".to_string()]
    );
}

#[test]
fn episodic_cache_load_rehydrates_cycle_roots() {
    let project = temp_dir("cache-roots");
    let alloc = CycleAwareKindAllocator::shared(MemoryKind::Episodic, 8);

    {
        let store: EpisodeStore<String> = EpisodeStore::with_allocator(Arc::clone(&alloc));
        store
            .append_cache_text(&project, 11, "rooted".to_string())
            .expect("append backend episode");
        assert_eq!(store.allocator_root_stats().active_roots, 1);
    }

    assert_eq!(alloc.root_stats().active_roots, 0);
    let recovered: EpisodeStore<String> = EpisodeStore::with_allocator(Arc::clone(&alloc));
    recovered
        .load_cache_text(&project)
        .expect("load backend episodes");

    assert_eq!(recovered.allocator_root_stats().active_roots, 1);
    assert_eq!(recovered.snapshot()[0].value, "rooted");
}

#[test]
fn episodic_cache_rejects_corrupt_backend_without_mutating_store() {
    let project = temp_dir("cache-corrupt");
    let path = episodic_cache_log_path_for(&project).expect("construct backend path");
    std::fs::create_dir_all(path.parent().expect("cache log has parent"))
        .expect("create backend dir");
    let original_log = format!("{}broken-line\n", cache_header_for_project(&project));
    std::fs::write(&path, &original_log).expect("write corrupt backend log");

    let store: EpisodeStore<String> = EpisodeStore::new();
    store.append_at(9, "keep".to_string());

    let result = store.append_cache_text(&project, 10, "new".to_string());

    assert!(result.is_err());
    assert_eq!(
        std::fs::read_to_string(&path).expect("read corrupt backend log"),
        original_log
    );
    let snapshot = store.snapshot();
    assert_eq!(snapshot.len(), 1);
    assert_eq!(snapshot[0].timestamp_unix, 9);
    assert_eq!(snapshot[0].value, "keep");
}

#[test]
fn episodic_cache_rejects_type_invalid_existing_backend() {
    let project = temp_dir("cache-type-invalid");
    let path = episodic_cache_log_path_for(&project).expect("construct backend path");
    std::fs::create_dir_all(path.parent().expect("cache log has parent"))
        .expect("create backend dir");
    let original_log = format!("{}1\t616263\n", cache_header_for_project(&project));
    std::fs::write(&path, &original_log).expect("write typed-invalid backend log");

    let store: EpisodeStore<u64> = EpisodeStore::new();
    store.append_at(9, 7);

    let result = store.append_cache_text(&project, 10, 42);

    assert!(matches!(
        result,
        Err(EpisodePersistenceError::InvalidValue {
            line: 3,
            value,
            ..
        }) if value == "abc"
    ));
    assert_eq!(
        std::fs::read_to_string(&path).expect("read typed-invalid backend log"),
        original_log
    );
    let snapshot = store.snapshot();
    assert_eq!(snapshot.len(), 1);
    assert_eq!(snapshot[0].timestamp_unix, 9);
    assert_eq!(snapshot[0].value, 7);
}

#[test]
fn episodic_cache_concurrent_appends_preserve_all_records() {
    let project = temp_dir("cache-concurrent");
    let writers = 8_u64;

    std::thread::scope(|scope| {
        for idx in 0..writers {
            let project = &project;
            scope.spawn(move || {
                let store: EpisodeStore<String> = EpisodeStore::new();
                store
                    .append_cache_text(project, 100 + idx, format!("writer-{idx}"))
                    .expect("append backend episode from writer");
            });
        }
    });

    let recovered: EpisodeStore<String> = EpisodeStore::new();
    recovered
        .load_cache_text(&project)
        .expect("load backend episodes");
    let mut values: Vec<_> = recovered
        .snapshot()
        .into_iter()
        .map(|episode| episode.value)
        .collect();
    values.sort();

    assert_eq!(values.len(), writers as usize);
    assert_eq!(
        values,
        (0..writers)
            .map(|idx| format!("writer-{idx}"))
            .collect::<Vec<_>>()
    );
}

#[cfg(unix)]
#[test]
fn episodic_cache_backend_uses_private_unix_permissions() {
    use std::os::unix::fs::PermissionsExt;

    let project = temp_dir("cache-permissions");
    let path = episodic_cache_log_path_for(&project).expect("construct backend path");
    let store: EpisodeStore<String> = EpisodeStore::new();

    store
        .append_cache_text(&project, 11, "private".to_string())
        .expect("append backend episode");

    let cache_dir_mode = std::fs::metadata(project.join(EPISODIC_CACHE_DIR))
        .expect("cache dir metadata")
        .permissions()
        .mode()
        & 0o777;
    let episodic_dir_mode = std::fs::metadata(
        project
            .join(EPISODIC_CACHE_DIR)
            .join(EPISODIC_CACHE_EPISODIC_DIR),
    )
    .expect("episodic dir metadata")
    .permissions()
    .mode()
        & 0o777;
    let file_mode = std::fs::metadata(&path)
        .expect("cache log metadata")
        .permissions()
        .mode()
        & 0o777;

    assert_eq!(cache_dir_mode, 0o700);
    assert_eq!(episodic_dir_mode, 0o700);
    assert_eq!(file_mode, 0o600);
}

#[cfg(unix)]
#[test]
fn episodic_cache_rejects_symlinked_cache_directory_without_writing_outside_root() {
    use std::os::unix::fs::symlink;

    let project = temp_dir("cache-symlink-project");
    let outside = temp_dir("cache-symlink-outside");
    symlink(&outside, project.join(EPISODIC_CACHE_DIR)).expect("create cache-dir symlink");

    let store: EpisodeStore<String> = EpisodeStore::new();
    store.append_at(9, "keep".to_string());

    let result = store.append_cache_text(&project, 10, "new".to_string());

    assert!(matches!(
        result,
        Err(EpisodePersistenceError::UnsafePath { .. })
    ));
    assert!(
        !outside.join(EPISODIC_CACHE_EPISODIC_DIR).exists(),
        "backend must not follow .garnet-cache symlink outside project root"
    );
    let snapshot = store.snapshot();
    assert_eq!(snapshot.len(), 1);
    assert_eq!(snapshot[0].timestamp_unix, 9);
    assert_eq!(snapshot[0].value, "keep");
}

#[cfg(unix)]
#[test]
fn episodic_cache_path_helper_canonicalizes_symlinked_project_roots() {
    use std::os::unix::fs::symlink;

    let real_project = temp_dir("cache-real-project");
    let link_parent = temp_dir("cache-project-link-parent");
    let project_link = link_parent.join("linked-project");
    symlink(&real_project, &project_link).expect("create project-root symlink");

    let path = episodic_cache_log_path_for(&project_link).expect("construct backend path");

    assert_eq!(
        path,
        std::fs::canonicalize(&real_project)
            .expect("canonical real project path")
            .join(EPISODIC_CACHE_DIR)
            .join(EPISODIC_CACHE_EPISODIC_DIR)
            .join(EPISODIC_CACHE_LOG_FILE)
    );
}

#[cfg(unix)]
#[test]
fn episodic_cache_append_uses_existing_unlocked_lockfile() {
    let project = temp_dir("cache-stale-lock");
    let path = episodic_cache_log_path_for(&project).expect("construct backend path");
    let lock_path = path.with_file_name("episodes.mnemos.lock");
    std::fs::create_dir_all(lock_path.parent().expect("lock has parent"))
        .expect("create backend dir");
    std::fs::write(&lock_path, "pid=999999\ncreated_unix=1\n").expect("write stale lock");

    let store: EpisodeStore<String> = EpisodeStore::new();
    store
        .append_cache_text(&project, 11, "after-stale-lock".to_string())
        .expect("append through existing unlocked lockfile");

    assert!(lock_path.exists());
    let recovered: EpisodeStore<String> = EpisodeStore::new();
    recovered
        .load_cache_text(&project)
        .expect("load recovered stale-lock backend");
    let snapshot = recovered.snapshot();
    assert_eq!(snapshot.len(), 1);
    assert_eq!(snapshot[0].value, "after-stale-lock");
}
