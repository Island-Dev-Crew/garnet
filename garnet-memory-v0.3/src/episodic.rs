//! Episodic memory: append-only log with timestamp indexing.

use crate::{
    AllocRequest, AllocRootStats, AllocStats, CycleNodeId, HeapKindAllocator, KindAllocator,
    MemoryKind, MemoryPolicy,
};
use std::cell::RefCell;
#[cfg(unix)]
use std::ffi::CString;
use std::fmt;
use std::fs::{self, File, OpenOptions};
#[cfg(any(unix, windows))]
use std::io;
#[cfg(unix)]
use std::io::Read;
use std::io::{ErrorKind, Write};
#[cfg(unix)]
use std::os::fd::{AsRawFd, FromRawFd};
#[cfg(unix)]
use std::os::raw::{c_char, c_int};
#[cfg(unix)]
use std::os::unix::fs::{DirBuilderExt, MetadataExt, OpenOptionsExt};
#[cfg(windows)]
use std::os::windows::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::Arc;
#[cfg(windows)]
use std::thread;
#[cfg(windows)]
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};

pub const EPISODIC_TEXT_LOG_MAX_BYTES: u64 = 8 * 1024 * 1024;
pub const EPISODIC_CACHE_DIR: &str = ".garnet-cache";
pub const EPISODIC_CACHE_EPISODIC_DIR: &str = "episodic";
pub const EPISODIC_CACHE_LOG_FILE: &str = "episodes.mnemos";

const EPISODIC_CACHE_LOCK_FILE: &str = "episodes.mnemos.lock";
#[cfg(windows)]
const EPISODIC_CACHE_LOCK_ATTEMPTS: usize = 1_000;
#[cfg(windows)]
const EPISODIC_CACHE_LOCK_SLEEP: Duration = Duration::from_millis(5);
#[cfg(unix)]
const LOCK_EX: i32 = 2;
#[cfg(unix)]
const LOCK_UN: i32 = 8;

#[cfg(any(target_os = "linux", target_os = "android"))]
const EPISODIC_CACHE_OPENAT_SUPPORTED: bool = true;
#[cfg(target_vendor = "apple")]
const EPISODIC_CACHE_OPENAT_SUPPORTED: bool = true;
#[cfg(all(
    unix,
    not(any(target_os = "linux", target_os = "android", target_vendor = "apple"))
))]
const EPISODIC_CACHE_OPENAT_SUPPORTED: bool = false;

#[cfg(any(target_os = "linux", target_os = "android"))]
const O_RDONLY: c_int = 0;
#[cfg(any(target_os = "linux", target_os = "android"))]
const O_WRONLY: c_int = 1;
#[cfg(any(target_os = "linux", target_os = "android"))]
const O_CREAT: c_int = 0o100;
#[cfg(any(target_os = "linux", target_os = "android"))]
const O_EXCL: c_int = 0o200;
#[cfg(any(target_os = "linux", target_os = "android"))]
const O_TRUNC: c_int = 0o1000;
#[cfg(any(target_os = "linux", target_os = "android"))]
const O_NOFOLLOW: c_int = 0o400000;
#[cfg(any(target_os = "linux", target_os = "android"))]
const O_DIRECTORY: c_int = 0o200000;

#[cfg(target_vendor = "apple")]
const O_RDONLY: c_int = 0;
#[cfg(target_vendor = "apple")]
const O_WRONLY: c_int = 1;
#[cfg(target_vendor = "apple")]
const O_CREAT: c_int = 0x0000_0200;
#[cfg(target_vendor = "apple")]
const O_EXCL: c_int = 0x0000_0800;
#[cfg(target_vendor = "apple")]
const O_TRUNC: c_int = 0x0000_0400;
#[cfg(target_vendor = "apple")]
const O_NOFOLLOW: c_int = 0x0000_0100;
#[cfg(target_vendor = "apple")]
const O_DIRECTORY: c_int = 0x0010_0000;

#[cfg(all(
    unix,
    not(any(target_os = "linux", target_os = "android", target_vendor = "apple"))
))]
const O_RDONLY: c_int = 0;
#[cfg(all(
    unix,
    not(any(target_os = "linux", target_os = "android", target_vendor = "apple"))
))]
const O_WRONLY: c_int = 0;
#[cfg(all(
    unix,
    not(any(target_os = "linux", target_os = "android", target_vendor = "apple"))
))]
const O_CREAT: c_int = 0;
#[cfg(all(
    unix,
    not(any(target_os = "linux", target_os = "android", target_vendor = "apple"))
))]
const O_EXCL: c_int = 0;
#[cfg(all(
    unix,
    not(any(target_os = "linux", target_os = "android", target_vendor = "apple"))
))]
const O_TRUNC: c_int = 0;
#[cfg(all(
    unix,
    not(any(target_os = "linux", target_os = "android", target_vendor = "apple"))
))]
const O_NOFOLLOW: c_int = 0;
#[cfg(all(
    unix,
    not(any(target_os = "linux", target_os = "android", target_vendor = "apple"))
))]
const O_DIRECTORY: c_int = 0;

#[cfg(unix)]
unsafe extern "C" {
    fn flock(fd: i32, operation: i32) -> i32;
    fn openat(dirfd: c_int, pathname: *const c_char, flags: c_int, ...) -> c_int;
    fn renameat(
        olddirfd: c_int,
        oldpath: *const c_char,
        newdirfd: c_int,
        newpath: *const c_char,
    ) -> c_int;
    fn unlinkat(dirfd: c_int, pathname: *const c_char, flags: c_int) -> c_int;
}

#[derive(Debug, Clone)]
pub struct Episode<T> {
    pub timestamp_unix: u64,
    pub value: T,
}

