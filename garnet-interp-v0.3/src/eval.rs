//! Expression evaluator — implements the 11-level Pratt precedence tower at runtime.

use crate::control::{eval_if, eval_match, eval_try};
use crate::env::Env;
use crate::error::RuntimeError;
use crate::stmt;
use crate::value::{bind_params, FnValue, MemoryBackend, TypeValue, Value};
use garnet_parser::ast::{BinOp, ClosureBody, Expr, StringLit, UnOp};
use garnet_parser::token::StrPart;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

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
        Expr::Closure { params, body, .. } => {
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
            })))
        }
        Expr::Spawn { expr, .. } => {
            // Rung-3 interpreter runs `spawn` synchronously. The actor
            // runtime (Rung 6) supplies the real parallelism later.
            eval_expr(expr, env)
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
    // Fallback: treat the last segment as a top-level name. This lets simple
    // module-qualified calls like `Storage::read_block` resolve against global
    // names while the full module system is still stubbed.
    let last = segs
        .last()
        .ok_or_else(|| RuntimeError::Message("empty path expression".into()))?;
    env.get(last)
        .ok_or_else(|| RuntimeError::Message(format!("unresolved path: {}", segs.join("::"))))
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
                (Int(a), Int(b)) => Ok(Int(a / b)),
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
                    Ok(Int(a % b))
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

fn call_fn(f: &FnValue, args: Vec<Value>) -> Result<Value, RuntimeError> {
    let call_env = Env::new_child(&f.captured);
    bind_params(&f.def.params, args, &call_env)?;
    // Execute body.
    for s in &f.def.body.stmts {
        match stmt::exec_stmt(s, &call_env) {
            Ok(()) => {}
            Err(RuntimeError::Return(v)) => return Ok(v),
            Err(e) => return Err(e),
        }
    }
    if let Some(tail) = &f.def.body.tail_expr {
        match eval_expr(tail, &call_env) {
            Ok(v) => Ok(v),
            Err(RuntimeError::Return(v)) => Ok(v),
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
    _env: &Rc<Env>,
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
        Value::Struct { fields, .. } => {
            // User-written methods aren't wired up until Rung 4 resolves `impl`
            // blocks. Fall back to field access by name if possible.
            if args.is_empty() {
                if let Some(v) = fields.borrow().get(method) {
                    return Ok(v.clone());
                }
            }
            Err(RuntimeError::msg(format!(
                "struct method dispatch for '{method}' requires Rung 4 impl resolution"
            )))
        }
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
