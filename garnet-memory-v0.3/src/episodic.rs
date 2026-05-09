//! Episodic memory: append-only log with timestamp indexing.

use crate::{
    AllocRequest, AllocRootStats, AllocStats, CycleNodeId, HeapKindAllocator, KindAllocator,
    MemoryKind, MemoryPolicy,
};
use std::cell::RefCell;
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

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
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