struct StoredEpisode<T> {
    event: Episode<T>,
    root: Option<CycleNodeId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EpisodePersistenceError {
    Io {
        action: &'static str,
        path: String,
        error: String,
    },
    MissingHeader,
    UnsupportedHeader(String),
    MalformedLine {
        line: usize,
        reason: String,
    },
    InvalidTimestamp {
        line: usize,
        value: String,
    },
    InvalidHex {
        line: usize,
        value: String,
    },
    InvalidUtf8 {
        line: usize,
        error: String,
    },
    InvalidValue {
        line: usize,
        value: String,
        error: String,
    },
    LogTooLarge {
        path: String,
        bytes: u64,
        limit: u64,
    },
    UnsafePath {
        path: String,
        reason: String,
    },
}

impl fmt::Display for EpisodePersistenceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io {
                action,
                path,
                error,
            } => write!(
                f,
                "failed to {action} episodic persistence file {path}: {error}"
            ),
            Self::MissingHeader => write!(f, "missing episodic persistence header"),
            Self::UnsupportedHeader(header) => {
                write!(f, "unsupported episodic persistence header {header:?}")
            }
            Self::MalformedLine { line, reason } => {
                write!(f, "malformed episodic persistence line {line}: {reason}")
            }
            Self::InvalidTimestamp { line, value } => {
                write!(
                    f,
                    "invalid episodic persistence timestamp on line {line}: {value:?}"
                )
            }
            Self::InvalidHex { line, value } => {
                write!(
                    f,
                    "invalid episodic persistence payload hex on line {line}: {value:?}"
                )
            }
            Self::InvalidUtf8 { line, error } => {
                write!(
                    f,
                    "invalid episodic persistence UTF-8 payload on line {line}: {error}"
                )
            }
            Self::InvalidValue { line, value, error } => {
                write!(
                    f,
                    "invalid episodic persistence value on line {line}: {value:?}: {error}"
                )
            }
            Self::LogTooLarge { path, bytes, limit } => {
                write!(
                    f,
                    "episodic persistence file {path} is too large: {bytes} bytes exceeds {limit} byte limit"
                )
            }
            Self::UnsafePath { path, reason } => {
                write!(f, "unsafe episodic persistence path {path}: {reason}")
            }
        }
    }
}

impl std::error::Error for EpisodePersistenceError {}

pub struct EpisodeStore<T> {
    events: RefCell<Vec<StoredEpisode<T>>>,
    alloc: Arc<dyn KindAllocator>,
    policy: MemoryPolicy,
    eviction_enabled: bool,
}

impl<T> Default for EpisodeStore<T> {
    fn default() -> Self {
        Self::with_policy_state(MemoryPolicy::default_for(MemoryKind::Episodic), false)
    }
}

impl<T> EpisodeStore<T> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_policy(policy: MemoryPolicy) -> Self {
        Self::with_policy_state(policy, true)
    }

    pub fn with_allocator(alloc: Arc<dyn KindAllocator>) -> Self {
        Self::with_policy_allocator_state(
            MemoryPolicy::default_for(MemoryKind::Episodic),
            alloc,
            false,
        )
    }

    pub fn with_policy_and_allocator(policy: MemoryPolicy, alloc: Arc<dyn KindAllocator>) -> Self {
        Self::with_policy_allocator_state(policy, alloc, true)
    }

    fn with_policy_state(policy: MemoryPolicy, eviction_enabled: bool) -> Self {
        Self::with_policy_allocator_state(
            policy,
            HeapKindAllocator::shared(MemoryKind::Episodic),
            eviction_enabled,
        )
    }

    fn with_policy_allocator_state(
        policy: MemoryPolicy,
        alloc: Arc<dyn KindAllocator>,
        eviction_enabled: bool,
    ) -> Self {
        assert_eq!(alloc.kind(), MemoryKind::Episodic);
        Self {
            events: RefCell::new(Vec::new()),
            alloc,
            policy,
            eviction_enabled,
        }
    }

    /// Append an event tagged with the current system time.
    pub fn append(&self, value: T) {
        self.append_at(unix_now(), value);
    }

    /// Append with an explicit timestamp (useful for replay and testing).
    pub fn append_at(&self, timestamp: u64, value: T) {
        self.alloc.reserve(AllocRequest::for_items::<Episode<T>>(1));
        let mut events = self.events.borrow_mut();
        events.reserve(1);
        let event = Episode {
            timestamp_unix: timestamp,
            value,
        };
        events.push(StoredEpisode {
            event,
            root: self.alloc.retain_root("episodic:event"),
        });
    }

    pub fn len(&self) -> usize {
        self.events.borrow().len()
    }

    pub fn is_empty(&self) -> bool {
        self.events.borrow().is_empty()
    }

    pub fn allocator_stats(&self) -> AllocStats {
        self.alloc.stats()
    }

    pub fn allocator_root_stats(&self) -> AllocRootStats {
        self.alloc.root_stats()
    }

    fn evict_at(&self, now: u64) {
        if !self.eviction_enabled {
            return;
        }
        let mut events = self.events.borrow_mut();
        let mut retained = Vec::with_capacity(events.len());
        for event in events.drain(..) {
            let age = now.saturating_sub(event.event.timestamp_unix) as f64;
            if self.policy.should_retain(self.policy.score(1.0, age, 1.0)) {
                retained.push(event);
            } else {
                self.release_event_root(event);
            }
        }
        *events = retained;
        let high_water = self.policy.compaction_high_water;
        if high_water > 0 && events.len() > high_water {
            let drop_count = events.len() - high_water;
            for event in events.drain(0..drop_count) {
                self.release_event_root(event);
            }
        }
    }

    fn release_event_root(&self, event: StoredEpisode<T>) {
        if let Some(root) = event.root {
            self.alloc.release_root(root);
        }
    }

    fn replace_events(&self, episodes: Vec<Episode<T>>) {
        let mut events = self.events.borrow_mut();
        for event in events.drain(..) {
            self.release_event_root(event);
        }
        events.reserve(episodes.len());
        for event in episodes {
            self.alloc.reserve(AllocRequest::for_items::<Episode<T>>(1));
            events.push(StoredEpisode {
                event,
                root: self.alloc.retain_root("episodic:event"),
            });
        }
    }
}

impl<T: Clone> EpisodeStore<T> {
    /// Return the N most recent events (or all if N > len).
    pub fn recent(&self, n: usize) -> Vec<Episode<T>> {
        self.evict_at(unix_now());
        let events = self.events.borrow();
        let start = events.len().saturating_sub(n);
        events[start..]
            .iter()
            .map(|stored| stored.event.clone())
            .collect()
    }

    /// Return events whose timestamp ≥ since.
    pub fn since(&self, since: u64) -> Vec<Episode<T>> {
        self.evict_at(unix_now());
        self.events
            .borrow()
            .iter()
            .filter(|stored| stored.event.timestamp_unix >= since)
            .map(|stored| stored.event.clone())
            .collect()
    }

    pub fn snapshot(&self) -> Vec<Episode<T>> {
        self.evict_at(unix_now());
        self.events
            .borrow()
            .iter()
            .map(|stored| stored.event.clone())
            .collect()
    }
}

