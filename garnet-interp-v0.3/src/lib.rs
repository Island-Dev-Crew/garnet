//! # Garnet Interpreter v0.3
//!
//! Tree-walk interpreter for managed-mode Garnet programs. Rung 3 of the
//! engineering ladder. Evaluates the AST produced by `garnet-parser` v0.3.
//!
//! ## Usage
//!
//! ```no_run
//! use garnet_interp::{Interpreter, RuntimeError, Value};
//! fn demo() -> Result<(), RuntimeError> {
//!     let src = r#"def main() { 1 + 2 }"#;
//!     let mut interp = Interpreter::new();
//!     interp.load_source(src)?;
//!     let result = interp.call("main", vec![])?;
//!     assert!(matches!(result, Value::Int(3)));
//!     Ok(())
//! }
//! ```

// RB-2 crash-surface sweep: user-facing crates must not unwrap/expect on
// reachable paths. Sanctioned escapes are in-line `// INVARIANT:` allows
// (provably-cannot-fail) and the one documented `// FAIL-CLOSED:` abort
// (machine_key). Test code is exempt via the cfg_attr below.
#![deny(clippy::unwrap_used, clippy::expect_used)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

pub mod control;
pub mod env;
pub mod error;
pub mod eval;
pub mod output;
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

use garnet_parser::ast::{FnDef, Item, Module, TypeExpr};
use std::rc::Rc;

/// The top-level interpreter. Owns the global environment and the set of
/// loaded modules. Simple, single-threaded; meant for the REPL and embedded
/// use.
pub struct Interpreter {
    pub global: Rc<Env>,
    /// When true (the default), host-authority primitives reached with no
    /// `@caps` frame are refused for this instance's load/eval/call operations
    /// (deny-by-default embedder posture). `new_permissive()` clears it.
    strict: bool,
}

impl Interpreter {
    /// Create a fresh interpreter with the prelude pre-loaded.
    ///
    /// Strict-by-default (S114 acceptance, cond. #5): an embedder that loads and
    /// runs untrusted Garnet source through this interpreter refuses undeclared
    /// host authority even when the source is loaded/evaluated outside a program
    /// entry frame. Use [`Self::new_permissive`] for the legacy fail-open
    /// direct-call behavior (Rust unit tests, benches, trusted internal loads).
    pub fn new() -> Self {
        Self::with_strict(true)
    }

    /// Create a fresh interpreter that ALLOWS host-authority primitives reached
    /// with no `@caps` frame (the pre-S114 direct-call default). This is the
    /// explicit opt-out from strict-by-default; a permissive instance still
    /// cannot escape a process that has latched the global strict-no-frame gate
    /// (`eval::set_strict_no_frame`) — that latch always dominates.
    pub fn new_permissive() -> Self {
        Self::with_strict(false)
    }

    fn with_strict(strict: bool) -> Self {
        let global = Rc::new(Env::new_root());
        prelude::install(&global);
        Self { global, strict }
    }

    /// The strict scope for this instance's operations, or `None` when permissive.
    fn strict_scope(&self) -> Option<eval::StrictScope> {
        self.strict.then(eval::StrictScope::enter)
    }

    /// Load a Garnet source string under the default edition (v1.0). Parses then
    /// registers every top-level item into the global environment. Raises a
    /// `RuntimeError` on parse failure or top-level evaluation failure (e.g.
    /// evaluating a `let` rhs).
    pub fn load_source(&mut self, src: &str) -> Result<(), RuntimeError> {
        self.load_source_with_edition(src, garnet_parser::Edition::default())
    }

    /// Load a Garnet source string under an explicit [`garnet_parser::Edition`].
    /// Editions gate only the lexical surface, so for source valid in the
    /// default edition the registered items are identical to [`Self::load_source`].
    pub fn load_source_with_edition(
        &mut self,
        src: &str,
        edition: garnet_parser::Edition,
    ) -> Result<(), RuntimeError> {
        let module = garnet_parser::parse_source_with_edition(src, edition)
            .map_err(|e| RuntimeError::Parse(format!("{e:?}")))?;
        self.load_module(module)
    }

