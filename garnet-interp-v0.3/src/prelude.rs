//! Built-in functions and values, installed into the interpreter's global scope.
//!
//! The full source of this file is exposed as `PRELUDE_SOURCE` so the
//! deterministic build manifest (Paper VI Contribution 7) can include a
//! real content hash of the prelude — if anything in this file changes, the
//! manifest's `prelude_hash` changes. This fixes the v3.2 gap where the
//! manifest hashed a static version string instead of actual content.

use crate::env::Env;
use crate::error::RuntimeError;
use crate::value::{NativeFnValue, Value};
use std::rc::Rc;

/// Canonical source of the prelude. Hashed by the deterministic-build
/// manifest so any prelude change is visible in downstream manifests.
pub const PRELUDE_SOURCE: &str = include_str!("prelude.rs");

/// Version tag prepended to `PRELUDE_SOURCE` when the manifest hashes the
/// prelude. Bumped when the prelude's SEMANTIC contract changes in a way
/// that old manifests should explicitly reject (rather than just reporting
/// a different hash). Different from Cargo's package version.
pub const PRELUDE_VERSION: &str = "garnet-prelude-v0.3.3";

/// Install all prelude built-ins into the given global environment.
pub fn install(global: &Env) {
    // v3.4.1 Day 1 — wire stdlib host primitives BEFORE the legacy interp
    // prelude so the legacy entries (print/println/etc.) shadow any colliding
    // bridged entries. (No collisions at present; this ordering preserves
    // existing behavior while letting stdlib primitives fill gaps.)
    crate::stdlib_bridge::install(global);

    define_native(global, "print", None, prim_print);
    define_native(global, "println", None, prim_println);
    define_native(global, "type_of", Some(1), prim_type_of);
    define_native(global, "to_s", Some(1), prim_to_s);
    define_native(global, "to_i", Some(1), prim_to_i);
    define_native(global, "to_f", Some(1), prim_to_f);
    define_native(global, "len", Some(1), prim_len);
    define_native(global, "is_nil", Some(1), prim_is_nil);
    define_native(global, "array", None, prim_array);
    define_native(global, "map", None, prim_map);
    define_native(global, "ok", Some(1), prim_ok);
    define_native(global, "err", Some(1), prim_err);
    define_native(global, "some", Some(1), prim_some);
    define_native(global, "none", Some(0), prim_none);
    define_native(global, "assert", Some(1), prim_assert);
    define_native(global, "assert_eq", Some(2), prim_assert_eq);
    define_native(global, "log", Some(1), prim_log);
    define_native(global, "raise_msg", Some(1), prim_raise_msg);
    define_native(global, "filter", Some(2), prim_filter);
    define_native(global, "reduce", Some(3), prim_reduce);
    define_native(global, "compute_fulfillment_plan", None, prim_identity_stub);
    // Allow example programs to reference common type constructors.
    global.define("Ok", make_variant_constructor("Result", "Ok", 1));
    global.define("Err", make_variant_constructor("Result", "Err", 1));
    global.define("Some", make_variant_constructor("Option", "Some", 1));
    global.define("None", make_variant_zero("Option", "None"));
    global.define("true", Value::Bool(true));
    global.define("false", Value::Bool(false));
    global.define("nil", Value::Nil);
}

fn define_native(env: &Env, name: &'static str, arity: Option<usize>, ptr: crate::value::NativeFn) {
    env.define(
        name,
        Value::NativeFn(Rc::new(NativeFnValue { name, arity, ptr })),
    );
}

// ── Primitive functions ──

fn prim_print(args: Vec<Value>) -> Result<Value, RuntimeError> {
    let rendered: Vec<String> = args.iter().map(|v| v.display()).collect();
    print!("{}", rendered.join(" "));
    Ok(Value::Nil)
}

fn prim_println(args: Vec<Value>) -> Result<Value, RuntimeError> {
    let rendered: Vec<String> = args.iter().map(|v| v.display()).collect();
    println!("{}", rendered.join(" "));
    Ok(Value::Nil)
}

fn prim_type_of(args: Vec<Value>) -> Result<Value, RuntimeError> {
    Ok(Value::str(args[0].type_name()))
}

fn prim_to_s(args: Vec<Value>) -> Result<Value, RuntimeError> {
    Ok(Value::str(args[0].display()))
}

fn prim_to_i(args: Vec<Value>) -> Result<Value, RuntimeError> {
    match &args[0] {
        Value::Int(i) => Ok(Value::Int(*i)),
        Value::Float(f) => Ok(Value::Int(*f as i64)),
        Value::Str(s) => s
            .parse::<i64>()
            .map(Value::Int)
            .map_err(|_| RuntimeError::msg(format!("to_i: cannot parse {s:?}"))),
        Value::Bool(b) => Ok(Value::Int(if *b { 1 } else { 0 })),
        other => Err(RuntimeError::type_err("numeric or string", other)),
    }
}

fn prim_to_f(args: Vec<Value>) -> Result<Value, RuntimeError> {
    match &args[0] {
        Value::Int(i) => Ok(Value::Float(*i as f64)),
        Value::Float(f) => Ok(Value::Float(*f)),
        Value::Str(s) => s
            .parse::<f64>()
            .map(Value::Float)
            .map_err(|_| RuntimeError::msg(format!("to_f: cannot parse {s:?}"))),
        other => Err(RuntimeError::type_err("numeric or string", other)),
    }
}

