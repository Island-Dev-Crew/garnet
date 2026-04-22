//! Episodic memory: append-only log with timestamp indexing.

use std::cell::RefCell;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct Episode<T> {
    pub timestamp_unix: u64,
    pub value: T,
}

pub struct EpisodeStore<T> {
    events: RefCell<Vec<Episode<T>>>,
}

impl<T> Default for EpisodeStore<T> {
    fn default() -> Self {
        Self {
            events: RefCell::new(Vec::new()),
        }
    }
}

impl<T> EpisodeStore<T> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Append an event tagged with the current system time.
    pub fn append(&self, value: T) {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        self.events.borrow_mut().push(Episode {
            timestamp_unix: ts,
            value,
        });
    }

    /// Append with an explicit timestamp (useful for replay and testing).
    pub fn append_at(&self, timestamp: u64, value: T) {
        self.events.borrow_mut().push(Episode {
            timestamp_unix: timestamp,
            value,
        });
    }

    pub fn len(&self) -> usize {
        self.events.borrow().len()
    }

    pub fn is_empty(&self) -> bool {
        self.events.borrow().is_empty()
    }
}

impl<T: Clone> EpisodeStore<T> {
    /// Return the N most recent events (or all if N > len).
    pub fn recent(&self, n: usize) -> Vec<Episode<T>> {
        let events = self.events.borrow();
        let start = events.len().saturating_sub(n);
        events[start..].to_vec()
    }

    /// Return events whose timestamp ≥ since.
    pub fn since(&self, since: u64) -> Vec<Episode<T>> {
        self.events
            .borrow()
            .iter()
            .filter(|e| e.timestamp_unix >= since)
            .cloned()
            .collect()
    }

    pub fn snapshot(&self) -> Vec<Episode<T>> {
        self.events.borrow().clone()
    }
}