    /// Load source while holding the named program entry's `@caps` frame.
    ///
    /// Top-level `let`/`const` initializers execute during module registration,
    /// before `call_entry` can install `main`'s capability frame. This method
    /// parses first, extracts the entry annotations, then loads the module under
    /// that frame so load-time host authority is checked against the same entry
    /// declaration as body-time authority.
    pub fn load_source_with_entry_caps(
        &mut self,
        src: &str,
        entry: &str,
    ) -> Result<(), RuntimeError> {
        self.load_source_with_edition_entry_caps(src, garnet_parser::Edition::default(), entry)
    }

    /// Edition-aware variant of [`Self::load_source_with_entry_caps`].
    pub fn load_source_with_edition_entry_caps(
        &mut self,
        src: &str,
        edition: garnet_parser::Edition,
        entry: &str,
    ) -> Result<(), RuntimeError> {
        let module = garnet_parser::parse_source_with_edition(src, edition)
            .map_err(|e| RuntimeError::Parse(format!("{e:?}")))?;
        self.load_module_with_entry_caps(module, entry)
    }

    /// Register a parsed module into the global environment.
    pub fn load_module(&mut self, module: Module) -> Result<(), RuntimeError> {
        // Strict-by-default: top-level `let`/`const` initializers evaluate here
        // at no active frame; a strict instance denies undeclared host authority
        // in them. Harmless on the framed load path (an entry frame is already
        // active, so the no-frame branch is never reached).
        let _strict = self.strict_scope();
        // S114-FIX-2: validate every `@max_depth(N)` annotation's range BEFORE
        // registering anything — top-level fns, impl methods, and nested-module
        // fns alike — so `garnet run` refuses an out-of-range bound anywhere
        // `garnet check` does, on both backends, even when the function is never
        // called. A pre-pass means a bad bound fails the load cleanly with
        // nothing partially registered.
        validate_module_max_depth(&module.items)?;
        for item in module.items {
            self.register_item(item)?;
        }
        Ok(())
    }

    /// Register a parsed module under a program-entry capability frame.
    pub fn load_module_with_entry_caps(
        &mut self,
        module: Module,
        entry: &str,
    ) -> Result<(), RuntimeError> {
        let entry_annotations = module
            .items
            .iter()
            .find_map(|item| match item {
                Item::Fn(fn_def) if fn_def.name == entry => Some(fn_def.annotations.clone()),
                _ => None,
            })
            .unwrap_or_default();
        let _entry_caps = eval::enter_entry_caps_for_annotations(&entry_annotations);
        self.load_module(module)
    }

