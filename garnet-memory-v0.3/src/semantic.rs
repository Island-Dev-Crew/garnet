//! Semantic memory: a vector-indexed fact store with cosine similarity search.

use std::cell::RefCell;

#[derive(Debug, Clone)]
pub struct Fact<T> {
    pub embedding: Vec<f32>,
    pub value: T,
}

pub struct VectorIndex<T> {
    facts: RefCell<Vec<Fact<T>>>,
}

impl<T> Default for VectorIndex<T> {
    fn default() -> Self {
        Self {
            facts: RefCell::new(Vec::new()),
        }
    }
}

impl<T> VectorIndex<T> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&self, embedding: Vec<f32>, value: T) {
        self.facts.borrow_mut().push(Fact { embedding, value });
    }

    pub fn len(&self) -> usize {
        self.facts.borrow().len()
    }

    pub fn is_empty(&self) -> bool {
        self.facts.borrow().is_empty()
    }
}

impl<T: Clone> VectorIndex<T> {
    /// Top-k cosine-similarity search. Returns (score, value) pairs sorted
    /// descending. This is the naive O(n·d) baseline — good enough for the
    /// reference implementation.
    pub fn search(&self, query: &[f32], k: usize) -> Vec<(f32, T)> {
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
