//! Expression evaluator — implements the 11-level Pratt precedence tower at runtime.

use crate::control::{eval_if, eval_match, eval_try};
use crate::env::Env;
use crate::error::RuntimeError;
use crate::stmt;
use crate::value::{
    bind_params, ensure_protocol_satisfied, has_dynamic_annotation, protocol_for_type, ActorHandle,
    ActorMessage, FnValue, MemoryBackend, TypeValue, Value,
};
use garnet_parser::ast::{
    ActorDef, ActorItem, Annotation, BinOp, Capability, ClosureBody, Expr, FnMode, StringLit,
    TypeExpr, UnOp,
};
use garnet_parser::token::StrPart;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

const DEFAULT_MANAGED_ACTOR_MAILBOX_CAPACITY: usize = 1024;
const MAX_MANAGED_ACTOR_MAILBOX_CAPACITY: usize = 1_048_576;

/// Evaluate an expression in the given environment.
pub fn eval_expr(expr: &Expr, env: &Rc<Env>) -> Result<Value, RuntimeError> {
    match expr {
        // ── Literals ──
        Expr::Int(v, _) => Ok(Value::Int(*v)),
        Expr::Float(v, _) => Ok(Value::Float(*v)),
        Expr::Bool(b, _) => Ok(Value::Bool(*b)),
        Expr::Nil(_) => Ok(Value::Nil),
        Expr::Symbol(s, _) => Ok(Value::sym(s.clone())),
        Expr::Str(lit, _) => eval_string(lit, env),

        // ── Names ──
        Expr::Ident(name, _) => env
            .get(name)
            .ok_or_else(|| RuntimeError::Message(format!("undefined variable: {name}"))),
        Expr::Path(segs, _) => eval_path(segs, env),

        // ── Operators ──
        Expr::Binary { op, lhs, rhs, .. } => eval_binary(*op, lhs, rhs, env),
        Expr::Unary { op, expr, .. } => eval_unary(*op, expr, env),

        // ── Calls & access ──
        Expr::Call { callee, args, .. } => {
            let callee_val = eval_expr(callee, env)?;
            let arg_vals: Result<Vec<_>, _> = args.iter().map(|a| eval_expr(a, env)).collect();
            call_value(&callee_val, arg_vals?)
        }
        Expr::Method {
            receiver,
            method,
            args,
            ..
        } => {
            let recv = eval_expr(receiver, env)?;
            let arg_vals: Result<Vec<_>, _> = args.iter().map(|a| eval_expr(a, env)).collect();
            call_method(&recv, method, arg_vals?, env)
        }
        Expr::Field {
            receiver, field, ..
        } => {
            let recv = eval_expr(receiver, env)?;
            access_field(&recv, field)
        }
        Expr::Index {
            receiver, index, ..
        } => {
            let recv = eval_expr(receiver, env)?;
            let idx = eval_expr(index, env)?;
            access_index(&recv, &idx)
        }
        Expr::Cast { expr, ty, .. } => {
            let value = eval_expr(expr, env)?;
            let protocol = protocol_for_type(ty, env).ok_or_else(|| {
                RuntimeError::msg(format!("cast target {} is not a protocol", type_name(ty)))
            })?;
            ensure_protocol_satisfied(&value, &protocol, env)?;
            Ok(value)
        }

        // ── Control-flow expressions ──
        Expr::If {
            condition,
            then_block,
            elsif_clauses,
            else_block,
            ..
        } => eval_if(
            condition,
            then_block,
            elsif_clauses,
            else_block.as_ref(),
            env,
        ),
        Expr::Match { subject, arms, .. } => eval_match(subject, arms, env),
        Expr::Try {
            body,
            rescues,
            ensure,
            ..
        } => eval_try(body, rescues, ensure.as_ref(), env),

        // ── First-class values ──
        Expr::Closure {
            params,
            body,
            is_do_block,
            ..
        } => {
            // Build a synthetic FnDef so `call_value` can reuse the same code path.
            let fn_def = garnet_parser::ast::FnDef {
                annotations: vec![],
                public: false,
                mode: garnet_parser::ast::FnMode::Managed,
                name: "<closure>".to_string(),
                type_params: vec![],
                params: params.clone(),
                return_ty: None,
                body: match body.as_ref() {
                    ClosureBody::Block(b) => b.clone(),
                    ClosureBody::Expr(e) => garnet_parser::ast::Block {
                        stmts: vec![],
                        tail_expr: Some(Box::new(e.clone())),
                        span: e.span(),
                    },
                },
                span: expr.span(),
            };
            Ok(Value::Fn(Rc::new(FnValue {
                def: fn_def,
                captured: Rc::clone(env),
                is_block: *is_do_block,
            })))
        }
        Expr::Spawn { expr, .. } => {
            // Rung-3 interpreter runs `spawn` synchronously. The actor
            // runtime (Rung 6) supplies the real parallelism later.
            match eval_expr(expr, env)? {
                Value::ActorType(actor) => spawn_actor_address(actor, env, None),
                other => Ok(other),
            }
        }
        Expr::Array { elements, .. } => {
            let items: Result<Vec<_>, _> = elements.iter().map(|e| eval_expr(e, env)).collect();
            Ok(Value::array(items?))
        }
        Expr::Map { entries, .. } => {
            let mut m = BTreeMap::new();
            for (k, v) in entries {
                let kv = eval_expr(k, env)?;
                let vv = eval_expr(v, env)?;
                let key_str = match &kv {
                    Value::Str(s) => s.to_string(),
                    Value::Symbol(s) => format!(":{s}"),
                    other => other.display(),
                };
                m.insert(key_str, vv);
            }
            Ok(Value::Map(Rc::new(RefCell::new(m))))
        }
    }
}

fn type_name(ty: &TypeExpr) -> String {
    match ty {
        TypeExpr::Named { path, .. } => path.join("::"),
        TypeExpr::Dyn { trait_ty, .. } => format!("dyn {}", type_name(trait_ty)),
        TypeExpr::Fn { .. } => "fn type".to_string(),
        TypeExpr::Tuple { .. } => "tuple type".to_string(),
        TypeExpr::Ref { inner, .. } => format!("ref {}", type_name(inner)),
    }
}