impl<T: ToString> EpisodeStore<T> {
    /// Persist a versioned text snapshot of the episodic log.
    ///
    /// The payload is intentionally dependency-free: a fixed header followed by
    /// `timestamp<TAB>hex(value.to_string())` records. The write goes through a
    /// sibling temp file and rename so callers do not observe a partial target
    /// file on ordinary local filesystems.
    pub fn save_text<P: AsRef<Path>>(&self, path: P) -> Result<(), EpisodePersistenceError> {
        self.evict_at(unix_now());
        let path = path.as_ref();
        validate_regular_target_if_exists(path)?;
        let mut out = String::from("garnet-episodic-v1\n");
        for stored in self.events.borrow().iter() {
            out.push_str(&stored.event.timestamp_unix.to_string());
            out.push('\t');
            out.push_str(&hex_encode(stored.event.value.to_string().as_bytes()));
            out.push('\n');
        }

        ensure_parent_dir(path)?;

        let tmp = temp_path_for(path);
        let mut file = create_private_new_file(&tmp)?;
        file.write_all(out.as_bytes())
            .map_err(|error| persistence_io("write", &tmp, error))?;
        file.sync_data()
            .map_err(|error| persistence_io("sync", &tmp, error))?;
        drop(file);
        fs::rename(&tmp, path).map_err(|error| {
            let _ = fs::remove_file(&tmp);
            persistence_io("rename", path, error)
        })?;
        set_path_private_file(path)?;
        Ok(())
    }
}

impl<T> EpisodeStore<T>
where
    T: ToString + FromStr,
    T::Err: fmt::Display,
{
    /// Commit one additional episode to a versioned text log, then add it to this store.
    ///
    /// Existing logs and the projected post-commit log are size-bounded before
    /// extension so malformed or oversized files cannot be silently carried
    /// into live memory. Existing records must parse as this store's `T`. The
    /// canonical file is updated through a temp-file rewrite and rename rather
    /// than an in-place append.
    pub fn append_text<P: AsRef<Path>>(
        &self,
        path: P,
        timestamp: u64,
        value: T,
    ) -> Result<(), EpisodePersistenceError> {
        let path = path.as_ref();
        ensure_parent_dir(path)?;
        let existing = prepare_append_log::<T>(path)?;
        let mut record = String::new();
        if existing.is_empty() {
            record.push_str("garnet-episodic-v1\n");
        }
        record.push_str(&timestamp.to_string());
        record.push('\t');
        record.push_str(&hex_encode(value.to_string().as_bytes()));
        record.push('\n');

        let projected_bytes = existing.len().saturating_add(record.len()) as u64;
        if projected_bytes > EPISODIC_TEXT_LOG_MAX_BYTES {
            return Err(EpisodePersistenceError::LogTooLarge {
                path: path.display().to_string(),
                bytes: projected_bytes,
                limit: EPISODIC_TEXT_LOG_MAX_BYTES,
            });
        }

        let tmp = temp_path_for(path);
        let mut file = create_private_new_file(&tmp)?;
        file.write_all(existing.as_bytes())
            .map_err(|error| persistence_io("write", &tmp, error))?;
        file.write_all(record.as_bytes())
            .map_err(|error| persistence_io("write", &tmp, error))?;
        file.sync_data()
            .map_err(|error| persistence_io("sync", &tmp, error))?;
        drop(file);
        fs::rename(&tmp, path).map_err(|error| {
            let _ = fs::remove_file(&tmp);
            persistence_io("rename", path, error)
        })?;
        set_path_private_file(path)?;
        self.append_at(timestamp, value);
        Ok(())
    }

    /// Commit one episode into the default per-project episodic cache backend.
    ///
    /// This is the narrow Phase 6N backend boundary for Mnemos' text format:
    /// `<project>/.garnet-cache/episodic/episodes.mnemos`. The path components
    /// are fixed, directories are private on Unix, symlink/non-regular targets
    /// are rejected, and a sibling lockfile serializes rewrite-based commits.
    /// It is distinct from the CLI's signed NDJSON `.garnet-cache/episodes.log`
    /// trust model and does not make these typed records trusted compiler input.
    pub fn append_cache_text<P: AsRef<Path>>(
        &self,
        project_root: P,
        timestamp: u64,
        value: T,
    ) -> Result<(), EpisodePersistenceError> {
        let backend = EpisodicCacheBackend::prepare(project_root.as_ref())?;
        self.append_prepared_cache_text(&backend, timestamp, value)
    }

    fn append_prepared_cache_text(
        &self,
        backend: &EpisodicCacheBackend,
        timestamp: u64,
        value: T,
    ) -> Result<(), EpisodePersistenceError> {
        backend.validate_dir_identity()?;
        let _lock = backend.acquire_lock()?;
        backend.validate_dir_identity()?;
        #[cfg(unix)]
        {
            self.append_text_in_prepared_cache(backend, timestamp, value)
        }
        #[cfg(not(unix))]
        {
            validate_regular_target_if_exists(&backend.log_path)?;
            self.append_text(&backend.log_path, timestamp, value)?;
            set_path_private_file(&backend.log_path)?;
            Ok(())
        }
    }
}

impl<T> EpisodeStore<T>
where
    T: FromStr,
    T::Err: fmt::Display,
{
    /// Replace this store with the contents of a versioned text snapshot.
    ///
    /// Parsing is all-or-nothing: malformed or unsupported files return an
    /// error before existing in-memory episodes or roots are touched.
    pub fn load_text<P: AsRef<Path>>(&self, path: P) -> Result<(), EpisodePersistenceError> {
        let path = path.as_ref();
        let raw = read_persistence_text(path)?;
        let episodes = parse_persisted_episodes(&raw)?;
        self.replace_events(episodes);
        Ok(())
    }

    /// Replace this store with the default per-project episodic cache backend.
    ///
    /// A missing backend log is treated as an empty backend. Existing corrupt,
    /// oversized, symlinked, or wrong-type logs fail before the live store is
    /// mutated.
    pub fn load_cache_text<P: AsRef<Path>>(
        &self,
        project_root: P,
    ) -> Result<(), EpisodePersistenceError> {
        let backend = EpisodicCacheBackend::prepare(project_root.as_ref())?;
        self.load_prepared_cache_text(&backend)
    }

    fn load_prepared_cache_text(
        &self,
        backend: &EpisodicCacheBackend,
    ) -> Result<(), EpisodePersistenceError> {
        backend.validate_dir_identity()?;
        let _lock = backend.acquire_lock()?;
        backend.validate_dir_identity()?;
        #[cfg(unix)]
        {
            let Some(raw) = read_persistence_text_in_dir_optional(
                &backend.episodic_dir,
                EPISODIC_CACHE_LOG_FILE,
                &backend.log_path,
            )?
            else {
                self.replace_events(Vec::new());
                return Ok(());
            };
            let episodes = parse_persisted_episodes(&raw)?;
            self.replace_events(episodes);
            Ok(())
        }
        #[cfg(not(unix))]
        {
            validate_regular_target_if_exists(&backend.log_path)?;
            if !backend.log_path.exists() {
                self.replace_events(Vec::new());
                return Ok(());
            }
            self.load_text(&backend.log_path)
        }
    }
}

