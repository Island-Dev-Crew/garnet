//! Episodic memory: append-only log with timestamp indexing.

use crate::{AllocRequest, AllocStats, HeapKindAllocator, KindAllocator, MemoryKind, MemoryPolicy};
use std::cell::RefCell;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct Episode<T> {
    pub timestamp_unix: u64,
    pub value: T,
}

pub struct EpisodeStore<T> {
    events: RefCell<Vec<Episode<T>>>,
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
        events.push(Episode {
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

    pub fn allocator_stats(&self) -> AllocStats {
        self.alloc.stats()
    }

    fn evict_at(&self, now: u64) {
        if !self.eviction_enabled {
            return;
        }
        let mut events = self.events.borrow_mut();
        events.retain(|event| {
            let age = now.saturating_sub(event.timestamp_unix) as f64;
            self.policy.should_retain(self.policy.score(1.0, age, 1.0))
        });
        let high_water = self.policy.compaction_high_water;
        if high_water > 0 && events.len() > high_water {
            let drop_count = events.len() - high_water;
            events.drain(0..drop_count);
        }
    }
}

impl<T: Clone> EpisodeStore<T> {
    /// Return the N most recent events (or all if N > len).
    pub fn recent(&self, n: usize) -> Vec<Episode<T>> {
        self.evict_at(unix_now());
        let events = self.events.borrow();
        let start = events.len().saturating_sub(n);
        events[start..].to_vec()
    }

    /// Return events whose timestamp ≥ since.
    pub fn since(&self, since: u64) -> Vec<Episode<T>> {
        self.evict_at(unix_now());
        self.events
            .borrow()
            .iter()
            .filter(|e| e.timestamp_unix >= since)
            .cloned()
            .collect()
    }

    pub fn snapshot(&self) -> Vec<Episode<T>> {
        self.evict_at(unix_now());
        self.events.borrow().clone()
    }
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}
