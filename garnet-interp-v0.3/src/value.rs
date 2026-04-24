//! Runtime values for the Garnet managed-mode interpreter.
//!
//! Values are reference-counted (`Rc`) for Ruby-like sharing. Heavy structures
//! (arrays, maps) use `Rc<RefCell<_>>` for interior mutability. This is the
//! ARC-with-interior-mutability story from Mini-Spec §8.2.

use crate::env::Env;
use crate::error::RuntimeError;
use garnet_parser::ast::{EnumDef, FnDef, MemoryKind, Param, StructDef};
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

/// A runtime value. All managed-mode values are represented here.
#[derive(Clone)]
pub enum Value {
    Nil,
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(Rc<String>),
    Symbol(Rc<String>),
    Array(Rc<RefCell<Vec<Value>>>),
    Map(Rc<RefCell<BTreeMap<String, Value>>>),
    Range {
        start: i64,
        end: i64,
        inclusive: bool,
    },

    /// A user-defined function or closure.
    Fn(Rc<FnValue>),
    /// A native function (Rust-implemented built-in).
    NativeFn(Rc<NativeFnValue>),

    /// A struct or enum *type* (used at construction sites and in patterns).
    Type(Rc<TypeValue>),
    /// A struct instance: (path-or-type-name, field values).
    Struct {
        name: Rc<String>,
        fields: Rc<RefCell<BTreeMap<String, Value>>>,
    },
    /// An enum-variant instance: (enum path, variant name, arguments).
    Variant {
        path: Rc<Vec<String>>,
        variant: Rc<String>,
        fields: Rc<Vec<Value>>,
    },

    /// A first-class memory-store handle (working/episodic/semantic/procedural).
    /// As of v3.2 the real backing is supplied by `garnet_memory`; the kind +
    /// name fields preserve display semantics, while `backend` holds the live
    /// store. Multiple variable bindings to the same memory unit share the same
    /// `Rc` backend, mirroring the ARC sharing the rest of the interp uses.
    MemoryStore {
        kind: MemoryKind,
        name: String,
        backend: MemoryBackend,
    },

    /// A tuple (fixed-size, heterogeneous).
    Tuple(Rc<Vec<Value>>),
}

/// Runtime tag on every memory-kind handle (v3.3 Security Layer 1 —
/// hardening pattern **KindGuard**). Non-sequential `u8` values — 0x57
/// 'W', 0x45 'E', 0x53 'S', 0x50 'P' — so memory corruption is loud
/// (a zeroed or random byte doesn't accidentally alias a valid tag).
///
/// The outer `MemoryBackend` enum provides compile-time kind safety
/// today; `KindTag` is the **defense in depth** that survives any
/// future IR lowering that discards the enum discriminant. Every
/// prim that operates on a `MemoryBackend` validates the tag before
/// dispatch, so a `Value::MemoryStore { kind: Working, backend:
/// MemoryBackend::Semantic(_) }` (constructed via direct struct-init,
/// bypassing `for_kind`) fails with a clear `RuntimeError` instead of
/// silently invoking the wrong store's methods.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KindTag {
    Working = 0x57,
    Episodic = 0x45,
    Semantic = 0x53,
    Procedural = 0x50,
}

impl KindTag {
    pub fn from_memory_kind(k: MemoryKind) -> Self {
        match k {
            MemoryKind::Working => KindTag::Working,
            MemoryKind::Episodic => KindTag::Episodic,
            MemoryKind::Semantic => KindTag::Semantic,
            MemoryKind::Procedural => KindTag::Procedural,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            KindTag::Working => "WorkingStore",
            KindTag::Episodic => "EpisodeStore",
            KindTag::Semantic => "VectorIndex",
            KindTag::Procedural => "WorkflowStore",
        }
    }
}

/// Kind-aware backing for a `MemoryStore` value. Each variant holds the
/// `garnet_memory` store implementation appropriate for its kind. Paper VI
/// Contribution 4 (kind-aware allocation) is realised here: declaring
/// `memory working scratch : Buffer` produces a `MemoryBackend::Working`,
/// `memory episodic log : EpisodeStore<Event>` produces `Episodic`, etc.
#[derive(Clone)]
pub enum MemoryBackend {
    Working(Rc<garnet_memory::WorkingStore<Value>>),
    Episodic(Rc<garnet_memory::EpisodeStore<Value>>),
    Semantic(Rc<garnet_memory::VectorIndex<Value>>),
    Procedural(Rc<garnet_memory::WorkflowStore<Value>>),
}

impl MemoryBackend {
    /// Construct the appropriate backend for the declared memory kind. Used
    /// by the interpreter's `register_item` for `Item::Memory`.
    pub fn for_kind(kind: MemoryKind) -> Self {
        match kind {
            MemoryKind::Working => {
                MemoryBackend::Working(Rc::new(garnet_memory::WorkingStore::new()))
            }
            MemoryKind::Episodic => {
                MemoryBackend::Episodic(Rc::new(garnet_memory::EpisodeStore::new()))
            }
            MemoryKind::Semantic => {
                MemoryBackend::Semantic(Rc::new(garnet_memory::VectorIndex::new()))
            }
            MemoryKind::Procedural => {
                MemoryBackend::Procedural(Rc::new(garnet_memory::WorkflowStore::new()))
            }
        }
    }

