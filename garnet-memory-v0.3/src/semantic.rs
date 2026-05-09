//! Semantic memory: a vector-indexed fact store with cosine similarity search.

use crate::{AllocRequest, AllocStats, HeapKindAllocator, KindAllocator, MemoryKind, MemoryPolicy};
use std::cell::RefCell;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct Fact<T> {
    pub embedding: Vec<f32>,
    pub value: T,
    pub inserted_unix: u64,
    pub importance: f64,
}

pub struct VectorIndex<T> {
    facts: RefCell<Vec<Fact<T>>>,
    alloc: Arc<dyn KindAllocator>,
    policy: MemoryPolicy,
    eviction_enabled: bool,
}

impl<T> Default for VectorIndex<T> {
    fn default() -> Self {
        Self::with_policy_state(MemoryPolicy::default_for(MemoryKind::Semantic), false)
    }
}

impl<T> VectorIndex<T> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_policy(policy: MemoryPolicy) -> Self {
        Self::with_policy_state(policy, true)
    }

    pub fn with_allocator(alloc: Arc<dyn KindAllocator>) -> Self {
        Self::with_policy_allocator_state(
            MemoryPolicy::default_for(MemoryKind::Semantic),
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
            HeapKindAllocator::shared(MemoryKind::Semantic),
            eviction_enabled,
        )
    }

    fn with_policy_allocator_state(
        policy: MemoryPolicy,
        alloc: Arc<dyn KindAllocator>,
        eviction_enabled: bool,
    ) -> Self {
        assert_eq!(alloc.kind(), MemoryKind::Semantic);
        Self {
            facts: RefCell::new(Vec::new()),
            alloc,
            policy,
            eviction_enabled,
        }
    }

    pub fn insert(&self, embedding: Vec<f32>, value: T) {
        self.insert_with_importance_at(embedding, value, 1.0, unix_now());
    }

    pub fn insert_with_importance_at(
        &self,
        embedding: Vec<f32>,
        value: T,
        importance: f64,
        inserted_unix: u64,
    ) {
        self.alloc.reserve(AllocRequest::for_items::<Fact<T>>(1));
        let mut facts = self.facts.borrow_mut();
        facts.reserve(1);
        facts.push(Fact {
            embedding,
            value,
            inserted_unix,
            importance,
        });
    }

    pub fn len(&self) -> usize {
        self.facts.borrow().len()
    }

    pub fn is_empty(&self) -> bool {
        self.facts.borrow().is_empty()
    }

    pub fn allocator_stats(&self) -> AllocStats {
        self.alloc.stats()
    }

    fn evict_for_query(&self, query: &[f32], now: u64) {
        if !self.eviction_enabled {
            return;
        }
        let mut facts = self.facts.borrow_mut();
        facts.retain(|fact| {
            let relevance = cosine_sim(&fact.embedding, query) as f64;
            let age = now.saturating_sub(fact.inserted_unix) as f64;
            self.policy
                .should_retain(self.policy.score(relevance, age, fact.importance))
        });

        let high_water = self.policy.compaction_high_water;
        if high_water > 0 && facts.len() > high_water {
            facts.sort_by(|a, b| {
                let score_a = self.policy.score(
                    cosine_sim(&a.embedding, query) as f64,
                    now.saturating_sub(a.inserted_unix) as f64,
                    a.importance,
                );
                let score_b = self.policy.score(
                    cosine_sim(&b.embedding, query) as f64,
                    now.saturating_sub(b.inserted_unix) as f64,
                    b.importance,
                );
                score_b
                    .partial_cmp(&score_a)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            facts.truncate(high_water);
        }
    }
}

impl<T: Clone> VectorIndex<T> {
    /// Top-k cosine-similarity search. Returns (score, value) pairs sorted
    /// descending. This is the naive O(n·d) baseline — good enough for the
    /// reference implementation.
    pub fn search(&self, query: &[f32], k: usize) -> Vec<(f32, T)> {
        self.evict_for_query(query, unix_now());
        let facts = self.facts.borrow();
        let mut scored: Vec<(f32, T)> = facts
            .iter()
            .map(|f| (cosine_sim(&f.embedding, query), f.value.clone()))
            .collect();
        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(k);
        scored
    }
}

fn cosine_sim(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    let mut dot = 0.0f32;
    let mut na = 0.0f32;
    let mut nb = 0.0f32;
    for i in 0..a.len() {
        dot += a[i] * b[i];
        na += a[i] * a[i];
        nb += b[i] * b[i];
    }
    let denom = (na.sqrt() * nb.sqrt()).max(1e-9);
    dot / denom
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}
