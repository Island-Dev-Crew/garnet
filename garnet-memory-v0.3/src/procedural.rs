//! Procedural memory: copy-on-write workflow store with version history.

use std::cell::RefCell;
use std::collections::BTreeMap;

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
}

impl<T> Default for WorkflowStore<T> {
    fn default() -> Self {
        Self {
            workflows: RefCell::new(BTreeMap::new()),
        }
    }
}

impl<T: Clone> WorkflowStore<T> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&self, name: impl Into<String>, initial: T) {
        let name = name.into();
        self.workflows.borrow_mut().insert(
            name.clone(),
            Workflow {
                name,
                versions: vec![initial],
            },
        );
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
            w.update(f);
        }
    }
}