    /// Static name for the backend kind — used by tests to confirm dispatch
    /// without inspecting the `Rc<store>` interior.
    pub fn kind_name(&self) -> &'static str {
        self.kind_tag().name()
    }

    /// Read the backend's kind tag. Defense-in-depth primitive consumed
    /// by `ensure_kind` before dispatch.
    pub fn kind_tag(&self) -> KindTag {
        match self {
            MemoryBackend::Working(_) => KindTag::Working,
            MemoryBackend::Episodic(_) => KindTag::Episodic,
            MemoryBackend::Semantic(_) => KindTag::Semantic,
            MemoryBackend::Procedural(_) => KindTag::Procedural,
        }
    }

    /// Validate that this backend matches the expected kind before
    /// dispatching a method. Used by `dispatch_memory_method` as a
    /// redundant check against the outer enum match — so a mismatched
    /// `Value::MemoryStore { kind: _, backend: _ }` (constructed via
    /// direct struct-init that bypasses `for_kind`, or surviving a
    /// future IR lowering that drops the enum discriminant) is rejected
    /// with a clear error rather than silently invoking a wrong method.
    pub fn ensure_kind_matches(&self, declared: MemoryKind) -> Result<(), KindMismatch> {
        let actual = self.kind_tag();
        let expected = KindTag::from_memory_kind(declared);
        if actual == expected {
            Ok(())
        } else {
            Err(KindMismatch { actual, expected })
        }
    }
}

/// Structured error returned by `MemoryBackend::ensure_kind_matches`
/// when the declared memory kind disagrees with the backend's runtime
/// tag. Callers convert this into a `RuntimeError` with a useful
/// diagnostic.
#[derive(Debug, Clone, Copy)]
pub struct KindMismatch {
    pub actual: KindTag,
    pub expected: KindTag,
}

pub struct FnValue {
    pub def: FnDef,
    pub captured: Rc<Env>,
}

pub type NativeFn = fn(args: Vec<Value>) -> Result<Value, RuntimeError>;

pub struct NativeFnValue {
    pub name: &'static str,
    pub arity: Option<usize>, // None = variadic
    pub ptr: NativeFn,
}

pub enum TypeValue {
    Struct(StructDef),
    Enum(EnumDef),
}

impl Value {
    /// Ruby-style truthiness: only `nil` and `false` are falsy.
    pub fn truthy(&self) -> bool {
        !matches!(self, Value::Nil | Value::Bool(false))
    }

    /// Human-friendly rendering for `to_s` and `println`.
    pub fn display(&self) -> String {
        match self {
            Value::Nil => "nil".to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Int(i) => i.to_string(),
            Value::Float(f) => format!("{f}"),
            Value::Str(s) => (**s).clone(),
            Value::Symbol(s) => format!(":{}", s),
            Value::Array(a) => {
                let inner = a
                    .borrow()
                    .iter()
                    .map(|v| v.debug())
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("[{inner}]")
            }
            Value::Map(m) => {
                let inner = m
                    .borrow()
                    .iter()
                    .map(|(k, v)| format!("{k:?} => {}", v.debug()))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{{{inner}}}")
            }
            Value::Range {
                start,
                end,
                inclusive,
            } => {
                if *inclusive {
                    format!("{start}...{end}")
                } else {
                    format!("{start}..{end}")
                }
            }
            Value::Fn(f) => format!("<fn {}>", f.def.name),
            Value::NativeFn(n) => format!("<native fn {}>", n.name),
            Value::Type(t) => match t.as_ref() {
                TypeValue::Struct(s) => format!("<struct {}>", s.name),
                TypeValue::Enum(e) => format!("<enum {}>", e.name),
            },
            Value::Struct { name, fields } => {
                let inner = fields
                    .borrow()
                    .iter()
                    .map(|(k, v)| format!("{k}: {}", v.debug()))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{} {{ {} }}", name, inner)
            }
            Value::Variant {
                path,
                variant,
                fields,
            } => {
                let prefix = path.join("::");
                if fields.is_empty() {
                    format!("{prefix}::{variant}")
                } else {
                    let inner = fields
                        .iter()
                        .map(|v| v.debug())
                        .collect::<Vec<_>>()
                        .join(", ");
                    format!("{prefix}::{variant}({inner})")
                }
            }
            Value::MemoryStore { kind, name, .. } => {
                format!("<memory {} {}>", format!("{kind:?}").to_lowercase(), name)
            }
            Value::Tuple(items) => {
                let inner = items
                    .iter()
                    .map(|v| v.debug())
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("({inner})")
            }
        }
    }