#[cfg(unix)]
impl<T> EpisodeStore<T>
where
    T: ToString + FromStr,
    T::Err: fmt::Display,
{
    fn append_text_in_prepared_cache(
        &self,
        backend: &EpisodicCacheBackend,
        timestamp: u64,
        value: T,
    ) -> Result<(), EpisodePersistenceError> {
        let existing = prepare_append_log_in_dir::<T>(
            &backend.episodic_dir,
            EPISODIC_CACHE_LOG_FILE,
            &backend.log_path,
        )?;
        let mut record = String::new();
        if existing.is_empty() {
            record.push_str("garnet-episodic-v1\n");
        }
        record.push_str(&timestamp.to_string());
        record.push('\t');
        record.push_str(&hex_encode(value.to_string().as_bytes()));
        record.push('\n');

        let projected_bytes = existing.len().saturating_add(record.len()) as u64;
        if projected_bytes > EPISODIC_TEXT_LOG_MAX_BYTES {
            return Err(EpisodePersistenceError::LogTooLarge {
                path: backend.log_path.display().to_string(),
                bytes: projected_bytes,
                limit: EPISODIC_TEXT_LOG_MAX_BYTES,
            });
        }

        write_persistence_text_in_dir(
            &backend.episodic_dir,
            EPISODIC_CACHE_LOG_FILE,
            &backend.log_path,
            &[existing.as_bytes(), record.as_bytes()],
        )?;
        backend.validate_dir_identity()?;
        self.append_at(timestamp, value);
        Ok(())
    }
}

/// Return the canonical fixed per-project episodic text backend path.
///
/// This helper resolves the project root before appending the fixed backend
/// components. Use `append_cache_text` and `load_cache_text` for the guarded
/// backend operations.
pub fn episodic_cache_log_path_for<P: AsRef<Path>>(
    project_root: P,
) -> Result<PathBuf, EpisodePersistenceError> {
    Ok(fs::canonicalize(project_root.as_ref())
        .map_err(|error| persistence_io("canonicalize", project_root.as_ref(), error))?
        .join(EPISODIC_CACHE_DIR)
        .join(EPISODIC_CACHE_EPISODIC_DIR)
        .join(EPISODIC_CACHE_LOG_FILE))
}

struct EpisodicCacheBackend {
    #[cfg(unix)]
    episodic_dir_path: PathBuf,
    log_path: PathBuf,
    lock_path: PathBuf,
    #[cfg(unix)]
    episodic_dir: File,
}

impl EpisodicCacheBackend {
    fn prepare(project_root: &Path) -> Result<Self, EpisodePersistenceError> {
        let root = fs::canonicalize(project_root)
            .map_err(|error| persistence_io("canonicalize", project_root, error))?;
        let metadata =
            fs::metadata(&root).map_err(|error| persistence_io("metadata", &root, error))?;
        if !metadata.is_dir() {
            return Err(unsafe_path(&root, "project root must be a directory"));
        }

        let cache_dir = root.join(EPISODIC_CACHE_DIR);
        ensure_private_dir(&cache_dir)?;
        let episodic_dir = cache_dir.join(EPISODIC_CACHE_EPISODIC_DIR);
        ensure_private_dir(&episodic_dir)?;

        let log_path = episodic_cache_log_path_for(&root)?;
        let lock_path = episodic_dir.join(EPISODIC_CACHE_LOCK_FILE);
        #[cfg(unix)]
        let episodic_dir_file = open_validated_private_dir(&episodic_dir)?;
        #[cfg(unix)]
        {
            validate_regular_target_in_dir_if_exists(
                &episodic_dir_file,
                EPISODIC_CACHE_LOG_FILE,
                &log_path,
            )?;
            validate_regular_target_in_dir_if_exists(
                &episodic_dir_file,
                EPISODIC_CACHE_LOCK_FILE,
                &lock_path,
            )?;
        }
        #[cfg(not(unix))]
        {
            validate_regular_target_if_exists(&log_path)?;
            validate_regular_target_if_exists(&lock_path)?;
        }

        Ok(Self {
            #[cfg(unix)]
            episodic_dir_path: episodic_dir,
            log_path,
            lock_path,
            #[cfg(unix)]
            episodic_dir: episodic_dir_file,
        })
    }

    #[cfg(unix)]
    fn validate_dir_identity(&self) -> Result<(), EpisodePersistenceError> {
        let path_metadata = fs::symlink_metadata(&self.episodic_dir_path)
            .map_err(|error| persistence_io("metadata", &self.episodic_dir_path, error))?;
        if path_metadata.file_type().is_symlink() {
            return Err(unsafe_path(
                &self.episodic_dir_path,
                "directory must not be a symlink",
            ));
        }
        if !path_metadata.is_dir() {
            return Err(unsafe_path(
                &self.episodic_dir_path,
                "path must be a directory",
            ));
        }
        let fd_metadata = self
            .episodic_dir
            .metadata()
            .map_err(|error| persistence_io("metadata", &self.episodic_dir_path, error))?;
        if fd_metadata.dev() != path_metadata.dev() || fd_metadata.ino() != path_metadata.ino() {
            return Err(unsafe_path(
                &self.episodic_dir_path,
                "directory changed after episodic cache backend preparation",
            ));
        }
        Ok(())
    }

    #[cfg(not(unix))]
    fn validate_dir_identity(&self) -> Result<(), EpisodePersistenceError> {
        Ok(())
    }