fn eval_string(lit: &StringLit, env: &Rc<Env>) -> Result<Value, RuntimeError> {
    let mut out = String::new();
    for part in &lit.parts {
        match part {
            StrPart::Lit(s) => out.push_str(s),
            StrPart::Interp(src) => {
                // Re-parse & evaluate the interpolation expression body.
                let wrapped = format!("def __interp__() {{ {src} }}");
                let module = garnet_parser::parse_source(&wrapped)
                    .map_err(|e| RuntimeError::Parse(format!("{e:?}")))?;
                let mut interp_result = Value::Nil;
                for item in module.items {
                    if let garnet_parser::ast::Item::Fn(fn_def) = item {
                        if let Some(tail) = fn_def.body.tail_expr {
                            interp_result = eval_expr(&tail, env)?;
                        }
                    }
                }
                out.push_str(&interp_result.display());
            }
        }
    }
    Ok(Value::str(out))
}

fn eval_path(segs: &[String], env: &Env) -> Result<Value, RuntimeError> {
    if segs.len() == 1 {
        return env
            .get(&segs[0])
            .ok_or_else(|| RuntimeError::Message(format!("undefined name: {}", segs[0])));
    }
    // Two common shapes we handle:
    //   1. EnumType::Variant            → a nullary variant (no payload)
    //   2. EnumType::Variant(args...)   → handled by the Call wrapping us
    //   3. Module::name                 → look up on the root for now (Rung 6 formalizes)
    if segs.len() == 2 {
        if let Some(Value::Type(t)) = env.get(&segs[0]) {
            if let TypeValue::Enum(e) = t.as_ref() {
                let variant = &segs[1];
                if e.variants.iter().any(|v| v.name == *variant) {
                    return Ok(Value::Variant {
                        path: Rc::new(vec![e.name.clone()]),
                        variant: Rc::new(variant.clone()),
                        fields: Rc::new(Vec::new()),
                    });
                }
            }
        }
    }
    // Resolve the FULLY-QUALIFIED name first (e.g. `core::math::sqrt`,
    // `std::base64::encode`). Stdlib primitives bound under their full path
    // (S21 qualified dispatch) resolve here without colliding with bare
    // prelude builtins that share a last segment (`map` = Map ctor, `ok`/`err`
    // = Result builders, ...). This is additive: names that aren't bound under
    // their qualified form fall through to the existing last-segment behavior,
    // so `Storage::read_block` → top-level `read_block` still works.
    let qualified = segs.join("::");
    if let Some(v) = env.get(&qualified) {
        return Ok(v);
    }
    // Fallback: treat the last segment as a top-level name. This lets simple
    // module-qualified calls like `Storage::read_block` resolve against global
    // names while the full module system is still stubbed.
    let last = segs
        .last()
        .ok_or_else(|| RuntimeError::Message("empty path expression".into()))?;
    env.get(last)
        .ok_or_else(|| RuntimeError::Message(format!("unresolved path: {}", qualified)))
}

fn eval_binary(op: BinOp, lhs: &Expr, rhs: &Expr, env: &Rc<Env>) -> Result<Value, RuntimeError> {
    // Short-circuit for logical ops.
    if op == BinOp::And {
        let l = eval_expr(lhs, env)?;
        return if !l.truthy() {
            Ok(l)
        } else {
            eval_expr(rhs, env)
        };
    }
    if op == BinOp::Or {
        let l = eval_expr(lhs, env)?;
        return if l.truthy() {
            Ok(l)
        } else {
            eval_expr(rhs, env)
        };
    }
    // Pipeline: desugar `x |> f` into a call `f(x)`. If the RHS is already a
    // call, prepend x as the first argument.
    if op == BinOp::Pipeline {
        let l = eval_expr(lhs, env)?;
        return apply_pipeline(l, rhs, env);
    }

    let l = eval_expr(lhs, env)?;
    let r = eval_expr(rhs, env)?;

    use BinOp::*;
    use Value::*;
    match op {
        Add => match (&l, &r) {
            (Int(a), Int(b)) => Ok(Int(a + b)),
            (Float(a), Float(b)) => Ok(Float(a + b)),
            (Int(a), Float(b)) => Ok(Float(*a as f64 + b)),
            (Float(a), Int(b)) => Ok(Float(a + *b as f64)),
            (Str(a), Str(b)) => Ok(Value::str(format!("{a}{b}"))),
            _ => Err(RuntimeError::type_err("numeric or string pair", &l)),
        },
        Sub => match (&l, &r) {
            (Int(a), Int(b)) => Ok(Int(a - b)),
            (Float(a), Float(b)) => Ok(Float(a - b)),
            (Int(a), Float(b)) => Ok(Float(*a as f64 - b)),
            (Float(a), Int(b)) => Ok(Float(a - *b as f64)),
            _ => Err(RuntimeError::type_err("numeric pair", &l)),
        },
        Mul => match (&l, &r) {
            (Int(a), Int(b)) => Ok(Int(a * b)),
            (Float(a), Float(b)) => Ok(Float(a * b)),
            (Int(a), Float(b)) => Ok(Float(*a as f64 * b)),
            (Float(a), Int(b)) => Ok(Float(a * *b as f64)),
            _ => Err(RuntimeError::type_err("numeric pair", &l)),
        },
        Div => {
            // Detect division by zero uniformly across all numeric type pairs
            // before dispatching to the specific arithmetic branch.
            let div_by_zero = matches!(&r, Int(0)) || matches!(&r, Float(f) if *f == 0.0);
            if div_by_zero {
                return Err(RuntimeError::DivByZero);
            }
            match (&l, &r) {
                // checked_div: zero is pre-guarded above, so None here is
                // exactly the i64::MIN / -1 overflow — a diagnostic, not an
                // abort (RB-2).
                (Int(a), Int(b)) => a
                    .checked_div(*b)
                    .map(Int)
                    .ok_or_else(|| RuntimeError::Overflow(format!("{a} / {b}"))),
                (Float(a), Float(b)) => Ok(Float(a / b)),
                (Int(a), Float(b)) => Ok(Float(*a as f64 / b)),
                (Float(a), Int(b)) => Ok(Float(a / *b as f64)),
                _ => Err(RuntimeError::type_err("numeric pair", &l)),
            }
        }
        Mod => match (&l, &r) {
            (Int(a), Int(b)) => {
                if *b == 0 {
                    Err(RuntimeError::DivByZero)
                } else {
                    // None = i64::MIN % -1 overflow — diagnostic, not abort (RB-2).
                    a.checked_rem(*b)
                        .map(Int)
                        .ok_or_else(|| RuntimeError::Overflow(format!("{a} % {b}")))
                }
            }
            (Float(a), Float(b)) => Ok(Float(a % b)),
            _ => Err(RuntimeError::type_err("integer or float pair", &l)),
        },
        Eq => Ok(Bool(l.eq_deep(&r))),
        NotEq => Ok(Bool(!l.eq_deep(&r))),
        Lt | Gt | LtEq | GtEq => {
            let cmp = l
                .partial_compare(&r)
                .ok_or_else(|| RuntimeError::type_err("comparable pair", &l))?;
            Ok(Bool(match op {
                Lt => cmp.is_lt(),
                Gt => cmp.is_gt(),
                LtEq => cmp.is_le(),
                GtEq => cmp.is_ge(),
                _ => unreachable!(),
            }))
        }
        Range => match (&l, &r) {
            (Int(a), Int(b)) => Ok(Value::Range {
                start: *a,
                end: *b,
                inclusive: false,
            }),
            _ => Err(RuntimeError::type_err("integer pair", &l)),
        },
        RangeInclusive => match (&l, &r) {
            (Int(a), Int(b)) => Ok(Value::Range {
                start: *a,
                end: *b,
                inclusive: true,
            }),
            _ => Err(RuntimeError::type_err("integer pair", &l)),
        },
        And | Or | Pipeline => unreachable!(),
    }
}