    /// Debug rendering — like `display` but quotes strings.
    pub fn debug(&self) -> String {
        match self {
            Value::Str(s) => format!("{:?}", s.as_str()),
            _ => self.display(),
        }
    }

    /// Type name (used by the prelude's `type_of` and error messages).
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Nil => "Nil",
            Value::Bool(_) => "Bool",
            Value::Int(_) => "Int",
            Value::Float(_) => "Float",
            Value::Str(_) => "String",
            Value::Symbol(_) => "Symbol",
            Value::Array(_) => "Array",
            Value::Map(_) => "Map",
            Value::Range { .. } => "Range",
            Value::Fn(_) | Value::NativeFn(_) => "Fn",
            Value::Type(_) => "Type",
            Value::Struct { .. } => "Struct",
            Value::Variant { .. } => "Variant",
            Value::MemoryStore { .. } => "MemoryStore",
            Value::Tuple(_) => "Tuple",
        }
    }

    /// Deep equality (value semantics, not reference identity).
    pub fn eq_deep(&self, other: &Value) -> bool {
        use Value::*;
        match (self, other) {
            (Nil, Nil) => true,
            (Bool(a), Bool(b)) => a == b,
            (Int(a), Int(b)) => a == b,
            (Float(a), Float(b)) => a == b,
            (Int(a), Float(b)) | (Float(b), Int(a)) => (*a as f64) == *b,
            (Str(a), Str(b)) => **a == **b,
            (Symbol(a), Symbol(b)) => **a == **b,
            (Array(a), Array(b)) => {
                let a = a.borrow();
                let b = b.borrow();
                a.len() == b.len() && a.iter().zip(b.iter()).all(|(x, y)| x.eq_deep(y))
            }
            (Map(a), Map(b)) => {
                let a = a.borrow();
                let b = b.borrow();
                a.len() == b.len()
                    && a.iter()
                        .all(|(k, v)| b.get(k).is_some_and(|w| v.eq_deep(w)))
            }
            (
                Range {
                    start: s1,
                    end: e1,
                    inclusive: i1,
                },
                Range {
                    start: s2,
                    end: e2,
                    inclusive: i2,
                },
            ) => s1 == s2 && e1 == e2 && i1 == i2,
            (Tuple(a), Tuple(b)) => {
                a.len() == b.len() && a.iter().zip(b.iter()).all(|(x, y)| x.eq_deep(y))
            }
            (
                Variant {
                    path: p1,
                    variant: v1,
                    fields: f1,
                },
                Variant {
                    path: p2,
                    variant: v2,
                    fields: f2,
                },
            ) => {
                p1 == p2 && v1 == v2 && {
                    f1.len() == f2.len() && f1.iter().zip(f2.iter()).all(|(x, y)| x.eq_deep(y))
                }
            }
            _ => false,
        }
    }

    /// Ordering for comparison operators. Only compatible numeric and string
    /// values yield an ordering; others raise a runtime error at the call site.
    /// Renamed from `cmp` to avoid shadowing `std::cmp::Ord::cmp`.
    pub fn partial_compare(&self, other: &Value) -> Option<std::cmp::Ordering> {
        use std::cmp::Ordering;
        use Value::*;
        match (self, other) {
            (Int(a), Int(b)) => Some(a.cmp(b)),
            (Float(a), Float(b)) => a.partial_cmp(b),
            (Int(a), Float(b)) => (*a as f64).partial_cmp(b),
            (Float(a), Int(b)) => a.partial_cmp(&(*b as f64)),
            (Str(a), Str(b)) => Some(a.cmp(b)),
            (Bool(a), Bool(b)) => Some(a.cmp(b)),
            (Nil, Nil) => Some(Ordering::Equal),
            _ => None,
        }
    }
}

// ── Constructors used throughout the interpreter ──

impl Value {
    pub fn str(s: impl Into<String>) -> Value {
        Value::Str(Rc::new(s.into()))
    }
    pub fn sym(s: impl Into<String>) -> Value {
        Value::Symbol(Rc::new(s.into()))
    }
    pub fn array(items: Vec<Value>) -> Value {
        Value::Array(Rc::new(RefCell::new(items)))
    }
    pub fn map(entries: Vec<(String, Value)>) -> Value {
        let mut m = BTreeMap::new();
        for (k, v) in entries {
            m.insert(k, v);
        }
        Value::Map(Rc::new(RefCell::new(m)))
    }
    pub fn tuple(items: Vec<Value>) -> Value {
        Value::Tuple(Rc::new(items))
    }
}

impl std::fmt::Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.debug())
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display())
    }
}

/// Helper to read a parameter list into named slots for function call binding.
pub fn bind_params(params: &[Param], args: Vec<Value>, env: &Env) -> Result<(), RuntimeError> {
    if params.len() != args.len() {
        return Err(RuntimeError::Message(format!(
            "arity mismatch: expected {}, got {}",
            params.len(),
            args.len()
        )));
    }
    for (p, a) in params.iter().zip(args) {
        env.define(&p.name, a);
    }
    Ok(())
}