fn prim_len(args: Vec<Value>) -> Result<Value, RuntimeError> {
    match &args[0] {
        Value::Str(s) => Ok(Value::Int(s.chars().count() as i64)),
        Value::Array(a) => Ok(Value::Int(a.borrow().len() as i64)),
        Value::Map(m) => Ok(Value::Int(m.borrow().len() as i64)),
        other => Err(RuntimeError::type_err("String/Array/Map", other)),
    }
}

fn prim_is_nil(args: Vec<Value>) -> Result<Value, RuntimeError> {
    Ok(Value::Bool(matches!(args[0], Value::Nil)))
}

fn prim_array(args: Vec<Value>) -> Result<Value, RuntimeError> {
    Ok(Value::array(args))
}

fn prim_map(args: Vec<Value>) -> Result<Value, RuntimeError> {
    if !args.len().is_multiple_of(2) {
        return Err(RuntimeError::msg("map(): expected even number of args"));
    }
    let mut entries = Vec::new();
    let mut iter = args.into_iter();
    while let (Some(k), Some(v)) = (iter.next(), iter.next()) {
        let key = match k {
            Value::Str(s) => s.to_string(),
            Value::Symbol(s) => format!(":{s}"),
            other => other.display(),
        };
        entries.push((key, v));
    }
    Ok(Value::map(entries))
}

fn prim_ok(args: Vec<Value>) -> Result<Value, RuntimeError> {
    Ok(Value::Variant {
        path: Rc::new(vec!["Result".to_string()]),
        variant: Rc::new("Ok".to_string()),
        fields: Rc::new(args),
    })
}

fn prim_err(args: Vec<Value>) -> Result<Value, RuntimeError> {
    Ok(Value::Variant {
        path: Rc::new(vec!["Result".to_string()]),
        variant: Rc::new("Err".to_string()),
        fields: Rc::new(args),
    })
}

fn prim_some(args: Vec<Value>) -> Result<Value, RuntimeError> {
    Ok(Value::Variant {
        path: Rc::new(vec!["Option".to_string()]),
        variant: Rc::new("Some".to_string()),
        fields: Rc::new(args),
    })
}

fn prim_none(_args: Vec<Value>) -> Result<Value, RuntimeError> {
    Ok(Value::Variant {
        path: Rc::new(vec!["Option".to_string()]),
        variant: Rc::new("None".to_string()),
        fields: Rc::new(vec![]),
    })
}

fn prim_assert(args: Vec<Value>) -> Result<Value, RuntimeError> {
    if !args[0].truthy() {
        return Err(RuntimeError::Raised(Value::str("assertion failed")));
    }
    Ok(Value::Nil)
}

fn prim_assert_eq(args: Vec<Value>) -> Result<Value, RuntimeError> {
    if !args[0].eq_deep(&args[1]) {
        return Err(RuntimeError::Raised(Value::str(format!(
            "assertion failed: {} != {}",
            args[0].display(),
            args[1].display()
        ))));
    }
    Ok(Value::Nil)
}

fn prim_log(args: Vec<Value>) -> Result<Value, RuntimeError> {
    eprintln!("[log] {}", args[0].display());
    Ok(Value::Nil)
}

fn prim_raise_msg(args: Vec<Value>) -> Result<Value, RuntimeError> {
    Err(RuntimeError::Raised(args[0].clone()))
}

fn prim_filter(args: Vec<Value>) -> Result<Value, RuntimeError> {
    let items = match &args[0] {
        Value::Array(a) => a.borrow().clone(),
        other => return Err(RuntimeError::type_err("Array", other)),
    };
    let pred = args[1].clone();
    let mut out = Vec::new();
    for item in items {
        if crate::eval::call_value(&pred, vec![item.clone()])?.truthy() {
            out.push(item);
        }
    }
    Ok(Value::array(out))
}

fn prim_reduce(args: Vec<Value>) -> Result<Value, RuntimeError> {
    let items = match &args[0] {
        Value::Array(a) => a.borrow().clone(),
        other => return Err(RuntimeError::type_err("Array", other)),
    };
    let mut acc = args[1].clone();
    let f = args[2].clone();
    for item in items {
        acc = crate::eval::call_value(&f, vec![acc, item])?;
    }
    Ok(acc)
}

fn prim_identity_stub(args: Vec<Value>) -> Result<Value, RuntimeError> {
    // Helper used by a couple of example programs that call otherwise-missing
    // helpers. The real implementations live in user code; the stub returns
    // the first argument or Nil so example parsing/evaluation doesn't break.
    Ok(args.into_iter().next().unwrap_or(Value::Nil))
}

fn make_variant_constructor(
    enum_name: &str,
    variant: &str,
    arity: usize,
) -> Value {
    let enum_name = enum_name.to_string();
    let variant = variant.to_string();
    Value::NativeFn(Rc::new(NativeFnValue {
        name: "<variant>",
        arity: Some(arity),
        ptr: {
            // We can't easily close over enum_name here with a fn pointer, so
            // we use a trick: stash them in a small static dispatch table.
            match (enum_name.as_str(), variant.as_str()) {
                ("Result", "Ok") => prim_ok,
                ("Result", "Err") => prim_err,
                ("Option", "Some") => prim_some,
                _ => prim_identity_stub,
            }
        },
    }))
}

fn make_variant_zero(enum_name: &str, variant: &str) -> Value {
    Value::Variant {
        path: Rc::new(vec![enum_name.to_string()]),
        variant: Rc::new(variant.to_string()),
        fields: Rc::new(vec![]),
    }
}
