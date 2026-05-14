//! Procedural memory: copy-on-write workflow store with version history.

use crate::{
    AllocRequest, AllocRootStats, AllocStats, CycleNodeId, HeapKindAllocator, KindAllocator,
    MemoryKind,
};
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Workflow<T> {
    pub name: String,
    pub versions: Vec<T>,
}

impl<T: Clone> Workflow<T> {
    pub fn current(&self) -> Option<&T> {
        self.versions.last()
    }

    pub fn version(&self, n: usize) -> Option<&T> {
        self.versions.get(n)
    }

    /// Create a new version that is a copy of the latest, with the provided
    /// transformation applied. The original version is retained for rollback.
    pub fn update<F>(&mut self, f: F)
    where
        F: FnOnce(T) -> T,
    {
        let base = self.versions.last().cloned();
        if let Some(b) = base {
            self.versions.push(f(b));
        }
    }
}

pub struct WorkflowStore<T> {
    workflows: RefCell<BTreeMap<String, Workflow<T>>>,
    alloc: Arc<dyn KindAllocator>,
    roots: RefCell<BTreeMap<String, CycleNodeId>>,
}

impl<T> Default for WorkflowStore<T> {
    fn default() -> Self {
        Self::with_allocator(HeapKindAllocator::shared(MemoryKind::Procedural))
    }
}

impl<T> WorkflowStore<T> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_allocator(alloc: Arc<dyn KindAllocator>) -> Self {
        assert_eq!(alloc.kind(), MemoryKind::Procedural);
        Self {
            workflows: RefCell::new(BTreeMap::new()),
            alloc,
            roots: RefCell::new(BTreeMap::new()),
        }
    }

    pub fn allocator_stats(&self) -> AllocStats {
        self.alloc.stats()
    }

    pub fn allocator_root_stats(&self) -> AllocRootStats {
        self.alloc.root_stats()
    }
}

impl<T: Clone> WorkflowStore<T> {
    pub fn register(&self, name: impl Into<String>, initial: T) {
        let name = name.into();
        self.alloc
            .reserve(AllocRequest::for_items::<Workflow<T>>(1));
        if let Some(root) = self.roots.borrow_mut().remove(&name) {
            self.alloc.release_root(root);
        }
        let root = self.alloc.retain_root("procedural:workflow");
        self.workflows.borrow_mut().insert(
            name.clone(),
            Workflow {
                name: name.clone(),
                versions: vec![initial],
            },
        );
        if let Some(root) = root {
            self.roots.borrow_mut().insert(name, root);
        }
        self.alloc.collect_roots();
    }

    pub fn find(&self, name: &str) -> Option<Workflow<T>> {
        self.workflows.borrow().get(name).cloned()
    }

    pub fn replay(&self, name: &str, version: usize) -> Option<T> {
        self.workflows
            .borrow()
            .get(name)
            .and_then(|w| w.version(version).cloned())
    }

    pub fn update<F>(&self, name: &str, f: F)
    where
        F: FnOnce(T) -> T,
    {
        if let Some(w) = self.workflows.borrow_mut().get_mut(name) {
            self.alloc.reserve(AllocRequest::for_items::<T>(1));
            w.update(f);
        }
    }
}

impl<T> Drop for WorkflowStore<T> {
    fn drop(&mut self) {
        let roots = std::mem::take(self.roots.get_mut());
        for root in roots.into_values() {
            self.alloc.release_root(root);
        }
        self.alloc.collect_roots();
    }
}