    fn acquire_lock(&self) -> Result<CacheLock, EpisodePersistenceError> {
        #[cfg(unix)]
        {
            acquire_cache_lock_at(
                &self.episodic_dir,
                EPISODIC_CACHE_LOCK_FILE,
                &self.lock_path,
            )
        }
        #[cfg(windows)]
        {
            acquire_cache_lock(&self.lock_path)
        }
        #[cfg(not(any(unix, windows)))]
        {
            acquire_cache_lock(&self.lock_path)
        }
    }
}

#[cfg(unix)]
struct CacheLock {
    path: PathBuf,
    file: File,
}

#[cfg(not(unix))]
struct CacheLock {
    #[cfg(windows)]
    _path: PathBuf,
    #[cfg(windows)]
    _file: File,
}

#[cfg(unix)]
impl Drop for CacheLock {
    fn drop(&mut self) {
        let _ = unlock_cache_file(&self.file, &self.path);
    }
}

#[cfg(unix)]
fn acquire_cache_lock_at(
    dir: &File,
    name: &str,
    path: &Path,
) -> Result<CacheLock, EpisodePersistenceError> {
    let mut file = open_dir_entry(
        dir,
        name,
        O_WRONLY | O_CREAT | O_NOFOLLOW,
        0o600,
        path,
        "open",
    )?;
    lock_cache_file(&file, path)?;
    write_cache_lock_header(&mut file, path, &new_lock_marker())?;
    Ok(CacheLock {
        path: path.to_path_buf(),
        file,
    })
}

#[cfg(windows)]
fn acquire_cache_lock(path: &Path) -> Result<CacheLock, EpisodePersistenceError> {
    validate_regular_target_if_exists(path)?;
    for _ in 0..EPISODIC_CACHE_LOCK_ATTEMPTS {
        match OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .share_mode(0)
            .open(path)
        {
            Ok(mut file) => {
                write_cache_lock_header(&mut file, path, &new_lock_marker())?;
                return Ok(CacheLock {
                    _path: path.to_path_buf(),
                    _file: file,
                });
            }
            Err(error) if is_windows_lock_contention(&error) => {
                thread::sleep(EPISODIC_CACHE_LOCK_SLEEP);
            }
            Err(error) => return Err(persistence_io("lock", path, error)),
        }
    }

    Err(EpisodePersistenceError::Io {
        action: "lock",
        path: path.display().to_string(),
        error: "timed out waiting for episodic cache lock".to_string(),
    })
}

#[cfg(not(any(unix, windows)))]
fn acquire_cache_lock(path: &Path) -> Result<CacheLock, EpisodePersistenceError> {
    Err(EpisodePersistenceError::Io {
        action: "lock",
        path: path.display().to_string(),
        error: "episodic cache backend requires an OS-backed file lock on this platform"
            .to_string(),
    })
}

fn write_cache_lock_header(
    file: &mut File,
    path: &Path,
    marker: &str,
) -> Result<(), EpisodePersistenceError> {
    set_file_private(file, path)?;
    file.set_len(0)
        .map_err(|error| persistence_io("truncate", path, error))?;
    writeln!(file, "pid={}", std::process::id())
        .map_err(|error| persistence_io("write", path, error))?;
    writeln!(file, "marker={marker}").map_err(|error| persistence_io("write", path, error))?;
    writeln!(file, "created_unix={}", unix_now())
        .map_err(|error| persistence_io("write", path, error))?;
    file.sync_data()
        .map_err(|error| persistence_io("sync", path, error))
}

fn new_lock_marker() -> String {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos().to_string())
        .unwrap_or_else(|error| format!("clock-error-{error:?}"));
    format!("{}-{stamp}", std::process::id())
}

#[cfg(windows)]
fn is_windows_lock_contention(error: &io::Error) -> bool {
    matches!(error.raw_os_error(), Some(32) | Some(33))
        || error.kind() == ErrorKind::PermissionDenied
}

#[cfg(unix)]
fn lock_cache_file(file: &File, path: &Path) -> Result<(), EpisodePersistenceError> {
    flock_cache_file(file, path, LOCK_EX, "lock")
}

#[cfg(unix)]
fn unlock_cache_file(file: &File, path: &Path) -> Result<(), EpisodePersistenceError> {
    flock_cache_file(file, path, LOCK_UN, "unlock")
}

#[cfg(unix)]
fn flock_cache_file(
    file: &File,
    path: &Path,
    operation: i32,
    action: &'static str,
) -> Result<(), EpisodePersistenceError> {
    loop {
        let rc = unsafe { flock(file.as_raw_fd(), operation) };
        if rc == 0 {
            return Ok(());
        }
        let error = io::Error::last_os_error();
        if error.kind() == ErrorKind::Interrupted {
            continue;
        }
        return Err(persistence_io(action, path, error));
    }
}

#[cfg(unix)]
fn open_validated_private_dir(path: &Path) -> Result<File, EpisodePersistenceError> {
    if !EPISODIC_CACHE_OPENAT_SUPPORTED {
        return Err(EpisodePersistenceError::Io {
            action: "open",
            path: path.display().to_string(),
            error: "episodic cache backend requires an fd-anchored openat implementation on this Unix target"
                .to_string(),
        });
    }

    let before =
        fs::symlink_metadata(path).map_err(|error| persistence_io("metadata", path, error))?;
    if before.file_type().is_symlink() {
        return Err(unsafe_path(path, "directory must not be a symlink"));
    }
    if !before.is_dir() {
        return Err(unsafe_path(path, "path must be a directory"));
    }

    let file = match OpenOptions::new()
        .read(true)
        .custom_flags(O_DIRECTORY | O_NOFOLLOW)
        .open(path)
    {
        Ok(file) => file,
        Err(error) if is_symlink_loop_error(&error) => {
            return Err(unsafe_path(path, "directory must not be a symlink"));
        }
        Err(error) => return Err(persistence_io("open", path, error)),
    };

    let after =
        fs::symlink_metadata(path).map_err(|error| persistence_io("metadata", path, error))?;
    if after.file_type().is_symlink() {
        return Err(unsafe_path(path, "directory must not be a symlink"));
    }
    let fd_metadata = file
        .metadata()
        .map_err(|error| persistence_io("metadata", path, error))?;
    if !fd_metadata.is_dir() {
        return Err(unsafe_path(path, "path must be a directory"));
    }
    if fd_metadata.dev() != before.dev()
        || fd_metadata.ino() != before.ino()
        || fd_metadata.dev() != after.dev()
        || fd_metadata.ino() != after.ino()
    {
        return Err(unsafe_path(
            path,
            "directory changed while opening episodic cache backend",
        ));
    }

    Ok(file)
}

