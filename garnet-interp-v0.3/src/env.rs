//! Lexically scoped environment with interior mutability for ARC semantics.

use crate::value::Value;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// A lexical scope. Each `Env` has a parent pointer (None for the global
/// scope). Variable lookup walks the chain; `define` and `set` target the
/// innermost scope that holds the binding.
#[derive(Debug)]
pub struct Env {
    vars: RefCell<HashMap<String, Value>>,
    parent: Option<Rc<Env>>,
}

impl Env {
    pub fn new_root() -> Self {
        Self {
            vars: RefCell::new(HashMap::new()),
            parent: None,
        }
    }

    /// Create a nested scope with `parent` as the enclosing lexical scope.
    pub fn new_child(parent: &Rc<Env>) -> Rc<Env> {
        Rc::new(Self {
            vars: RefCell::new(HashMap::new()),
            parent: Some(Rc::clone(parent)),
        })
    }

    /// Define a new binding in the current scope (shadows any outer binding).
    pub fn define(&self, name: &str, value: Value) {
        self.vars.borrow_mut().insert(name.to_string(), value);
    }

    /// Look up a binding starting in the current scope and walking outward.
    pub fn get(&self, name: &str) -> Option<Value> {
        if let Some(v) = self.vars.borrow().get(name) {
            return Some(v.clone());
        }
        self.parent.as_ref().and_then(|p| p.get(name))
    }

    /// Update an existing binding. Returns `false` if the name is unbound.
    pub fn set(&self, name: &str, value: Value) -> bool {
        if self.vars.borrow().contains_key(name) {
            self.vars.borrow_mut().insert(name.to_string(), value);
            return true;
        }
        if let Some(parent) = self.parent.as_ref() {
            return parent.set(name, value);
        }
        false
    }

    /// Whether `name` is bound in this or any enclosing scope.
    pub fn contains(&self, name: &str) -> bool {
        self.vars.borrow().contains_key(name)
            || self.parent.as_ref().is_some_and(|p| p.contains(name))
    }
}