fn apply_pipeline(arg: Value, rhs: &Expr, env: &Rc<Env>) -> Result<Value, RuntimeError> {
    match rhs {
        Expr::Call { callee, args, .. } => {
            let callee_val = eval_expr(callee, env)?;
            let mut arg_vals = vec![arg];
            for a in args {
                arg_vals.push(eval_expr(a, env)?);
            }
            call_value(&callee_val, arg_vals)
        }
        Expr::Method {
            receiver,
            method,
            args,
            ..
        } => {
            // `x |> y.method(args)` → `y.method(x, args...)`
            let recv = eval_expr(receiver, env)?;
            let mut arg_vals = vec![arg];
            for a in args {
                arg_vals.push(eval_expr(a, env)?);
            }
            call_method(&recv, method, arg_vals, env)
        }
        _ => {
            // Plain callable: `x |> f` → `f(x)`
            let callee_val = eval_expr(rhs, env)?;
            call_value(&callee_val, vec![arg])
        }
    }
}

fn eval_unary(op: UnOp, inner: &Expr, env: &Rc<Env>) -> Result<Value, RuntimeError> {
    let v = eval_expr(inner, env)?;
    match op {
        UnOp::Neg => match v {
            Value::Int(i) => Ok(Value::Int(-i)),
            Value::Float(f) => Ok(Value::Float(-f)),
            other => Err(RuntimeError::type_err("numeric", &other)),
        },
        UnOp::Not => Ok(Value::Bool(!v.truthy())),
        UnOp::Question => {
            // Result / Option propagation: unwrap Ok(x) to x; bail out of the
            // caller with an Err(e) by raising it. Our interp models Ok/Err
            // as `Variant { path: [Result|Option], variant, fields }`.
            match &v {
                Value::Variant {
                    variant, fields, ..
                } if variant.as_str() == "Ok" => Ok(fields.first().cloned().unwrap_or(Value::Nil)),
                Value::Variant {
                    variant, fields, ..
                } if variant.as_str() == "Some" => {
                    Ok(fields.first().cloned().unwrap_or(Value::Nil))
                }
                Value::Variant { variant, .. }
                    if variant.as_str() == "Err" || variant.as_str() == "None" =>
                {
                    Err(RuntimeError::Raised(v.clone()))
                }
                _ => Err(RuntimeError::type_err("Result or Option variant", &v)),
            }
        }
    }
}

/// Call a function value with argument values.
pub fn call_value(callee: &Value, args: Vec<Value>) -> Result<Value, RuntimeError> {
    match callee {
        Value::Fn(f) => call_fn(f, args),
        Value::NativeFn(n) => {
            if let Some(expected) = n.arity {
                if args.len() != expected {
                    return Err(RuntimeError::Message(format!(
                        "{}: arity mismatch (expected {}, got {})",
                        n.name,
                        expected,
                        args.len()
                    )));
                }
            }
            (n.ptr)(args)
        }
        Value::Type(t) => {
            // Calling a type works as a constructor.
            match t.as_ref() {
                TypeValue::Struct(s) => {
                    if s.fields.len() != args.len() {
                        return Err(RuntimeError::Message(format!(
                            "{}: expected {} fields, got {}",
                            s.name,
                            s.fields.len(),
                            args.len()
                        )));
                    }
                    let mut m = BTreeMap::new();
                    for (f, v) in s.fields.iter().zip(args) {
                        m.insert(f.name.clone(), v);
                    }
                    Ok(Value::Struct {
                        name: Rc::new(s.name.clone()),
                        fields: Rc::new(RefCell::new(m)),
                        dynamic_methods: has_dynamic_annotation(&s.annotations)
                            .then(|| Rc::new(RefCell::new(BTreeMap::new()))),
                    })
                }
                TypeValue::Enum(_) => Err(RuntimeError::Message(
                    "call an enum variant, not the enum itself".to_string(),
                )),
            }
        }
        Value::Variant { path, variant, .. } => {
            // Called path-style: `Result::Ok(1)` parses as a Call whose callee
            // is the path value; the path resolution above returned a
            // zero-arg variant, and now we fill in the args.
            Ok(Value::Variant {
                path: Rc::clone(path),
                variant: Rc::clone(variant),
                fields: Rc::new(args),
            })
        }
        other => Err(RuntimeError::type_err("callable", other)),
    }
}

thread_local! {
    /// Per-function recursion depth for `@max_depth(N)` runtime enforcement
    /// (S89). Only functions that declare `@max_depth` are tracked — every other
    /// call is untouched. Keyed by function name (recursion is self-by-name); the
    /// counter unwinds via `MaxDepthGuard` on every return/error path, and lives
    /// on the per-run `garnet-interp` thread (S85), so it starts clean each run.
    static MAX_DEPTH_DEPTHS: RefCell<BTreeMap<String, u64>> = const { RefCell::new(BTreeMap::new()) };
}

/// The `@max_depth(N)` ceiling declared on a function, if any.
fn max_depth_ceiling(annotations: &[Annotation]) -> Option<i64> {
    annotations.iter().find_map(|a| match a {
        Annotation::MaxDepth(n, _) => Some(*n),
        _ => None,
    })
}

/// RAII guard for a `@max_depth` function's recursion counter: increments on
/// `enter`, decrements on drop, so the count is correct even when a call returns
/// an error (including the trap itself, whose guard drops as it unwinds).
struct MaxDepthGuard {
    name: String,
    depth: u64,
}

impl MaxDepthGuard {
    fn enter(name: String) -> Self {
        let depth = MAX_DEPTH_DEPTHS.with(|m| {
            let mut m = m.borrow_mut();
            let c = m.entry(name.clone()).or_insert(0);
            *c += 1;
            *c
        });
        MaxDepthGuard { name, depth }
    }
}