#[cfg(unix)]
fn validate_regular_target_in_dir_if_exists(
    dir: &File,
    name: &str,
    path: &Path,
) -> Result<(), EpisodePersistenceError> {
    let Some(file) = open_dir_entry_if_exists(dir, name, O_RDONLY | O_NOFOLLOW, 0, path, "open")?
    else {
        return Ok(());
    };
    validate_open_regular_file(&file, path).map(|_| ())
}

#[cfg(unix)]
fn prepare_append_log_in_dir<T>(
    dir: &File,
    name: &str,
    path: &Path,
) -> Result<String, EpisodePersistenceError>
where
    T: FromStr,
    T::Err: fmt::Display,
{
    let Some(raw) = read_persistence_text_in_dir_optional(dir, name, path)? else {
        return Ok(String::new());
    };
    parse_persisted_episodes::<T>(&raw)?;
    if !raw.ends_with('\n') {
        return Err(EpisodePersistenceError::MalformedLine {
            line: raw.lines().count(),
            reason: "append log must end at a complete record boundary".to_string(),
        });
    }
    Ok(raw)
}

#[cfg(unix)]
fn read_persistence_text_in_dir_optional(
    dir: &File,
    name: &str,
    path: &Path,
) -> Result<Option<String>, EpisodePersistenceError> {
    let Some(mut file) =
        open_dir_entry_if_exists(dir, name, O_RDONLY | O_NOFOLLOW, 0, path, "open")?
    else {
        return Ok(None);
    };
    validate_open_regular_file(&file, path)?;
    let mut raw = String::new();
    file.read_to_string(&mut raw)
        .map_err(|error| persistence_io("read", path, error))?;
    Ok(Some(raw))
}

#[cfg(unix)]
fn validate_open_regular_file(file: &File, path: &Path) -> Result<u64, EpisodePersistenceError> {
    let metadata = file
        .metadata()
        .map_err(|error| persistence_io("metadata", path, error))?;
    if !metadata.is_file() {
        return Err(unsafe_path(path, "target must be a regular file"));
    }
    if metadata.len() > EPISODIC_TEXT_LOG_MAX_BYTES {
        return Err(EpisodePersistenceError::LogTooLarge {
            path: path.display().to_string(),
            bytes: metadata.len(),
            limit: EPISODIC_TEXT_LOG_MAX_BYTES,
        });
    }
    Ok(metadata.len())
}

#[cfg(unix)]
fn write_persistence_text_in_dir(
    dir: &File,
    name: &str,
    path: &Path,
    chunks: &[&[u8]],
) -> Result<(), EpisodePersistenceError> {
    let temp_name = temp_entry_name_for(name);
    let temp_path = path.with_file_name(&temp_name);
    let mut file = open_dir_entry(
        dir,
        &temp_name,
        O_WRONLY | O_CREAT | O_EXCL | O_TRUNC | O_NOFOLLOW,
        0o600,
        &temp_path,
        "create",
    )?;

    let write_result = (|| {
        for chunk in chunks {
            file.write_all(chunk)
                .map_err(|error| persistence_io("write", &temp_path, error))?;
        }
        file.sync_data()
            .map_err(|error| persistence_io("sync", &temp_path, error))?;
        Ok(())
    })();

    if let Err(error) = write_result {
        let _ = unlink_dir_entry(dir, &temp_name);
        return Err(error);
    }
    drop(file);

    if let Err(error) = rename_dir_entry(dir, &temp_name, name, path) {
        let _ = unlink_dir_entry(dir, &temp_name);
        return Err(error);
    }
    Ok(())
}

#[cfg(unix)]
fn open_dir_entry_if_exists(
    dir: &File,
    name: &str,
    flags: c_int,
    mode: c_int,
    path: &Path,
    action: &'static str,
) -> Result<Option<File>, EpisodePersistenceError> {
    let c_name = cstring_entry_name(name, path)?;
    loop {
        let fd = unsafe { openat(dir.as_raw_fd(), c_name.as_ptr(), flags, mode) };
        if fd >= 0 {
            return Ok(Some(unsafe { File::from_raw_fd(fd) }));
        }
        let error = io::Error::last_os_error();
        if error.kind() == ErrorKind::Interrupted {
            continue;
        }
        if error.kind() == ErrorKind::NotFound {
            return Ok(None);
        }
        if is_symlink_loop_error(&error) {
            return Err(unsafe_path(path, "target must not be a symlink"));
        }
        return Err(persistence_io(action, path, error));
    }
}

#[cfg(unix)]
fn open_dir_entry(
    dir: &File,
    name: &str,
    flags: c_int,
    mode: c_int,
    path: &Path,
    action: &'static str,
) -> Result<File, EpisodePersistenceError> {
    let c_name = cstring_entry_name(name, path)?;
    let mut not_found_retries = 0;
    loop {
        let fd = unsafe { openat(dir.as_raw_fd(), c_name.as_ptr(), flags, mode) };
        if fd >= 0 {
            return Ok(unsafe { File::from_raw_fd(fd) });
        }
        let error = io::Error::last_os_error();
        if error.kind() == ErrorKind::Interrupted {
            continue;
        }
        if error.kind() == ErrorKind::NotFound && flags & O_CREAT != 0 && not_found_retries < 32 {
            not_found_retries += 1;
            std::thread::yield_now();
            continue;
        }
        if error.kind() == ErrorKind::NotFound {
            return Err(persistence_io(action, path, error));
        }
        if is_symlink_loop_error(&error) {
            return Err(unsafe_path(path, "target must not be a symlink"));
        }
        return Err(persistence_io(action, path, error));
    }
}

#[cfg(unix)]
fn rename_dir_entry(
    dir: &File,
    old_name: &str,
    new_name: &str,
    path: &Path,
) -> Result<(), EpisodePersistenceError> {
    let old_c_name = cstring_entry_name(old_name, path)?;
    let new_c_name = cstring_entry_name(new_name, path)?;
    loop {
        let rc = unsafe {
            renameat(
                dir.as_raw_fd(),
                old_c_name.as_ptr(),
                dir.as_raw_fd(),
                new_c_name.as_ptr(),
            )
        };
        if rc == 0 {
            return Ok(());
        }
        let error = io::Error::last_os_error();
        if error.kind() == ErrorKind::Interrupted {
            continue;
        }
        return Err(persistence_io("rename", path, error));
    }
}

