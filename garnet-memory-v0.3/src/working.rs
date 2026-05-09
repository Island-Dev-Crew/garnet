//! Arena-style working memory: bulk-alloc, bulk-free at scope exit.

use crate::{AllocRequest, AllocStats, HeapKindAllocator, KindAllocator, MemoryKind};
use std::cell::RefCell;
use std::sync::Arc;

/// A reference-counted arena. `T` is whatever value type the caller stores.
/// Typical usage: push items during a scope, drop the whole store at scope
/// exit (the Rust destructor reclaims memory in O(1) amortised).
pub struct WorkingStore<T> {
    items: RefCell<Vec<T>>,
    alloc: Arc<dyn KindAllocator>,
}

impl<T> Default for WorkingStore<T> {
    fn default() -> Self {
        Self::with_allocator(HeapKindAllocator::shared(MemoryKind::Working))
    }
}

impl<T> WorkingStore<T> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_allocator(alloc: Arc<dyn KindAllocator>) -> Self {
        assert_eq!(alloc.kind(), MemoryKind::Working);
        Self {
            items: RefCell::new(Vec::new()),
            alloc,
        }
    }

    /// Append an item; returns the dense index (stable until the store is
    /// cleared).
    pub fn push(&self, value: T) -> usize {
        self.alloc.reserve(AllocRequest::for_items::<T>(1));
        let mut items = self.items.borrow_mut();
        items.reserve(1);
        items.push(value);
        items.len() - 1
    }

    pub fn len(&self) -> usize {
        self.items.borrow().len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.borrow().is_empty()
    }

    /// Apply a closure to the element at the given index.
    pub fn with<F, R>(&self, index: usize, f: F) -> Option<R>
    where
        F: FnOnce(&T) -> R,
    {
        self.items.borrow().get(index).map(f)
    }

    /// Drop all stored values, reclaiming memory.
    pub fn clear(&self) {
        self.items.borrow_mut().clear();
        self.alloc.reset();
    }

    pub fn allocator_stats(&self) -> AllocStats {
        self.alloc.stats()
    }
}

impl<T: Clone> WorkingStore<T> {
    pub fn snapshot(&self) -> Vec<T> {
        self.items.borrow().clone()
    }
}
