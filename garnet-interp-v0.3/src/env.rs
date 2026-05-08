//! Lexically scoped environment with interior mutability for ARC semantics.

use crate::value::Value;
use garnet_parser::ast::ProtocolDef;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// A lexical scope. Each `Env` has a parent pointer (None for the global
/// scope). Variable lookup walks the chain; `define` and `set` target the
/// innermost scope that holds the binding.
#[derive(Debug)]
pub struct Env {
    vars: RefCell<HashMap<String, Value>>,
    protocols: RefCell<HashMap<String, ProtocolDef>>,
    impl_methods: RefCell<HashMap<String, HashMap<String, Value>>>,
    active_block: RefCell<Option<Value>>,
    parent: Option<Rc<Env>>,
}

impl Env {
    pub fn new_root() -> Self {
        Self {
            vars: RefCell::new(HashMap::new()),
            protocols: RefCell::new(HashMap::new()),
            impl_methods: RefCell::new(HashMap::new()),
            active_block: RefCell::new(None),
            parent: None,
        }
    }

    /// Create a nested scope with `parent` as the enclosing lexical scope.
    pub fn new_child(parent: &Rc<Env>) -> Rc<Env> {
        Rc::new(Self {
            vars: RefCell::new(HashMap::new()),
            protocols: RefCell::new(HashMap::new()),
            impl_methods: RefCell::new(HashMap::new()),
            active_block: RefCell::new(None),
            parent: Some(Rc::clone(parent)),
        })
    }

    /// Bind the implicit block for this call frame.
    pub fn set_active_block(&self, block: Value) {
        *self.active_block.borrow_mut() = Some(block);
    }

    /// Look up the implicit block for `yield`, walking outward through nested
    /// lexical scopes created inside the same call frame.
    pub fn active_block(&self) -> Option<Value> {
        self.active_block
            .borrow()
            .clone()
            .or_else(|| self.parent.as_ref().and_then(|p| p.active_block()))
    }

    /// Define a new binding in the current scope (shadows any outer binding).
    pub fn define(&self, name: &str, value: Value) {
        self.vars.borrow_mut().insert(name.to_string(), value);
    }

    pub fn define_protocol(&self, protocol: ProtocolDef) {
        self.protocols
            .borrow_mut()
            .insert(protocol.name.clone(), protocol);
    }

    /// Look up a binding starting in the current scope and walking outward.
    pub fn get(&self, name: &str) -> Option<Value> {
        if let Some(v) = self.vars.borrow().get(name) {
            return Some(v.clone());
        }
        self.parent.as_ref().and_then(|p| p.get(name))
    }

    pub fn get_protocol(&self, name: &str) -> Option<ProtocolDef> {
        if let Some(protocol) = self.protocols.borrow().get(name) {
            return Some(protocol.clone());
        }
        self.parent.as_ref().and_then(|p| p.get_protocol(name))
    }

    pub fn define_impl_method(&self, type_name: &str, method_name: &str, method: Value) {
        self.impl_methods
            .borrow_mut()
            .entry(type_name.to_string())
            .or_default()
            .insert(method_name.to_string(), method);
    }

    pub fn get_impl_method(&self, type_name: &str, method_name: &str) -> Option<Value> {
        if let Some(method) = self
            .impl_methods
            .borrow()
            .get(type_name)
            .and_then(|methods| methods.get(method_name))
        {
            return Some(method.clone());
        }
        self.parent
            .as_ref()
            .and_then(|p| p.get_impl_method(type_name, method_name))
    }

    pub fn has_impl_method(&self, type_name: &str, method_name: &str) -> bool {
        self.get_impl_method(type_name, method_name).is_some()
    }

    pub fn impl_method_names(&self, type_name: &str) -> Vec<String> {
        let mut names = self
            .impl_methods
            .borrow()
            .get(type_name)
            .map(|methods| methods.keys().cloned().collect::<Vec<_>>())
            .unwrap_or_default();
        if let Some(parent) = self.parent.as_ref() {
            names.extend(parent.impl_method_names(type_name));
        }
        names
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