#[cfg(unix)]
fn unlink_dir_entry(dir: &File, name: &str) -> Result<(), EpisodePersistenceError> {
    let c_name = cstring_entry_name(name, Path::new(name))?;
    loop {
        let rc = unsafe { unlinkat(dir.as_raw_fd(), c_name.as_ptr(), 0) };
        if rc == 0 {
            return Ok(());
        }
        let error = io::Error::last_os_error();
        if error.kind() == ErrorKind::Interrupted {
            continue;
        }
        return Err(persistence_io("remove", Path::new(name), error));
    }
}

#[cfg(unix)]
fn cstring_entry_name(name: &str, path: &Path) -> Result<CString, EpisodePersistenceError> {
    CString::new(name).map_err(|_| unsafe_path(path, "path component must not contain NUL"))
}

#[cfg(unix)]
fn temp_entry_name_for(name: &str) -> String {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    format!(".{name}.tmp-{}-{nonce}", std::process::id())
}

#[cfg(unix)]
fn is_symlink_loop_error(error: &io::Error) -> bool {
    matches!(error.raw_os_error(), Some(40) | Some(62))
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn temp_path_for(path: &Path) -> PathBuf {
    let file_name = path
        .file_name()
        .map(|name| name.to_string_lossy())
        .unwrap_or_else(|| "episodes.mnemos".into());
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    path.with_file_name(format!(".{file_name}.tmp-{}-{nonce}", std::process::id()))
}

fn ensure_parent_dir(path: &Path) -> Result<(), EpisodePersistenceError> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).map_err(|error| persistence_io("create", parent, error))?;
        }
    }
    Ok(())
}

fn prepare_append_log<T>(path: &Path) -> Result<String, EpisodePersistenceError>
where
    T: FromStr,
    T::Err: fmt::Display,
{
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() => {
            return Err(unsafe_path(path, "target must not be a symlink"));
        }
        Ok(metadata) if !metadata.is_file() => {
            return Err(unsafe_path(path, "target must be a regular file"));
        }
        Ok(metadata) if metadata.len() > EPISODIC_TEXT_LOG_MAX_BYTES => {
            return Err(EpisodePersistenceError::LogTooLarge {
                path: path.display().to_string(),
                bytes: metadata.len(),
                limit: EPISODIC_TEXT_LOG_MAX_BYTES,
            });
        }
        Ok(_) => {}
        Err(error) if error.kind() == ErrorKind::NotFound => {
            return Ok(String::new());
        }
        Err(error) => return Err(persistence_io("metadata", path, error)),
    }

    match read_persistence_text(path) {
        Ok(raw) => {
            parse_persisted_episodes::<T>(&raw)?;
            if !raw.ends_with('\n') {
                return Err(EpisodePersistenceError::MalformedLine {
                    line: raw.lines().count(),
                    reason: "append log must end at a complete record boundary".to_string(),
                });
            }
            Ok(raw)
        }
        Err(error) => Err(error),
    }
}

fn read_persistence_text(path: &Path) -> Result<String, EpisodePersistenceError> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() => {
            return Err(unsafe_path(path, "target must not be a symlink"));
        }
        Ok(metadata) if !metadata.is_file() => {
            return Err(unsafe_path(path, "target must be a regular file"));
        }
        Ok(metadata) if metadata.len() > EPISODIC_TEXT_LOG_MAX_BYTES => {
            return Err(EpisodePersistenceError::LogTooLarge {
                path: path.display().to_string(),
                bytes: metadata.len(),
                limit: EPISODIC_TEXT_LOG_MAX_BYTES,
            });
        }
        Ok(_) => {}
        Err(error) => return Err(persistence_io("metadata", path, error)),
    }

    fs::read_to_string(path).map_err(|error| persistence_io("read", path, error))
}

fn create_private_new_file(path: &Path) -> Result<File, EpisodePersistenceError> {
    let mut options = OpenOptions::new();
    options.create_new(true).write(true);
    #[cfg(unix)]
    options.mode(0o600);
    let file = options
        .open(path)
        .map_err(|error| persistence_io("create", path, error))?;
    set_file_private(&file, path)?;
    Ok(file)
}

fn ensure_private_dir(path: &Path) -> Result<(), EpisodePersistenceError> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() => {
            return Err(unsafe_path(path, "directory must not be a symlink"));
        }
        Ok(metadata) if !metadata.is_dir() => {
            return Err(unsafe_path(path, "path must be a directory"));
        }
        Ok(_) => {}
        Err(error) if error.kind() == ErrorKind::NotFound => match create_private_dir(path) {
            Ok(()) => {}
            Err(error) if error.kind() == ErrorKind::AlreadyExists => {
                validate_private_dir_after_race(path)?;
            }
            Err(error) => return Err(persistence_io("create", path, error)),
        },
        Err(error) => return Err(persistence_io("metadata", path, error)),
    }
    set_path_private_dir(path)?;
    Ok(())
}

fn create_private_dir(path: &Path) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        let mut builder = fs::DirBuilder::new();
        builder.mode(0o700);
        builder.create(path)
    }

    #[cfg(not(unix))]
    {
        fs::DirBuilder::new().create(path)
    }
}

fn validate_private_dir_after_race(path: &Path) -> Result<(), EpisodePersistenceError> {
    let metadata =
        fs::symlink_metadata(path).map_err(|error| persistence_io("metadata", path, error))?;
    if metadata.file_type().is_symlink() {
        return Err(unsafe_path(path, "directory must not be a symlink"));
    }
    if !metadata.is_dir() {
        return Err(unsafe_path(path, "path must be a directory"));
    }
    Ok(())
}

fn validate_regular_target_if_exists(path: &Path) -> Result<(), EpisodePersistenceError> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() => {
            Err(unsafe_path(path, "target must not be a symlink"))
        }
        Ok(metadata) if !metadata.is_file() => {
            Err(unsafe_path(path, "target must be a regular file"))
        }
        Ok(_) => Ok(()),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
        Err(error) => Err(persistence_io("metadata", path, error)),
    }
}

fn unsafe_path(path: &Path, reason: impl Into<String>) -> EpisodePersistenceError {
    EpisodePersistenceError::UnsafePath {
        path: path.display().to_string(),
        reason: reason.into(),
    }
}