impl Drop for MaxDepthGuard {
    fn drop(&mut self) {
        MAX_DEPTH_DEPTHS.with(|m| {
            if let Some(c) = m.borrow_mut().get_mut(&self.name) {
                *c = c.saturating_sub(1);
            }
        });
    }
}

// ── @caps host-authority runtime enforcement (S90) ────────────────────────────

/// The capability's `@caps(...)` identifier (matches `Capability::from_ident`).
fn cap_ident(c: &Capability) -> String {
    match c {
        Capability::Fs => "fs".to_string(),
        Capability::Net => "net".to_string(),
        Capability::NetInternal => "net_internal".to_string(),
        Capability::Time => "time".to_string(),
        Capability::Proc => "proc".to_string(),
        Capability::Ffi => "ffi".to_string(),
        Capability::Wildcard => "*".to_string(),
        Capability::Other(s) => s.clone(),
    }
}

/// The capabilities declared by a managed function's `@caps(...)` annotation(s).
fn declared_caps(annotations: &[Annotation]) -> Vec<String> {
    annotations
        .iter()
        .filter_map(|a| match a {
            Annotation::Caps(caps, _) => Some(caps.iter().map(cap_ident).collect::<Vec<_>>()),
            _ => None,
        })
        .flatten()
        .collect()
}

/// The active capability context on the per-run `garnet-interp` thread.
/// `active_frames` counts program-entry frames plus managed functions currently
/// on the stack, and `counts` is the multiset union of every `@caps` they
/// declared. A host-authority primitive is permitted iff some active program
/// frame in the call chain declared the required capability (the static
/// caps-graph propagates caps up every frame, so a *checked* program always
/// carries it — only under-declared programs trap).
struct CapsContext {
    active_frames: u32,
    counts: BTreeMap<String, u32>,
    entry_frames: u32,
    entry_counts: BTreeMap<String, u32>,
}

thread_local! {
    static ACTIVE_CAPS: RefCell<CapsContext> =
        const {
            RefCell::new(CapsContext {
                active_frames: 0,
                counts: BTreeMap::new(),
                entry_frames: 0,
                entry_counts: BTreeMap::new(),
            })
        };
}

/// RAII guard: records a program/managed frame + declared caps; unwinds both on
/// drop (every return/error path, including a trap).
struct CapsGuard {
    idents: Vec<String>,
    entry: bool,
}

impl CapsGuard {
    fn enter(annotations: &[Annotation]) -> Self {
        Self::enter_caps(declared_caps(annotations), false)
    }

    fn enter_entry(annotations: &[Annotation]) -> Self {
        Self::enter_caps(declared_caps(annotations), true)
    }

    fn enter_caps(idents: Vec<String>, entry: bool) -> Self {
        ACTIVE_CAPS.with(|c| {
            let mut c = c.borrow_mut();
            c.active_frames += 1;
            if entry {
                c.entry_frames += 1;
            }
            for id in &idents {
                *c.counts.entry(id.clone()).or_insert(0) += 1;
                if entry {
                    *c.entry_counts.entry(id.clone()).or_insert(0) += 1;
                }
            }
        });
        CapsGuard { idents, entry }
    }
}

impl Drop for CapsGuard {
    fn drop(&mut self) {
        ACTIVE_CAPS.with(|c| {
            let mut c = c.borrow_mut();
            c.active_frames = c.active_frames.saturating_sub(1);
            if self.entry {
                c.entry_frames = c.entry_frames.saturating_sub(1);
            }
            for id in &self.idents {
                if let Some(n) = c.counts.get_mut(id) {
                    *n = n.saturating_sub(1);
                }
                if self.entry {
                    if let Some(n) = c.entry_counts.get_mut(id) {
                        *n = n.saturating_sub(1);
                    }
                }
            }
        });
    }
}

/// Enforce that a host-authority primitive's required capability was declared in
/// the calling chain. S91 adds a program-entry frame for `garnet run --interp`
/// so safe-mode entry points are covered too. Outside any program frame (e.g. a
/// direct host/test call) there is no `@caps` context to enforce, so the call is
/// allowed.
pub(crate) fn require_capability(needed: &str, fn_name: &str) -> Result<(), RuntimeError> {
    ACTIVE_CAPS.with(|c| {
        let c = c.borrow();
        if c.active_frames == 0 {
            return Ok(());
        }
        let has = |k: &str| c.counts.get(k).copied().unwrap_or(0) > 0;
        if has("*") || has(needed) {
            Ok(())
        } else {
            Err(RuntimeError::msg(format!(
                "capability: `{fn_name}` requires @caps({needed}), not declared in the calling chain"
            )))
        }
    })
}

/// Enforce that a host-authority primitive's required capability was declared by
/// the program entry point itself. S92 uses this for subprocess launch surfaces
/// so `main @caps()` cannot launder process authority through a helper that
/// declares `@caps(proc)`. Direct host/test calls outside a program entry frame
/// remain allowed because there is no source-level entry declaration to inspect.
pub(crate) fn require_entry_capability(needed: &str, fn_name: &str) -> Result<(), RuntimeError> {
    ACTIVE_CAPS.with(|c| {
        let c = c.borrow();
        if c.entry_frames == 0 {
            return Ok(());
        }
        let has = |k: &str| c.entry_counts.get(k).copied().unwrap_or(0) > 0;
        if has("*") || has(needed) {
            Ok(())
        } else {
            Err(RuntimeError::msg(format!(
                "capability: `{fn_name}` requires program entry @caps({needed}), not declared by the entry point"
            )))
        }
    })
}

/// Call a value as the program entry point, installing an `@caps` frame from the
/// entry function before dispatch. This closes the safe/direct-entry gap where a
/// safe-mode `fn main` could reach host authority with no managed frame active.
pub(crate) fn call_value_with_entry_caps(
    callee: &Value,
    args: Vec<Value>,
) -> Result<Value, RuntimeError> {
    let _entry_caps_guard = match callee {
        Value::Fn(f) => Some(CapsGuard::enter_entry(&f.def.annotations)),
        _ => None,
    };
    call_value(callee, args)
}

