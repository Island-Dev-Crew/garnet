//! Arena-style working memory: bulk-alloc, bulk-free at scope exit.

use std::cell::RefCell;

/// A reference-counted arena. `T` is whatever value type the caller stores.
/// Typical usage: push items during a scope, drop the whole store at scope
/// exit (the Rust destructor reclaims memory in O(1) amortised).
pub struct WorkingStore<T> {
    items: RefCell<Vec<T>>,
}

impl<T> Default for WorkingStore<T> {
    fn default() -> Self {
        Self {
            items: RefCell::new(Vec::new()),
        }
    }
}

impl<T> WorkingStore<T> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Append an item; returns the dense index (stable until the store is
    /// cleared).
    pub fn push(&self, value: T) -> usize {
        let mut items = self.items.borrow_mut();
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
    }
}

impl<T: Clone> WorkingStore<T> {
    pub fn snapshot(&self) -> Vec<T> {
        self.items.borrow().clone()
    }
}
