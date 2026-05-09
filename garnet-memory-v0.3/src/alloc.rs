//! Kind-aware allocator surface for Mnemos stores.
//!
//! This is the first production-facing allocator contract: each store records
//! typed allocation intent through a memory-kind-specific allocator while
//! keeping the existing `Vec` / `BTreeMap` backing stores intact. The trait is
//! object-safe so stores can carry `Arc<dyn KindAllocator>` without forcing a
//! generic allocator parameter through every interpreter-facing type.

use crate::policy::MemoryKind;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AllocRequest {
    pub items: usize,
    pub item_size: usize,
    pub align: usize,
}

impl AllocRequest {
    pub fn for_items<T>(items: usize) -> Self {
        Self {
            items,
            item_size: std::mem::size_of::<T>(),
            align: std::mem::align_of::<T>(),
        }
    }

    fn reserved_bytes(self) -> usize {
        self.items.saturating_mul(self.item_size)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AllocStats {
    pub kind: MemoryKind,
    pub allocations: usize,
    pub allocated_items: usize,
    pub bytes_reserved: usize,
    pub resets: usize,
}

impl AllocStats {
    pub fn new(kind: MemoryKind) -> Self {
        Self {
            kind,
            allocations: 0,
            allocated_items: 0,
            bytes_reserved: 0,
            resets: 0,
        }
    }
}

pub trait KindAllocator: Send + Sync {
    fn kind(&self) -> MemoryKind;
    fn reserve(&self, request: AllocRequest);
    fn reset(&self);
    fn stats(&self) -> AllocStats;
}

#[derive(Debug)]
pub struct HeapKindAllocator {
    kind: MemoryKind,
    stats: Mutex<AllocStats>,
}

impl HeapKindAllocator {
    pub fn new(kind: MemoryKind) -> Self {
        Self {
            kind,
            stats: Mutex::new(AllocStats::new(kind)),
        }
    }

    pub fn shared(kind: MemoryKind) -> Arc<dyn KindAllocator> {
        Arc::new(Self::new(kind))
    }
}

impl KindAllocator for HeapKindAllocator {
    fn kind(&self) -> MemoryKind {
        self.kind
    }

    fn reserve(&self, request: AllocRequest) {
        let mut stats = self.stats.lock().expect("allocator stats poisoned");
        stats.allocations += 1;
        stats.allocated_items += request.items;
        stats.bytes_reserved += request.reserved_bytes();
    }

    fn reset(&self) {
        let mut stats = self.stats.lock().expect("allocator stats poisoned");
        stats.resets += 1;
    }

    fn stats(&self) -> AllocStats {
        *self.stats.lock().expect("allocator stats poisoned")
    }
}