#[cfg(unix)]
fn set_path_private_dir(path: &Path) -> Result<(), EpisodePersistenceError> {
    use std::os::unix::fs::PermissionsExt;

    fs::set_permissions(path, fs::Permissions::from_mode(0o700))
        .map_err(|error| persistence_io("chmod", path, error))
}

#[cfg(not(unix))]
fn set_path_private_dir(_path: &Path) -> Result<(), EpisodePersistenceError> {
    Ok(())
}

#[cfg(unix)]
fn set_file_private(file: &File, path: &Path) -> Result<(), EpisodePersistenceError> {
    use std::os::unix::fs::PermissionsExt;

    file.set_permissions(fs::Permissions::from_mode(0o600))
        .map_err(|error| persistence_io("chmod", path, error))
}

#[cfg(not(unix))]
fn set_file_private(_file: &File, _path: &Path) -> Result<(), EpisodePersistenceError> {
    Ok(())
}

#[cfg(unix)]
fn set_path_private_file(path: &Path) -> Result<(), EpisodePersistenceError> {
    use std::os::unix::fs::PermissionsExt;

    fs::set_permissions(path, fs::Permissions::from_mode(0o600))
        .map_err(|error| persistence_io("chmod", path, error))
}

#[cfg(not(unix))]
fn set_path_private_file(_path: &Path) -> Result<(), EpisodePersistenceError> {
    Ok(())
}

fn persistence_io(
    action: &'static str,
    path: &Path,
    error: std::io::Error,
) -> EpisodePersistenceError {
    EpisodePersistenceError::Io {
        action,
        path: path.display().to_string(),
        error: error.to_string(),
    }
}

fn parse_persisted_episodes<T>(raw: &str) -> Result<Vec<Episode<T>>, EpisodePersistenceError>
where
    T: FromStr,
    T::Err: fmt::Display,
{
    let mut lines = raw.lines();
    let header = lines.next().ok_or(EpisodePersistenceError::MissingHeader)?;
    if header != "garnet-episodic-v1" {
        return Err(EpisodePersistenceError::UnsupportedHeader(
            header.to_string(),
        ));
    }

    let mut episodes = Vec::new();
    for (idx, line) in lines.enumerate() {
        let line_no = idx + 2;
        let Some((timestamp_text, payload_hex)) = line.split_once('\t') else {
            return Err(EpisodePersistenceError::MalformedLine {
                line: line_no,
                reason: "expected timestamp and hex payload separated by one tab".to_string(),
            });
        };
        if payload_hex.contains('\t') {
            return Err(EpisodePersistenceError::MalformedLine {
                line: line_no,
                reason: "unexpected extra tab in payload".to_string(),
            });
        }
        let timestamp = timestamp_text.parse::<u64>().map_err(|_| {
            EpisodePersistenceError::InvalidTimestamp {
                line: line_no,
                value: timestamp_text.to_string(),
            }
        })?;
        let bytes = hex_decode(payload_hex, line_no)?;
        let value_text =
            String::from_utf8(bytes).map_err(|error| EpisodePersistenceError::InvalidUtf8 {
                line: line_no,
                error: error.to_string(),
            })?;
        let value =
            value_text
                .parse::<T>()
                .map_err(|error| EpisodePersistenceError::InvalidValue {
                    line: line_no,
                    value: value_text.clone(),
                    error: error.to_string(),
                })?;
        episodes.push(Episode {
            timestamp_unix: timestamp,
            value,
        });
    }
    Ok(episodes)
}

fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0x0f) as usize] as char);
    }
    out
}

fn hex_decode(hex: &str, line: usize) -> Result<Vec<u8>, EpisodePersistenceError> {
    if !hex.len().is_multiple_of(2) {
        return Err(EpisodePersistenceError::InvalidHex {
            line,
            value: hex.to_string(),
        });
    }
    let mut bytes = Vec::with_capacity(hex.len() / 2);
    for pair in hex.as_bytes().chunks_exact(2) {
        let hi = hex_nibble(pair[0]).ok_or_else(|| EpisodePersistenceError::InvalidHex {
            line,
            value: hex.to_string(),
        })?;
        let lo = hex_nibble(pair[1]).ok_or_else(|| EpisodePersistenceError::InvalidHex {
            line,
            value: hex.to_string(),
        })?;
        bytes.push((hi << 4) | lo);
    }
    Ok(bytes)
}

fn hex_nibble(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

impl<T> Drop for EpisodeStore<T> {
    fn drop(&mut self) {
        for event in self.events.get_mut().drain(..) {
            if let Some(root) = event.root {
                self.alloc.release_root(root);
            }
        }
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::os::unix::fs::symlink;

    fn temp_project(name: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time after epoch")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!(
            "garnet-memory-{name}-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir_all(&dir).expect("create temp project");
        dir
    }

    #[test]
    fn prepared_cache_backend_rejects_directory_swap_before_write() {
        let project = temp_project("prepared-cache-swap");
        let backend = EpisodicCacheBackend::prepare(&project).expect("prepare backend");
        let episodic_dir = project
            .join(EPISODIC_CACHE_DIR)
            .join(EPISODIC_CACHE_EPISODIC_DIR);
        let moved_dir = project.join("moved-episodic");
        let outside = temp_project("prepared-cache-outside");
        fs::rename(&episodic_dir, &moved_dir).expect("move validated episodic dir");
        symlink(&outside, &episodic_dir).expect("replace episodic dir with symlink");

        let store: EpisodeStore<String> = EpisodeStore::new();
        let result = store.append_prepared_cache_text(&backend, 10, "new".to_string());

        assert!(matches!(
            result,
            Err(EpisodePersistenceError::UnsafePath { .. })
        ));
        assert!(
            !outside.join(EPISODIC_CACHE_LOG_FILE).exists(),
            "prepared backend must not follow a swapped episodic-dir symlink"
        );
        assert!(store.is_empty());
    }
}

#[cfg(all(test, windows))]
mod windows_tests {
    use super::*;

    #[test]
    fn windows_lock_contention_retries_sharing_and_lock_violations() {
        assert!(is_windows_lock_contention(&io::Error::from_raw_os_error(
            32
        )));
        assert!(is_windows_lock_contention(&io::Error::from_raw_os_error(
            33
        )));
        assert!(is_windows_lock_contention(&io::Error::new(
            ErrorKind::PermissionDenied,
            "exclusive lock busy",
        )));
        assert!(!is_windows_lock_contention(&io::Error::new(
            ErrorKind::NotFound,
            "missing lock parent",
        )));
    }
}