    fn register_item(&mut self, item: Item) -> Result<(), RuntimeError> {
        match item {
            Item::Fn(fn_def) => {
                let name = fn_def.name.clone();
                let closure = Value::Fn(Rc::new(value::FnValue {
                    def: fn_def,
                    captured: Rc::clone(&self.global),
                    is_block: false,
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
            Item::Protocol(protocol) => {
                self.global.define_protocol(protocol);
            }
            Item::Actor(actor) => {
                let name = actor.name.clone();
                self.global.define(&name, Value::ActorType(Rc::new(actor)));
            }
            Item::Impl(impl_block) => {
                let type_name = named_type_name(&impl_block.target).map(str::to_string);
                let trait_name = impl_block.trait_ty.as_ref().and_then(named_type_name);
                if let (Some(type_name), Some(trait_name)) = (type_name.as_ref(), trait_name) {
                    if value::has_dynamic_annotation(&impl_block.annotations) {
                        for method in impl_block.methods {
                            let method_name = method.name.clone();
                            let closure = Value::Fn(Rc::new(value::FnValue {
                                def: method,
                                captured: Rc::clone(&self.global),
                                is_block: false,
                            }));
                            self.global.define_dynamic_impl_method(
                                type_name,
                                trait_name,
                                &method_name,
                                closure,
                            );
                        }
                    }
                    return Ok(());
                }
                if let Some(type_name) = type_name {
                    for method in impl_block.methods {
                        let method_name = method.name.clone();
                        let closure = Value::Fn(Rc::new(value::FnValue {
                            def: method,
                            captured: Rc::clone(&self.global),
                            is_block: false,
                        }));
                        self.global
                            .define_impl_method(&type_name, &method_name, closure);
                    }
                }
            }
            Item::Trait(_) | Item::Module(_) | Item::Use(_) => {
                // Parsed and accepted, but deferred to later rungs for full
                // evaluation. Traits wait on type-check;
                // Modules/Use need module-system plumbing.
            }
        }
        Ok(())
    }

    /// Evaluate a single expression against the global scope.
    pub fn eval_expr_src(&self, src: &str) -> Result<Value, RuntimeError> {
        // Strict-by-default: a bare expression evaluates with no active frame.
        let _strict = self.strict_scope();
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
        // Strict-by-default: an embedded `call` pushes a managed frame only if
        // the callee is a `def`; a bare `fn` reaches host authority frame-less.
        let _strict = self.strict_scope();
        eval::call_value(&callee, args)
    }

    /// The names bound in the global environment (prelude builtins + every
    /// top-level item loaded so far). Read-only introspection used by the CLI
    /// REPL (RB-7) to feed tab-completion from the live session. No semantic
    /// effect; the bytecode/VM paths are unaffected.
    #[must_use]
    pub fn live_binding_names(&self) -> Vec<String> {
        self.global.local_names()
    }

    /// The function value bound to `name` in the global environment, if any.
    /// Lets the REPL's `?doc` read a user function's arity + `@caps` surface
    /// off the loaded definition. Read-only.
    #[must_use]
    pub fn lookup_binding(&self, name: &str) -> Option<Value> {
        self.global.get(name)
    }

    /// Call a named global function as the program entry point.
    ///
    /// Unlike embedded `call`, this installs a capability frame from the entry
    /// function's `@caps(...)` annotations before dispatch. That keeps
    /// `garnet run --interp` authority-checked even when `main` is safe-mode
    /// (`fn`) and would not otherwise push a managed frame.
    pub fn call_entry(&self, name: &str, args: Vec<Value>) -> Result<Value, RuntimeError> {
        let callee = self
            .global
            .get(name)
            .ok_or_else(|| RuntimeError::Message(format!("unknown function '{name}'")))?;
        // Harmless under strict: the entry frame installed below means host
        // calls run with an active frame; the strict scope only matters if the
        // entry has no `@caps` and reaches host authority before the frame.
        let _strict = self.strict_scope();
        eval::call_value_with_entry_caps(&callee, args)
    }

    /// Install the program-entry `@caps` frame for `name` **without** running the
    /// body (S100), returning an RAII scope the caller holds for the duration of
    /// the run. The bytecode VM uses this so its fallback path enforces the S92
    /// program-entry capability gate identically to `call_entry` on `--interp`;
    /// without it, undeclared subprocess authority laundered through a helper
    /// traps under `--interp` but is allowed under `--vm`. Returns `None` if
    /// `name` is not a bound function value (no entry annotations to install).
    pub fn enter_entry_caps_frame(&self, name: &str) -> Option<eval::EntryCapsScope> {
        let callee = self.global.get(name)?;
        eval::enter_entry_caps_for(&callee)
    }
}

/// Recursively validate the `@max_depth(N)` range on every function the module
/// hosts — top-level `Item::Fn`, `Item::Impl` methods, and functions inside
/// nested `Item::Module`s — mirroring `garnet check`'s coverage so an invalid
/// bound is refused at load regardless of where it is declared or whether the
/// function is ever called (S114-FIX-2). Without the recursion, an out-of-range
/// `@max_depth` on an impl method or a nested-module function passed `garnet run`
/// while `garnet check` rejected it.
fn validate_module_max_depth(items: &[Item]) -> Result<(), RuntimeError> {
    for item in items {
        match item {
            Item::Fn(f) => eval::validate_max_depth_annotation(&f.annotations, &f.name)?,
            Item::Impl(block) => {
                for m in &block.methods {
                    eval::validate_max_depth_annotation(&m.annotations, &m.name)?;
                }
            }
            Item::Module(m) => validate_module_max_depth(&m.items)?,
            _ => {}
        }
    }
    Ok(())
}

fn named_type_name(ty: &TypeExpr) -> Option<&str> {
    match ty {
        TypeExpr::Named { path, .. } => path.last().map(String::as_str),
        _ => None,
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
