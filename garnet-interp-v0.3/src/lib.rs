//! # Garnet Interpreter v0.3
//!
//! Tree-walk interpreter for managed-mode Garnet programs. Rung 3 of the
//! engineering ladder. Evaluates the AST produced by `garnet-parser` v0.3.
//!
//! ## Usage
//!
//! ```no_run
//! use garnet_interp::{Interpreter, Value};
//! let src = r#"def main() { 1 + 2 }"#;
//! let mut interp = Interpreter::new();
//! interp.load_source(src).unwrap();
//! let result = interp.call("main", vec![]).unwrap();
//! assert!(matches!(result, Value::Int(3)));
//! ```

pub mod control;
pub mod env;
pub mod error;
pub mod eval;
pub mod pattern;
pub mod prelude;
pub mod repl;
pub mod stdlib_bridge;
pub mod stmt;
pub mod value;

pub use env::Env;
pub use error::RuntimeError;
pub use prelude::{PRELUDE_SOURCE, PRELUDE_VERSION};
pub use value::Value;

use garnet_parser::ast::{FnDef, Item, Module};
use std::rc::Rc;

/// The top-level interpreter. Owns the global environment and the set of
/// loaded modules. Simple, single-threaded; meant for the REPL and embedded
/// use.
pub struct Interpreter {
    pub global: Rc<Env>,
}

impl Interpreter {
    /// Create a fresh interpreter with the prelude pre-loaded.
    pub fn new() -> Self {
        let global = Rc::new(Env::new_root());
        prelude::install(&global);
        Self { global }
    }

    /// Load a Garnet source string. Parses then registers every top-level item
    /// into the global environment. Raises a `RuntimeError` on parse failure or
    /// evaluation failure at the top level (e.g. evaluating a `let` rhs).
    pub fn load_source(&mut self, src: &str) -> Result<(), RuntimeError> {
        let module =
            garnet_parser::parse_source(src).map_err(|e| RuntimeError::Parse(format!("{e:?}")))?;
        self.load_module(module)
    }

    /// Register a parsed module into the global environment.
    pub fn load_module(&mut self, module: Module) -> Result<(), RuntimeError> {
        for item in module.items {
            self.register_item(item)?;
        }
        Ok(())
    }

    fn register_item(&mut self, item: Item) -> Result<(), RuntimeError> {
        match item {
            Item::Fn(fn_def) => {
                let name = fn_def.name.clone();
                let closure = Value::Fn(Rc::new(value::FnValue {
                    def: fn_def,
                    captured: Rc::clone(&self.global),
                }));
                self.global.define(&name, closure);
            }
            Item::Let(decl) => {
                let val = eval::eval_expr(&decl.value, &self.global)?;
                self.global.define(&decl.name, val);
            }
            Item::Const(decl) => {
                let val = eval::eval_expr(&decl.value, &self.global)?;
                self.global.define(&decl.name, val);
            }
            Item::Memory(decl) => {
                // Kind-aware allocator dispatch (Paper VI Contribution 4):
                // each declared kind gets its purpose-built backing store.
                let backend = value::MemoryBackend::for_kind(decl.kind);
                let store = Value::MemoryStore {
                    kind: decl.kind,
                    name: decl.name.clone(),
                    backend,
                };
                self.global.define(&decl.name, store);
            }
            Item::Struct(s) => {
                let name = s.name.clone();
                self.global
                    .define(&name, Value::Type(Rc::new(value::TypeValue::Struct(s))));
            }
            Item::Enum(e) => {
                let name = e.name.clone();
                self.global
                    .define(&name, Value::Type(Rc::new(value::TypeValue::Enum(e))));
            }
            Item::Trait(_) | Item::Impl(_) | Item::Module(_) | Item::Use(_) | Item::Actor(_) => {
                // Parsed and accepted, but deferred to later rungs for full
                // evaluation. Traits/Impls wait on type-check; Modules/Use
                // need module-system plumbing; Actors need runtime (Rung 6).
            }
        }
        Ok(())
    }

    /// Evaluate a single expression against the global scope.
    pub fn eval_expr_src(&self, src: &str) -> Result<Value, RuntimeError> {
        // Wrap the expression in a fn so the parser's top-level grammar is happy.
        let wrapped = format!("def __repl_expr__() {{ {src} }}");
        let module = garnet_parser::parse_source(&wrapped)
            .map_err(|e| RuntimeError::Parse(format!("{e:?}")))?;
        // Extract the tail expression from the fn body.
        for item in module.items {
            if let Item::Fn(fn_def) = item {
                if let Some(tail) = fn_def.body.tail_expr {
                    return eval::eval_expr(&tail, &self.global);
                }
                // No tail expression — evaluate stmts and return Nil.
                for s in &fn_def.body.stmts {
                    stmt::exec_stmt(s, &self.global)?;
                }
                return Ok(Value::Nil);
            }
        }
        Err(RuntimeError::Message("empty expression".to_string()))
    }

    /// Call a named global function with argument values.
    pub fn call(&self, name: &str, args: Vec<Value>) -> Result<Value, RuntimeError> {
        let callee = self
            .global
            .get(name)
            .ok_or_else(|| RuntimeError::Message(format!("unknown function '{name}'")))?;
        eval::call_value(&callee, args)
    }
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}

/// Extract the function definition for a top-level function by name (for testing).
pub fn find_fn<'a>(module: &'a Module, name: &str) -> Option<&'a FnDef> {
    module.items.iter().find_map(|it| match it {
        Item::Fn(f) if f.name == name => Some(f),
        _ => None,
    })
}
