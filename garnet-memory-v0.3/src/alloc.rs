//! Kind-aware allocator surface for Mnemos stores.
//!
//! This is the first production-facing allocator contract: each store records
//! typed allocation intent through a memory-kind-specific allocator while
//! keeping the existing `Vec` / `BTreeMap` backing stores intact. The trait is
//! object-safe so stores can carry `Arc<dyn KindAllocator>` without forcing a
//! generic allocator parameter through every interpreter-facing type.

use crate::cycle::{
    CycleAllocationMode, CycleAllocatorFixture, CycleCollectReport, CycleGraphError, CycleNodeId,
    CycleScan,
};
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AllocRootStats {
    pub kind: MemoryKind,
    pub roots_created: usize,
    pub active_roots: usize,
    pub roots_released: usize,
    pub buffered_roots: usize,
    pub collected_roots: usize,
}

impl AllocRootStats {
    pub fn new(kind: MemoryKind) -> Self {
        Self {
            kind,
            roots_created: 0,
            active_roots: 0,
            roots_released: 0,
            buffered_roots: 0,
            collected_roots: 0,
        }
    }
}

pub trait KindAllocator: Send + Sync {
    fn kind(&self) -> MemoryKind;
    fn reserve(&self, request: AllocRequest);
    fn reset(&self);
    fn stats(&self) -> AllocStats;

    fn retain_root(&self, _label: &str) -> Option<CycleNodeId> {
        None
    }

    fn release_root(&self, _root: CycleNodeId) -> Option<CycleCollectReport> {
        None
    }

    /// Run any pending allocator-owned cycle-collection work.
    ///
    /// Production allocator-integrated ARC collectors will call this as a
    /// deferred finalization boundary; this reference implementation exposes it
    /// to keep the API surface explicit even while the final runtime path is
    /// still deferred.
    fn collect_roots(&self) -> Option<CycleCollectReport> {
        None
    }

    fn root_stats(&self) -> AllocRootStats {
        AllocRootStats::new(self.kind())
    }
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

#[derive(Debug)]
pub struct CycleAwareKindAllocator {
    kind: MemoryKind,
    stats: Mutex<AllocStats>,
    root_stats: Mutex<AllocRootStats>,
    cycle_fixture: Mutex<CycleAllocatorFixture>,
}

impl CycleAwareKindAllocator {
    pub fn new(kind: MemoryKind, threshold: usize) -> Self {
        Self {
            kind,
            stats: Mutex::new(AllocStats::new(kind)),
            root_stats: Mutex::new(AllocRootStats::new(kind)),
            cycle_fixture: Mutex::new(CycleAllocatorFixture::with_threshold(
                CycleScan::Kind(kind),
                threshold,
            )),
        }
    }

    pub fn shared(kind: MemoryKind, threshold: usize) -> Arc<dyn KindAllocator> {
        Arc::new(Self::new(kind, threshold))
    }

    pub fn allocate_arc(&self, label: impl Into<String>) -> CycleNodeId {
        self.cycle_fixture
            .lock()
            .expect("cycle fixture poisoned")
            .allocate_arc(self.kind, label)
    }

    pub fn allocate_safe(&self, label: impl Into<String>) -> CycleNodeId {
        self.cycle_fixture
            .lock()
            .expect("cycle fixture poisoned")
            .allocate_safe(self.kind, label)
    }

    pub fn add_edge(&self, from: CycleNodeId, to: CycleNodeId) -> Result<(), CycleGraphError> {
        self.cycle_fixture
            .lock()
            .expect("cycle fixture poisoned")
            .add_edge(from, to)
    }

    pub fn remove_edge(
        &self,
        from: CycleNodeId,
        to: CycleNodeId,
    ) -> Result<Option<CycleCollectReport>, CycleGraphError> {
        let mut fixture = self.cycle_fixture.lock().expect("cycle fixture poisoned");
        let report = fixture.remove_edge(from, to)?;
        let buffered_roots = fixture.buffer_len();
        drop(fixture);

        self.record_collection(buffered_roots, &report);
        Ok(report)
    }

    pub fn buffered_roots(&self) -> Vec<CycleNodeId> {
        self.cycle_fixture
            .lock()
            .expect("cycle fixture poisoned")
            .buffered_roots()
    }

    pub fn contains(&self, id: CycleNodeId) -> bool {
        self.cycle_fixture
            .lock()
            .expect("cycle fixture poisoned")
            .contains(id)
    }

    pub fn label(&self, id: CycleNodeId) -> Option<String> {
        self.cycle_fixture
            .lock()
            .expect("cycle fixture poisoned")
            .graph()
            .label(id)
            .map(str::to_owned)
    }

    pub fn allocation_mode(&self, id: CycleNodeId) -> Option<CycleAllocationMode> {
        self.cycle_fixture
            .lock()
            .expect("cycle fixture poisoned")
            .graph()
            .allocation_mode(id)
    }

    fn record_collection(&self, buffered_roots: usize, report: &Option<CycleCollectReport>) {
        let mut stats = self
            .root_stats
            .lock()
            .expect("allocator root stats poisoned");
        stats.buffered_roots = buffered_roots;
        if let Some(report) = report {
            stats.collected_roots += report.collected.len();
        }
    }
}

impl KindAllocator for CycleAwareKindAllocator {
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

    fn retain_root(&self, label: &str) -> Option<CycleNodeId> {
        let mut fixture = self.cycle_fixture.lock().expect("cycle fixture poisoned");
        let id = fixture.allocate_arc(self.kind, label);
        fixture.add_root(id).expect("newly allocated root exists");
        let buffered_roots = fixture.buffer_len();
        drop(fixture);

        let mut stats = self
            .root_stats
            .lock()
            .expect("allocator root stats poisoned");
        stats.roots_created += 1;
        stats.active_roots += 1;
        stats.buffered_roots = buffered_roots;
        Some(id)
    }

    fn release_root(&self, root: CycleNodeId) -> Option<CycleCollectReport> {
        let mut fixture = self.cycle_fixture.lock().expect("cycle fixture poisoned");
        let report = match fixture.release_root(root) {
            Ok(report) => report,
            Err(_) => return None,
        };
        let buffered_roots = fixture.buffer_len();
        drop(fixture);

        self.record_collection(buffered_roots, &report);
        let mut stats = self
            .root_stats
            .lock()
            .expect("allocator root stats poisoned");
        stats.roots_released += 1;
        stats.active_roots = stats.active_roots.saturating_sub(1);
        report
    }

    fn collect_roots(&self) -> Option<CycleCollectReport> {
        let mut fixture = self.cycle_fixture.lock().expect("cycle fixture poisoned");
        let buffered_roots = fixture.buffer_len();
        if buffered_roots == 0 {
            return None;
        }

        let report = Some(fixture.collect_buffered_cycles());
        drop(fixture);

        self.record_collection(0, &report);
        report
    }

    fn root_stats(&self) -> AllocRootStats {
        *self
            .root_stats
            .lock()
            .expect("allocator root stats poisoned")
    }
}
