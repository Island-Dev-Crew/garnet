//! Episodic memory: append-only log with timestamp indexing.

use crate::{
    AllocRequest, AllocRootStats, AllocStats, CycleNodeId, HeapKindAllocator, KindAllocator,
    MemoryKind, MemoryPolicy,
};
use std::cell::RefCell;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

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
        let mut out = String::from("garnet-episodic-v1\n");
        for stored in self.events.borrow().iter() {
            out.push_str(&stored.event.timestamp_unix.to_string());
            out.push('\t');
            out.push_str(&hex_encode(stored.event.value.to_string().as_bytes()));
            out.push('\n');
        }

        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)
                    .map_err(|error| persistence_io("create", parent, error))?;
            }
        }

        let tmp = temp_path_for(path);
        fs::write(&tmp, out.as_bytes()).map_err(|error| persistence_io("write", &tmp, error))?;
        fs::rename(&tmp, path).map_err(|error| {
            let _ = fs::remove_file(&tmp);
            persistence_io("rename", path, error)
        })?;
        Ok(())
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
        let raw = fs::read_to_string(path).map_err(|error| persistence_io("read", path, error))?;
        let episodes = parse_persisted_episodes(&raw)?;
        self.replace_events(episodes);
        Ok(())
    }
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