/// A program-entry `@caps` scope owned by an out-of-interpreter caller (the
/// bytecode VM, S100). It holds the entry `CapsGuard`, so dropping it unwinds the
/// entry frame. The VM holds one across an entire run, making its fallback path
/// enforce the S92 program-entry capability gate identically to `--interp`'s
/// `call_entry` — closing the VM `@caps`-laundering seam.
pub struct EntryCapsScope(#[allow(dead_code)] CapsGuard);

/// Install a program-entry `@caps` frame for `callee` if it is a function value,
/// returning the scope to hold for the run (S100). Mirrors the entry-frame install
/// in `call_value_with_entry_caps` but returns the live guard instead of running
/// the body, so the VM can hold it across native + fallback dispatch.
pub fn enter_entry_caps_for(callee: &Value) -> Option<EntryCapsScope> {
    match callee {
        Value::Fn(f) => Some(EntryCapsScope(CapsGuard::enter_entry(&f.def.annotations))),
        _ => None,
    }
}

fn call_fn(f: &FnValue, mut args: Vec<Value>) -> Result<Value, RuntimeError> {
    // `@caps` host-authority enforcement (S90): a managed function pushes its
    // declared `@caps` onto the active-capability context for the duration of its
    // body, so a host-authority primitive (`std::env`/`std::process`/`fs::`/
    // `std::log::to_file`) traps if no frame in the call chain declared its cap.
    let _caps_guard = if matches!(f.def.mode, FnMode::Managed) {
        Some(CapsGuard::enter(&f.def.annotations))
    } else {
        None
    };
    // `@max_depth(N)` runtime enforcement (S89): a function that declares
    // `@max_depth` traps deterministically when its recursion depth exceeds N —
    // real enforcement (the interpreter refuses to recurse further), distinct
    // from the S85 host-stack raise. Honest scope: this is the ONE enforced
    // ceiling; `@bounded` (Wasmtime fuel), memory, time, and mailbox remain
    // declared-not-enforced. The guard lives for the body's execution and
    // unwinds the counter on drop.
    let _depth_guard = match max_depth_ceiling(&f.def.annotations) {
        Some(n) => {
            let guard = MaxDepthGuard::enter(f.def.name.clone());
            if guard.depth > n.max(0) as u64 {
                return Err(RuntimeError::msg(format!(
                    "bounded: @max_depth({n}) exceeded for `{}` (recursion depth {})",
                    f.def.name, guard.depth
                )));
            }
            Some(guard)
        }
        None => None,
    };
    let active_block = if args.len() == f.def.params.len() + 1 {
        // Pop-then-match keeps this abort-free (RB-2): the popped value IS
        // the element `last()` just matched, so the non-block arms simply
        // fall through to "no trailing block".
        match args.last() {
            Some(Value::Fn(block)) if block.is_block => match args.pop() {
                Some(Value::Fn(block)) => Some(block),
                _ => None,
            },
            _ => None,
        }
    } else {
        None
    };
    let call_env = Env::new_child(&f.captured);
    bind_params(&f.def.params, args, &call_env)?;
    if let Some(block) = active_block {
        call_env.set_active_block(Value::Fn(block));
    }
    let is_block_closure = f.is_block;
    // Execute body.
    for s in &f.def.body.stmts {
        match stmt::exec_stmt(s, &call_env) {
            Ok(()) => {}
            Err(RuntimeError::Return(v)) => return Ok(v),
            Err(RuntimeError::Next(v)) if is_block_closure => return Ok(v),
            Err(RuntimeError::Next(_)) => {
                return Err(RuntimeError::msg("`next` used outside block"))
            }
            Err(e) => return Err(e),
        }
    }
    if let Some(tail) = &f.def.body.tail_expr {
        match eval_expr(tail, &call_env) {
            Ok(v) => Ok(v),
            Err(RuntimeError::Return(v)) => Ok(v),
            Err(RuntimeError::Next(v)) if is_block_closure => Ok(v),
            Err(RuntimeError::Next(_)) => Err(RuntimeError::msg("`next` used outside block")),
            Err(e) => Err(e),
        }
    } else {
        Ok(Value::Nil)
    }
}

fn call_method(
    recv: &Value,
    method: &str,
    args: Vec<Value>,
    env: &Rc<Env>,
) -> Result<Value, RuntimeError> {
    // Intrinsic methods on built-in values. Extending this to user-defined
    // struct methods requires linking impl blocks to structs, which is a
    // Rung 4+ concern. For now, common built-ins cover ~80% of programs.
    match recv {
        Value::Str(s) => match method {
            "len" | "length" | "size" => Ok(Value::Int(s.chars().count() as i64)),
            "upcase" | "to_upper" => Ok(Value::str(s.to_uppercase())),
            "downcase" | "to_lower" => Ok(Value::str(s.to_lowercase())),
            "to_s" => Ok(recv.clone()),
            "chars" => {
                let items: Vec<Value> = s.chars().map(|c| Value::str(c.to_string())).collect();
                Ok(Value::array(items))
            }
            "starts_with" | "starts_with?" => {
                let arg = args
                    .first()
                    .ok_or_else(|| RuntimeError::msg("starts_with: missing arg"))?;
                match arg {
                    Value::Str(t) => Ok(Value::Bool(s.starts_with(t.as_str()))),
                    _ => Err(RuntimeError::type_err("String", arg)),
                }
            }
            _ => Err(RuntimeError::msg(format!(
                "String has no method '{method}'"
            ))),
        },
        Value::Array(arr) => match method {
            "len" | "length" | "size" | "count" => Ok(Value::Int(arr.borrow().len() as i64)),
            "push" | "append" => {
                for a in args {
                    arr.borrow_mut().push(a);
                }
                Ok(recv.clone())
            }
            "first" => Ok(arr.borrow().first().cloned().unwrap_or(Value::Nil)),
            "last" => Ok(arr.borrow().last().cloned().unwrap_or(Value::Nil)),
            "map" => {
                let f = args
                    .first()
                    .ok_or_else(|| RuntimeError::msg("map: missing closure"))?
                    .clone();
                let mut out = Vec::new();
                for item in arr.borrow().iter() {
                    out.push(call_value(&f, vec![item.clone()])?);
                }
                Ok(Value::array(out))
            }
            "filter" | "select" => {
                let f = args
                    .first()
                    .ok_or_else(|| RuntimeError::msg("filter: missing closure"))?
                    .clone();
                let mut out = Vec::new();
                for item in arr.borrow().iter() {
                    if call_value(&f, vec![item.clone()])?.truthy() {
                        out.push(item.clone());
                    }
                }
                Ok(Value::array(out))
            }
            "reduce" => {
                let init = args
                    .first()
                    .ok_or_else(|| RuntimeError::msg("reduce: missing initial"))?
                    .clone();
                let f = args
                    .get(1)
                    .ok_or_else(|| RuntimeError::msg("reduce: missing closure"))?
                    .clone();
                let mut acc = init;
                for item in arr.borrow().iter() {
                    acc = call_value(&f, vec![acc, item.clone()])?;
                }
                Ok(acc)
            }
            "recent" => {
                let n = args
                    .first()
                    .and_then(|v| match v {
                        Value::Int(i) => Some(*i as usize),
                        _ => None,
                    })
                    .unwrap_or(0);
                let borrowed = arr.borrow();
                let len = borrowed.len();
                let start = len.saturating_sub(n);
                Ok(Value::array(borrowed[start..].to_vec()))
            }
            "to_s" => Ok(Value::str(recv.display())),
            _ => Err(RuntimeError::msg(format!("Array has no method '{method}'"))),
        },
        Value::Map(m) => match method {
            "len" | "size" => Ok(Value::Int(m.borrow().len() as i64)),
            "get" => {
                let key = args
                    .first()
                    .ok_or_else(|| RuntimeError::msg("get: missing key"))?;
                let key_str = match key {
                    Value::Str(s) => s.to_string(),
                    Value::Symbol(s) => format!(":{s}"),
                    other => other.display(),
                };
                Ok(m.borrow().get(&key_str).cloned().unwrap_or(Value::Nil))
            }
            "put" | "insert" => {
                let key = args
                    .first()
                    .ok_or_else(|| RuntimeError::msg("put: missing key"))?;
                let val = args
                    .get(1)
                    .ok_or_else(|| RuntimeError::msg("put: missing value"))?
                    .clone();
                let key_str = match key {
                    Value::Str(s) => s.to_string(),
                    Value::Symbol(s) => format!(":{s}"),
                    other => other.display(),
                };
                m.borrow_mut().insert(key_str, val);
                Ok(recv.clone())
            }
            "keys" => {
                let ks: Vec<Value> = m.borrow().keys().map(|k| Value::str(k.clone())).collect();
                Ok(Value::array(ks))
            }
            "values" => {
                let vs: Vec<Value> = m.borrow().values().cloned().collect();
                Ok(Value::array(vs))
            }
            _ => Err(RuntimeError::msg(format!("Map has no method '{method}'"))),
        },
        Value::Int(_) | Value::Float(_) => match method {
            "to_s" => Ok(Value::str(recv.display())),
            "to_i" => match recv {
                Value::Int(i) => Ok(Value::Int(*i)),
                Value::Float(f) => Ok(Value::Int(*f as i64)),
                _ => unreachable!(),
            },
            "to_f" => match recv {
                Value::Int(i) => Ok(Value::Float(*i as f64)),
                Value::Float(f) => Ok(Value::Float(*f)),
                _ => unreachable!(),
            },
            "abs" => match recv {
                Value::Int(i) => Ok(Value::Int(i.abs())),
                Value::Float(f) => Ok(Value::Float(f.abs())),
                _ => unreachable!(),
            },
            _ => Err(RuntimeError::msg(format!(
                "Number has no method '{method}'"
            ))),
        },
        Value::Struct {
            name,
            fields,
            dynamic_methods,
        } => {
            if let Some(methods) = dynamic_methods {
                match method {
                    "def_method" => {
                        let name = method_name_arg("def_method", args.first())?;
                        let body = args
                            .get(1)
                            .ok_or_else(|| RuntimeError::msg("def_method: missing body"))?
                            .clone();
                        if !matches!(body, Value::Fn(_)) {
                            return Err(RuntimeError::type_err("Fn", &body));
                        }
                        methods.borrow_mut().insert(name, body);
                        return Ok(recv.clone());
                    }
                    "undef_method" => {
                        let name = method_name_arg("undef_method", args.first())?;
                        methods.borrow_mut().remove(&name);
                        return Ok(recv.clone());
                    }
                    "responds_to" | "respond_to" => {
                        let queried = method_name_arg(method, args.first())?;
                        let has_dynamic = methods.borrow().contains_key(&queried);
                        let has_dynamic_impl = env.has_dynamic_impl_method(name.as_ref(), &queried);
                        let has_static = env.has_impl_method(name.as_ref(), &queried);
                        let has_field = fields.borrow().contains_key(&queried);
                        return Ok(Value::Bool(
                            has_dynamic || has_dynamic_impl || has_static || has_field,
                        ));
                    }
                    "method_names" => {
                        let mut names = methods
                            .borrow()
                            .keys()
                            .cloned()
                            .chain(env.dynamic_impl_method_names(name.as_ref()))
                            .chain(env.impl_method_names(name.as_ref()))
                            .collect::<Vec<_>>();
                        names.sort();
                        names.dedup();
                        let names = names.into_iter().map(Value::sym).collect::<Vec<_>>();
                        return Ok(Value::array(names));
                    }
                    _ => {
                        if let Some(dynamic) = methods.borrow().get(method).cloned() {
                            let mut dynamic_args = Vec::with_capacity(args.len() + 1);
                            dynamic_args.push(recv.clone());
                            dynamic_args.extend(args);
                            return call_value(&dynamic, dynamic_args);
                        }
                    }
                }
            }

            if let Some(dynamic_impl_method) = env.get_dynamic_impl_method(name.as_ref(), method) {
                let mut dynamic_impl_args = Vec::with_capacity(args.len() + 1);
                dynamic_impl_args.push(recv.clone());
                dynamic_impl_args.extend(args);
                return call_value(&dynamic_impl_method, dynamic_impl_args);
            }

            if let Some(static_method) = env.get_impl_method(name.as_ref(), method) {
                let mut static_args = Vec::with_capacity(args.len() + 1);
                static_args.push(recv.clone());
                static_args.extend(args);
                return call_value(&static_method, static_args);
            }

            // Field access by method name remains a cheap managed-mode
            // convenience and preserves the previous struct dispatch behavior.
            if args.is_empty() {
                if let Some(v) = fields.borrow().get(method) {
                    return Ok(v.clone());
                }
            }

            if method != "method_missing" {
                if let Some(method_missing) = env.get_impl_method(name.as_ref(), "method_missing") {
                    return call_value(
                        &method_missing,
                        vec![recv.clone(), Value::sym(method), Value::array(args)],
                    );
                }
            }

            Err(RuntimeError::msg(format!(
                "Struct has no method '{method}'"
            )))
        }
        Value::ActorType(actor) => call_actor_type_method(actor, method, args, env),
        Value::ActorAddress(handle) => call_actor_address_method(handle, method, args),
        Value::MemoryStore { kind, backend, .. } => {
            // v3.3 KindGuard: validate the declared kind against the
            // backend's runtime tag before dispatch. Catches
            // struct-init mismatch and any future-IR discriminant loss.
            if let Err(mismatch) = backend.ensure_kind_matches(*kind) {
                return Err(RuntimeError::msg(format!(
                    "kind mismatch: declared {} but backend holds {} (rejected by KindGuard)",
                    mismatch.expected.name(),
                    mismatch.actual.name()
                )));
            }
            dispatch_memory_method(backend, method, args, recv)
        }
        Value::Variant { variant, .. } => match method {
            "retryable?" => Ok(Value::Bool(variant.as_str() == "Retry")),
            "message" => Ok(Value::str(variant.to_string())),
            "ok?" => Ok(Value::Bool(variant.as_str() == "Ok")),
            "to_s" => Ok(Value::str(recv.display())),
            _ => Err(RuntimeError::msg(format!(
                "Variant has no built-in method '{method}'"
            ))),
        },
        _ => Err(RuntimeError::msg(format!(
            "value of type {} has no method '{method}'",
            recv.type_name()
        ))),
    }
}

fn call_actor_type_method(
    actor: &Rc<ActorDef>,
    method: &str,
    args: Vec<Value>,
    env: &Rc<Env>,
) -> Result<Value, RuntimeError> {
    if method == "spawn" {
        if args.len() > 1 {
            return Err(RuntimeError::msg("spawn: expected at most one argument"));
        }
        let capacity = match args.first() {
            Some(Value::Int(n)) => Some(valid_mailbox_capacity(*n)?),
            Some(other) => return Err(RuntimeError::type_err("Int", other)),
            None => None,
        };
        return spawn_actor_address(Rc::clone(actor), env, capacity);
    }

    let actor_env = init_actor_env(actor, env)?;
    call_actor_handler(actor, &actor_env, method, args)
}

fn spawn_actor_address(
    actor: Rc<ActorDef>,
    env: &Rc<Env>,
    capacity: Option<usize>,
) -> Result<Value, RuntimeError> {
    let actor_env = init_actor_env(actor.as_ref(), env)?;
    Ok(Value::ActorAddress(Rc::new(ActorHandle {
        actor,
        env: actor_env,
        mailbox: RefCell::new(Default::default()),
        capacity: capacity.unwrap_or(DEFAULT_MANAGED_ACTOR_MAILBOX_CAPACITY),
    })))
}

fn init_actor_env(actor: &ActorDef, env: &Rc<Env>) -> Result<Rc<Env>, RuntimeError> {
    let actor_env = Env::new_child(env);
    for item in &actor.items {
        match item {
            ActorItem::Let(decl) => {
                let value = eval_expr(&decl.value, &actor_env)?;
                actor_env.define(&decl.name, value);
            }
            ActorItem::Memory(decl) => {
                actor_env.define(
                    &decl.name,
                    Value::MemoryStore {
                        kind: decl.kind,
                        name: decl.name.clone(),
                        backend: MemoryBackend::for_kind(decl.kind),
                    },
                );
            }
            ActorItem::Protocol(_) | ActorItem::Handler(_) => {}
        }
    }
    Ok(actor_env)
}

fn call_actor_handler(
    actor: &ActorDef,
    actor_env: &Rc<Env>,
    method: &str,
    args: Vec<Value>,
) -> Result<Value, RuntimeError> {
    let handler = actor
        .items
        .iter()
        .find_map(|item| match item {
            ActorItem::Handler(handler) if handler.name == method => Some(handler),
            _ => None,
        })
        .ok_or_else(|| {
            RuntimeError::msg(format!("actor {} has no handler '{method}'", actor.name))
        })?;

    let call_env = Env::new_child(actor_env);
    bind_params(&handler.params, args, &call_env)?;
    match stmt::exec_block_value(&handler.body, &call_env) {
        Ok(value) => Ok(value),
        Err(RuntimeError::Return(value)) => Ok(value),
        Err(err) => Err(err),
    }
}

fn call_actor_address_method(
    handle: &ActorHandle,
    method: &str,
    args: Vec<Value>,
) -> Result<Value, RuntimeError> {
    match method {
        "ask" => {
            let message = actor_message_arg("ask", args)?;
            call_actor_handler(&handle.actor, &handle.env, &message.method, message.args)
        }
        "tell" => {
            let message = actor_message_arg(method, args)?;
            if queue_actor_message(handle, message) {
                Ok(Value::Bool(true))
            } else {
                Err(RuntimeError::msg("tell: actor mailbox is full"))
            }
        }
        "try_tell" => {
            let message = actor_message_arg(method, args)?;
            Ok(Value::Bool(queue_actor_message(handle, message)))
        }
        "drain" => {
            if !args.is_empty() {
                return Err(RuntimeError::msg("drain: expected no arguments"));
            }
            let mut drained = 0;
            while let Some(message) = handle.mailbox.borrow_mut().pop_front() {
                call_actor_handler(&handle.actor, &handle.env, &message.method, message.args)?;
                drained += 1;
            }
            Ok(Value::Int(drained))
        }
        "mailbox_size" | "mailbox_len" => {
            if !args.is_empty() {
                return Err(RuntimeError::msg(format!(
                    "{method}: expected no arguments"
                )));
            }
            Ok(Value::Int(handle.mailbox.borrow().len() as i64))
        }
        "mailbox_capacity" | "capacity" => {
            if !args.is_empty() {
                return Err(RuntimeError::msg(format!(
                    "{method}: expected no arguments"
                )));
            }
            Ok(Value::Int(handle.capacity as i64))
        }
        _ => call_actor_handler(&handle.actor, &handle.env, method, args),
    }
}

fn queue_actor_message(handle: &ActorHandle, message: ActorMessage) -> bool {
    let mut mailbox = handle.mailbox.borrow_mut();
    if mailbox.len() >= handle.capacity {
        return false;
    }
    mailbox.push_back(message);
    true
}

fn actor_message_arg(api: &str, args: Vec<Value>) -> Result<ActorMessage, RuntimeError> {
    let method = method_name_arg(api, args.first())?;
    Ok(ActorMessage {
        method,
        args: args.into_iter().skip(1).collect(),
    })
}

fn valid_mailbox_capacity(n: i64) -> Result<usize, RuntimeError> {
    if n <= 0 || n as usize > MAX_MANAGED_ACTOR_MAILBOX_CAPACITY {
        return Err(RuntimeError::msg(format!(
            "actor mailbox capacity must be in 1..={MAX_MANAGED_ACTOR_MAILBOX_CAPACITY}, got {n}"
        )));
    }
    Ok(n as usize)
}

fn method_name_arg(api: &str, arg: Option<&Value>) -> Result<String, RuntimeError> {
    match arg.ok_or_else(|| RuntimeError::msg(format!("{api}: missing method name")))? {
        Value::Symbol(s) | Value::Str(s) => Ok(s.to_string()),
        other => Err(RuntimeError::type_err("Symbol", other)),
    }
}

/// Dispatch a Garnet-level method call on a `MemoryStore` to the correct
/// `garnet_memory` backend. The set of methods accepted depends on the
/// memory kind — calling `recent` on a working store, for instance, is an
/// error because working memory is not a temporal log.
fn dispatch_memory_method(
    backend: &MemoryBackend,
    method: &str,
    args: Vec<Value>,
    recv: &Value,
) -> Result<Value, RuntimeError> {
    use MemoryBackend::*;
    match (backend, method) {
        // ── WorkingStore ──
        (Working(s), "push") | (Working(s), "append") => {
            let v = args
                .into_iter()
                .next()
                .ok_or_else(|| RuntimeError::msg("push: missing value"))?;
            let idx = s.push(v);
            Ok(Value::Int(idx as i64))
        }
        (Working(s), "len") | (Working(s), "size") => Ok(Value::Int(s.len() as i64)),
        (Working(s), "clear") => {
            s.clear();
            Ok(recv.clone())
        }
        (Working(s), "snapshot") => Ok(Value::array(s.snapshot())),

        // ── EpisodeStore ──
        (Episodic(s), "append") | (Episodic(s), "push") => {
            let v = args
                .into_iter()
                .next()
                .ok_or_else(|| RuntimeError::msg("append: missing value"))?;
            s.append(v);
            Ok(Value::Int(s.len() as i64))
        }
        (Episodic(s), "len") | (Episodic(s), "size") => Ok(Value::Int(s.len() as i64)),
        (Episodic(s), "recent") => {
            let n = args
                .first()
                .and_then(|v| match v {
                    Value::Int(i) => Some(*i as usize),
                    _ => None,
                })
                .unwrap_or(0);
            let events = s.recent(n);
            Ok(Value::array(events.into_iter().map(|e| e.value).collect()))
        }

        // ── VectorIndex ──
        (Semantic(s), "insert") => {
            // insert(embedding: Array<Float>, value: Value)
            let emb_val = args
                .first()
                .ok_or_else(|| RuntimeError::msg("insert: missing embedding"))?;
            let value = args
                .get(1)
                .cloned()
                .ok_or_else(|| RuntimeError::msg("insert: missing value"))?;
            let embedding = value_to_f32_vec(emb_val)?;
            s.insert(embedding, value);
            Ok(Value::Int(s.len() as i64))
        }
        (Semantic(s), "search") => {
            let query_val = args
                .first()
                .ok_or_else(|| RuntimeError::msg("search: missing query"))?;
            let k = args
                .get(1)
                .and_then(|v| match v {
                    Value::Int(i) => Some(*i as usize),
                    _ => None,
                })
                .unwrap_or(1);
            let query = value_to_f32_vec(query_val)?;
            let results = s.search(&query, k);
            Ok(Value::array(
                results.into_iter().map(|(_score, v)| v).collect(),
            ))
        }
        (Semantic(s), "len") | (Semantic(s), "size") => Ok(Value::Int(s.len() as i64)),

        // ── WorkflowStore ──
        (Procedural(s), "register") => {
            let name = match args.first() {
                Some(Value::Str(n)) => n.to_string(),
                _ => return Err(RuntimeError::msg("register: name must be String")),
            };
            let initial = args
                .get(1)
                .cloned()
                .ok_or_else(|| RuntimeError::msg("register: missing initial value"))?;
            s.register(name, initial);
            Ok(recv.clone())
        }
        (Procedural(s), "find") | (Procedural(s), "current") => {
            let name = match args.first() {
                Some(Value::Str(n)) => n.to_string(),
                _ => return Err(RuntimeError::msg("find: name must be String")),
            };
            Ok(s.find(&name)
                .and_then(|w| w.current().cloned())
                .unwrap_or(Value::Nil))
        }
        (Procedural(s), "replay") => {
            let name = match args.first() {
                Some(Value::Str(n)) => n.to_string(),
                _ => return Err(RuntimeError::msg("replay: name must be String")),
            };
            let version = args
                .get(1)
                .and_then(|v| match v {
                    Value::Int(i) => Some(*i as usize),
                    _ => None,
                })
                .unwrap_or(0);
            Ok(s.replay(&name, version).unwrap_or(Value::Nil))
        }

        (b, m) => Err(RuntimeError::msg(format!(
            "{} has no method '{}'",
            b.kind_name(),
            m
        ))),
    }
}

/// Convert a Garnet `Value::Array` of numbers into a `Vec<f32>` for the
/// vector-index backend.
fn value_to_f32_vec(v: &Value) -> Result<Vec<f32>, RuntimeError> {
    match v {
        Value::Array(arr) => arr
            .borrow()
            .iter()
            .map(|el| match el {
                Value::Int(i) => Ok(*i as f32),
                Value::Float(f) => Ok(*f as f32),
                other => Err(RuntimeError::type_err("number", other)),
            })
            .collect(),
        other => Err(RuntimeError::type_err("Array<Number>", other)),
    }
}

fn access_field(recv: &Value, field: &str) -> Result<Value, RuntimeError> {
    match recv {
        Value::Struct { fields, .. } => fields
            .borrow()
            .get(field)
            .cloned()
            .ok_or_else(|| RuntimeError::msg(format!("no field '{field}'"))),
        Value::Map(m) => Ok(m.borrow().get(field).cloned().unwrap_or(Value::Nil)),
        _ => Err(RuntimeError::msg(format!(
            "cannot access field '{field}' on {}",
            recv.type_name()
        ))),
    }
}

fn access_index(recv: &Value, idx: &Value) -> Result<Value, RuntimeError> {
    match (recv, idx) {
        (Value::Array(arr), Value::Int(i)) => {
            let borrowed = arr.borrow();
            let n = borrowed.len() as i64;
            let real = if *i < 0 { n + i } else { *i };
            if real < 0 || real >= n {
                return Err(RuntimeError::IndexOOB { idx: *i });
            }
            Ok(borrowed[real as usize].clone())
        }
        (Value::Map(m), Value::Str(s)) => {
            Ok(m.borrow().get(s.as_str()).cloned().unwrap_or(Value::Nil))
        }
        (Value::Map(m), Value::Symbol(s)) => {
            let k = format!(":{s}");
            Ok(m.borrow().get(&k).cloned().unwrap_or(Value::Nil))
        }
        (Value::Str(s), Value::Int(i)) => {
            let ch = s.chars().nth(*i as usize).unwrap_or('\0');
            Ok(Value::str(ch.to_string()))
        }
        _ => Err(RuntimeError::msg(format!(
            "cannot index {} with {}",
            recv.type_name(),
            idx.type_name()
        ))),
    }
}
