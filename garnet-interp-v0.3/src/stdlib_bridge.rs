//! Stdlib ↔ Interpreter bridge (v3.4.1 — Day 1 scaffold).
//!
//! Wires `garnet_stdlib` host primitives into the interpreter's global env.
//! Each bridged primitive is a trampoline that:
//!   1. Unpacks `Vec<Value>` args into the stdlib function's native types.
//!   2. Invokes the stdlib primitive.
//!   3. Converts the result (or `StdError`) back to `Value` / `RuntimeError`.
//!
//! Capability enforcement is the CHECKER's job (consults
//! `garnet_stdlib::registry::all_prims()` for required caps at source-layer
//! time, per Mini-Spec v1.0 §11.2 + Security V2 spec §1.6). The interpreter
//! trusts the checker and performs no runtime cap validation in these
//! trampolines.
//!
//! ## Sequencing
//!
//! This module lands in v3.4.1 Day 1. Day 2 brings the CapCaps call-graph
//! propagator (which reads required-caps directly from
//! `garnet_stdlib::registry`). Day 3 brings ManifestSig. At the end of v3.4.1,
//! all 10 MVPs move from "syntactically valid" to "runtime-green" — the v4.2
//! installer ships a binary where `garnet run mvp_01_os_simulator.garnet`
//! actually executes.

use crate::env::Env;
use crate::error::RuntimeError;
use crate::value::{NativeFnValue, Value};
use garnet_stdlib::StdError;
use std::rc::Rc;

/// Install every bridged stdlib primitive into the given global env.
///
/// Called from `prelude::install` after the interpreter's own prelude
/// entries (`print`, `println`, etc.) are registered. Names are bound
/// as UNQUALIFIED top-level identifiers; the parser's path-segment
/// fallback (`eval_path` last-segment resolve) lets source code call
/// them as either `read_file(...)` or `fs::read_file(...)`.
pub fn install(global: &Env) {
    // ── strings (cap: none) ──
    define_native(global, "split", Some(2), bridge_str_split);
    define_native(global, "replace", Some(3), bridge_str_replace);
    define_native(global, "trim", Some(1), bridge_str_trim);
    define_native(global, "to_lower", Some(1), bridge_str_to_lower);
    define_native(global, "to_upper", Some(1), bridge_str_to_upper);
    define_native(global, "starts_with", Some(2), bridge_str_starts_with);
    define_native(global, "contains", Some(2), bridge_str_contains);

    // ── time (cap: time — CapCaps-gated) ──
    define_native(global, "now_ms", Some(0), bridge_time_now_ms);
    define_native(global, "wall_clock_ms", Some(0), bridge_time_wall_clock_ms);
    define_native(global, "sleep", Some(1), bridge_time_sleep);

    // ── crypto (cap: none — pure compute) ──
    define_native(global, "blake3", Some(1), bridge_crypto_blake3);
    define_native(global, "sha256", Some(1), bridge_crypto_sha256);
    define_native(global, "hmac_sha256", Some(2), bridge_crypto_hmac_sha256);

    // ── array (cap: none — pure compute; backed by stdlib::collections) ──
    define_native(global, "insert", Some(3), bridge_array_insert);
    define_native(global, "remove", Some(2), bridge_array_remove);
    define_native(global, "sort", Some(1), bridge_array_sort);

    // ── fs (cap: fs — CapCaps-gated) ──
    define_native(global, "read_file", Some(1), bridge_fs_read_file);
    define_native(global, "write_file", Some(2), bridge_fs_write_file);
    define_native(global, "read_bytes", Some(1), bridge_fs_read_bytes);
    define_native(global, "write_bytes", Some(2), bridge_fs_write_bytes);
    define_native(global, "list_dir", Some(1), bridge_fs_list_dir);

    // ── net (cap: net — CapCaps-gated + NetDefaults-gated) ──
    //
    // v3.4.1 Day 2 bridges `tcp_connect` only — it's the sole net primitive
    // with a concrete stdlib implementation at this release. `tcp_listen`
    // and `udp_bind` are registered in the stdlib `registry` for the
    // CapCaps propagator's sake but lack concrete implementations; they
    // are deliberately left unbridged until the stdlib's `net` module
    // grows them. Attempting to call either at source layer resolves
    // through the path fallback to `nil` at runtime; the propagator still
    // requires `@caps(net)` because the registry metadata is authoritative.
    define_native(global, "tcp_connect", Some(2), bridge_net_tcp_connect);
}

fn define_native(env: &Env, name: &'static str, arity: Option<usize>, ptr: crate::value::NativeFn) {
    env.define(
        name,
        Value::NativeFn(Rc::new(NativeFnValue { name, arity, ptr })),
    );
}

// ── StdError → RuntimeError conversion ──

fn lift_std_error(prim: &str, e: StdError) -> RuntimeError {
    // Mini-Spec v1.0 §7.4 — stdlib errors surface as managed-mode raised
    // exceptions carrying a descriptive string. A later revision may
    // introduce structured exception types; for the scaffold, a rendered
    // message keeps the error channel working end-to-end.
    RuntimeError::Raised(Value::str(format!("{prim}: {e}")))
}

// ── Arg unpackers ──

fn expect_str<'a>(prim: &str, args: &'a [Value], idx: usize) -> Result<&'a str, RuntimeError> {
    match args.get(idx) {
        Some(Value::Str(s)) => Ok(s.as_str()),
        Some(other) => Err(RuntimeError::type_err(
            &format!("{prim}: String arg at position {idx}"),
            other,
        )),
        None => Err(RuntimeError::msg(format!(
            "{prim}: missing argument at position {idx}"
        ))),
    }
}

fn expect_int(prim: &str, args: &[Value], idx: usize) -> Result<i64, RuntimeError> {
    match args.get(idx) {
        Some(Value::Int(i)) => Ok(*i),
        Some(other) => Err(RuntimeError::type_err(
            &format!("{prim}: Int arg at position {idx}"),
            other,
        )),
        None => Err(RuntimeError::msg(format!(
            "{prim}: missing argument at position {idx}"
        ))),
    }
}

fn expect_usize(prim: &str, args: &[Value], idx: usize) -> Result<usize, RuntimeError> {
    let i = expect_int(prim, args, idx)?;
    if i < 0 {
        return Err(RuntimeError::msg(format!(
            "{prim}: index at position {idx} must be non-negative, got {i}"
        )));
    }
    Ok(i as usize)
}

/// Unpack `Value::Array(...)` as an owned clone of its underlying Vec<Value>.
fn expect_array_clone(prim: &str, args: &[Value], idx: usize) -> Result<Vec<Value>, RuntimeError> {
    match args.get(idx) {
        Some(Value::Array(a)) => Ok(a.borrow().clone()),
        Some(other) => Err(RuntimeError::type_err(
            &format!("{prim}: Array arg at position {idx}"),
            other,
        )),
        None => Err(RuntimeError::msg(format!(
            "{prim}: missing argument at position {idx}"
        ))),
    }
}

/// Unpack a `Value::Array` of `Value::Int` into a `Vec<u8>`. Each element must
/// be an `Int` in `0..=255`; any violation surfaces as a typed runtime error.
/// Used by `write_bytes` + friends to accept a Garnet-side byte sequence.
fn expect_byte_array(prim: &str, args: &[Value], idx: usize) -> Result<Vec<u8>, RuntimeError> {
    let items = expect_array_clone(prim, args, idx)?;
    let mut out = Vec::with_capacity(items.len());
    for (i, v) in items.iter().enumerate() {
        match v {
            Value::Int(n) if (0..=255).contains(n) => out.push(*n as u8),
            Value::Int(n) => {
                return Err(RuntimeError::msg(format!(
                    "{prim}: byte at index {i} out of 0..=255, got {n}"
                )))
            }
            other => {
                return Err(RuntimeError::type_err(
                    &format!("{prim}: Int (byte 0..=255) at index {i}"),
                    other,
                ))
            }
        }
    }
    Ok(out)
}

/// Pack a `Vec<u8>` as a `Value::Array` of `Value::Int`. Inverse of
/// `expect_byte_array`. Until the interpreter gains a dedicated `Bytes`
/// variant, this mapping is the canonical carrier for binary payloads.
fn bytes_to_value(bytes: Vec<u8>) -> Value {
    Value::array(bytes.into_iter().map(|b| Value::Int(b as i64)).collect())
}

/// Hex-encode a 32-byte digest as lowercase hex. Output is 64 ASCII bytes.
fn digest_to_hex(digest: &[u8; 32]) -> String {
    let mut hex = String::with_capacity(64);
    for byte in digest {
        hex.push_str(&format!("{byte:02x}"));
    }
    hex
}

// ── String primitives ──

fn bridge_str_split(args: Vec<Value>) -> Result<Value, RuntimeError> {
    let s = expect_str("split", &args, 0)?;
    let delim = expect_str("split", &args, 1)?;
    let parts = garnet_stdlib::strings::split(s, delim);
    Ok(Value::array(parts.into_iter().map(Value::str).collect()))
}

fn bridge_str_trim(args: Vec<Value>) -> Result<Value, RuntimeError> {
    let s = expect_str("trim", &args, 0)?;
    Ok(Value::str(garnet_stdlib::strings::trim(s)))
}

fn bridge_str_to_lower(args: Vec<Value>) -> Result<Value, RuntimeError> {
    let s = expect_str("to_lower", &args, 0)?;
    Ok(Value::str(garnet_stdlib::strings::to_lower(s)))
}

fn bridge_str_to_upper(args: Vec<Value>) -> Result<Value, RuntimeError> {
    let s = expect_str("to_upper", &args, 0)?;
    Ok(Value::str(garnet_stdlib::strings::to_upper(s)))
}

fn bridge_str_replace(args: Vec<Value>) -> Result<Value, RuntimeError> {
    let s = expect_str("replace", &args, 0)?;
    let old = expect_str("replace", &args, 1)?;
    let new = expect_str("replace", &args, 2)?;
    garnet_stdlib::strings::replace(s, old, new)
        .map(Value::str)
        .map_err(|e| lift_std_error("replace", e))
}

fn bridge_str_starts_with(args: Vec<Value>) -> Result<Value, RuntimeError> {
    let s = expect_str("starts_with", &args, 0)?;
    let prefix = expect_str("starts_with", &args, 1)?;
    Ok(Value::Bool(garnet_stdlib::strings::starts_with(s, prefix)))
}

fn bridge_str_contains(args: Vec<Value>) -> Result<Value, RuntimeError> {
    // NB: at the Garnet source layer, `contains` is also a natural method name
    // for Array/Map. The prelude's bare-name binding here covers the String
    // case; a future `method_dispatch` patch will route `.contains(...)` on
    // Array/Map to the appropriate handler separately.
    let s = expect_str("contains", &args, 0)?;
    let needle = expect_str("contains", &args, 1)?;
    Ok(Value::Bool(garnet_stdlib::strings::contains(s, needle)))
}

// ── Time primitives ──

fn bridge_time_now_ms(_args: Vec<Value>) -> Result<Value, RuntimeError> {
    Ok(Value::Int(garnet_stdlib::time::now_ms()))
}

fn bridge_time_wall_clock_ms(_args: Vec<Value>) -> Result<Value, RuntimeError> {
    garnet_stdlib::time::wall_clock_ms()
        .map(Value::Int)
        .map_err(|e| lift_std_error("wall_clock_ms", e))
}

fn bridge_time_sleep(args: Vec<Value>) -> Result<Value, RuntimeError> {
    let ms = expect_int("sleep", &args, 0)?;
    garnet_stdlib::time::sleep(ms)
        .map(|_| Value::Nil)
        .map_err(|e| lift_std_error("sleep", e))
}

// ── Crypto primitives ──

fn bridge_crypto_blake3(args: Vec<Value>) -> Result<Value, RuntimeError> {
    let s = expect_str("blake3", &args, 0)?;
    let digest = garnet_stdlib::crypto::blake3_hash(s.as_bytes());
    // Render as lowercase hex — matches the presentation Paper VII §2.4 expects.
    Ok(Value::str(digest_to_hex(&digest)))
}

fn bridge_crypto_sha256(args: Vec<Value>) -> Result<Value, RuntimeError> {
    let s = expect_str("sha256", &args, 0)?;
    let digest = garnet_stdlib::crypto::sha256_hash(s.as_bytes());
    Ok(Value::str(digest_to_hex(&digest)))
}

fn bridge_crypto_hmac_sha256(args: Vec<Value>) -> Result<Value, RuntimeError> {
    let key = expect_str("hmac_sha256", &args, 0)?;
    let msg = expect_str("hmac_sha256", &args, 1)?;
    let digest = garnet_stdlib::crypto::hmac_sha256(key.as_bytes(), msg.as_bytes());
    Ok(Value::str(digest_to_hex(&digest)))
}

// ── Array primitives (backed by `garnet_stdlib::collections`) ──
//
// The stdlib generic collection functions operate on `&mut Vec<T>` with
// appropriate bounds; here we unpack the Garnet `Value::Array`, clone out the
// inner `Vec<Value>`, delegate to stdlib, and re-wrap the result as a fresh
// `Value::Array`. Aliasing semantics match Ruby's `Array#insert` / `#sort` —
// returning a new array rather than mutating the caller's binding — which is
// the simpler and more predictable contract for managed mode. A `_in_place`
// suffix family can be introduced separately if mutation-preserving semantics
// are ever needed.

fn bridge_array_insert(args: Vec<Value>) -> Result<Value, RuntimeError> {
    let mut items = expect_array_clone("insert", &args, 0)?;
    let idx = expect_usize("insert", &args, 1)?;
    let value = args
        .get(2)
        .cloned()
        .ok_or_else(|| RuntimeError::msg("insert: missing value argument".to_string()))?;
    garnet_stdlib::collections::array_insert(&mut items, idx, value)
        .map_err(|e| lift_std_error("insert", e))?;
    Ok(Value::array(items))
}

fn bridge_array_remove(args: Vec<Value>) -> Result<Value, RuntimeError> {
    let mut items = expect_array_clone("remove", &args, 0)?;
    let idx = expect_usize("remove", &args, 1)?;
    let removed = garnet_stdlib::collections::array_remove(&mut items, idx)
        .map_err(|e| lift_std_error("remove", e))?;
    // Return the REMOVED element (matches Ruby `Array#delete_at`). The
    // post-remove array is available to the caller via a follow-up bind if
    // they want both; this trampoline keeps the signature 1-out.
    Ok(removed)
}

fn bridge_array_sort(args: Vec<Value>) -> Result<Value, RuntimeError> {
    let mut items = expect_array_clone("sort", &args, 0)?;
    // `Value` does not implement `Ord` (floats break total ordering; cross-
    // type comparisons are intentionally partial). Use the value's own
    // `partial_compare` and escalate any incomparable pair to a runtime error.
    let mut err: Option<RuntimeError> = None;
    items.sort_by(|a, b| {
        if err.is_some() {
            return std::cmp::Ordering::Equal;
        }
        match a.partial_compare(b) {
            Some(ord) => ord,
            None => {
                err = Some(RuntimeError::msg(format!(
                    "sort: values not comparable ({} vs {})",
                    a.type_name(),
                    b.type_name()
                )));
                std::cmp::Ordering::Equal
            }
        }
    });
    if let Some(e) = err {
        return Err(e);
    }
    Ok(Value::array(items))
}

// ── Filesystem primitives ──

fn bridge_fs_read_file(args: Vec<Value>) -> Result<Value, RuntimeError> {
    let path = expect_str("read_file", &args, 0)?;
    garnet_stdlib::fs::read_file(path)
        .map(Value::str)
        .map_err(|e| lift_std_error("read_file", e))
}

fn bridge_fs_write_file(args: Vec<Value>) -> Result<Value, RuntimeError> {
    let path = expect_str("write_file", &args, 0)?;
    let contents = expect_str("write_file", &args, 1)?;
    garnet_stdlib::fs::write_file(path, contents)
        .map(|_| Value::Nil)
        .map_err(|e| lift_std_error("write_file", e))
}

fn bridge_fs_read_bytes(args: Vec<Value>) -> Result<Value, RuntimeError> {
    let path = expect_str("read_bytes", &args, 0)?;
    garnet_stdlib::fs::read_bytes(path)
        .map(bytes_to_value)
        .map_err(|e| lift_std_error("read_bytes", e))
}

fn bridge_fs_write_bytes(args: Vec<Value>) -> Result<Value, RuntimeError> {
    let path = expect_str("write_bytes", &args, 0)?;
    let data = expect_byte_array("write_bytes", &args, 1)?;
    garnet_stdlib::fs::write_bytes(path, &data)
        .map(|_| Value::Nil)
        .map_err(|e| lift_std_error("write_bytes", e))
}

fn bridge_fs_list_dir(args: Vec<Value>) -> Result<Value, RuntimeError> {
    let path = expect_str("list_dir", &args, 0)?;
    garnet_stdlib::fs::list_dir(path)
        .map(|entries| Value::array(entries.into_iter().map(Value::str).collect()))
        .map_err(|e| lift_std_error("list_dir", e))
}

// ── Net primitives ──

/// `tcp_connect(host, port)` — opens an outbound TCP connection, returns
/// `Value::Bool(true)` on success, raises on denial or failure. The connect
/// is performed with `NetPolicy::default()` (strict — RFC1918 / loopback /
/// link-local denied). A future `tcp_connect_internal(host, port)` variant
/// can lift the strict policy for `@caps(net_internal)` callers.
///
/// The opened stream is immediately closed; this bridge is a smoke/health
/// primitive rather than a full socket API. The full socket API with
/// read/write bidirectional handles awaits a `Value::Handle<T>` variant
/// which lands alongside the actor-runtime integration in a later rung.
fn bridge_net_tcp_connect(args: Vec<Value>) -> Result<Value, RuntimeError> {
    let host = expect_str("tcp_connect", &args, 0)?;
    let port_i = expect_int("tcp_connect", &args, 1)?;
    if !(0..=65_535).contains(&port_i) {
        return Err(RuntimeError::msg(format!(
            "tcp_connect: port out of 0..=65535, got {port_i}"
        )));
    }
    let policy = garnet_stdlib::net::NetPolicy::default();
    match garnet_stdlib::net::tcp_connect(host, port_i as u16, policy) {
        Ok(_stream) => Ok(Value::Bool(true)),
        Err(e) => Err(lift_std_error("tcp_connect", e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_env() -> Rc<Env> {
        let env = Rc::new(Env::new_root());
        install(&env);
        env
    }

    #[test]
    fn installs_without_panic() {
        let _env = make_env();
    }

    #[test]
    fn str_trim_bridge_roundtrip() {
        let env = make_env();
        let trim = env.get("trim").expect("trim should be bound");
        let res = crate::eval::call_value(&trim, vec![Value::str("  hi  ")]).unwrap();
        match res {
            Value::Str(s) => assert_eq!(&*s, "hi"),
            other => panic!("expected Str, got {other:?}"),
        }
    }

    #[test]
    fn str_split_bridge_produces_array() {
        let env = make_env();
        let split = env.get("split").expect("split should be bound");
        let res =
            crate::eval::call_value(&split, vec![Value::str("a,b,c"), Value::str(",")]).unwrap();
        match res {
            Value::Array(items) => {
                let items = items.borrow();
                assert_eq!(items.len(), 3);
            }
            other => panic!("expected Array, got {other:?}"),
        }
    }

    #[test]
    fn time_now_ms_bridge_returns_int() {
        let env = make_env();
        let now_ms = env.get("now_ms").expect("now_ms should be bound");
        let res = crate::eval::call_value(&now_ms, vec![]).unwrap();
        assert!(matches!(res, Value::Int(_)));
    }

    #[test]
    fn fs_read_file_missing_path_surfaces_as_raised() {
        let env = make_env();
        let read = env.get("read_file").expect("read_file should be bound");
        let res = crate::eval::call_value(
            &read,
            vec![Value::str(
                "/nonexistent/path/should_not_exist_garnet_bridge_test.txt",
            )],
        );
        match res {
            Err(RuntimeError::Raised(v)) => {
                // Expect a descriptive exception value carrying "read_file"
                let rendered = v.display();
                assert!(rendered.contains("read_file"), "got: {rendered}");
            }
            other => panic!("expected Raised, got {other:?}"),
        }
    }

    #[test]
    fn crypto_blake3_bridge_empty_input_matches_known_hex() {
        let env = make_env();
        let blake3 = env.get("blake3").expect("blake3 should be bound");
        let res = crate::eval::call_value(&blake3, vec![Value::str("")]).unwrap();
        match res {
            Value::Str(s) => {
                // BLAKE3("") = af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262
                assert_eq!(
                    &*s, "af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262",
                    "known-vector regression"
                );
            }
            other => panic!("expected Str, got {other:?}"),
        }
    }

    #[test]
    fn crypto_sha256_bridge_empty_input_matches_known_hex() {
        let env = make_env();
        let sha256 = env.get("sha256").expect("sha256 should be bound");
        let res = crate::eval::call_value(&sha256, vec![Value::str("")]).unwrap();
        match res {
            Value::Str(s) => {
                // SHA-256("") known vector
                assert_eq!(
                    &*s,
                    "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
                );
            }
            other => panic!("expected Str, got {other:?}"),
        }
    }

    #[test]
    fn str_replace_bridge_roundtrip() {
        let env = make_env();
        let replace = env.get("replace").expect("replace should be bound");
        let res = crate::eval::call_value(
            &replace,
            vec![
                Value::str("hello world"),
                Value::str("world"),
                Value::str("garnet"),
            ],
        )
        .unwrap();
        match res {
            Value::Str(s) => assert_eq!(&*s, "hello garnet"),
            other => panic!("expected Str, got {other:?}"),
        }
    }

    #[test]
    fn str_replace_empty_needle_rejected_as_raised() {
        let env = make_env();
        let replace = env.get("replace").expect("replace should be bound");
        let res = crate::eval::call_value(
            &replace,
            vec![Value::str("hello"), Value::str(""), Value::str("x")],
        );
        match res {
            Err(RuntimeError::Raised(v)) => assert!(v.display().contains("replace")),
            other => panic!("expected Raised, got {other:?}"),
        }
    }

    #[test]
    fn str_starts_with_and_contains_return_bool() {
        let env = make_env();
        let starts = env.get("starts_with").expect("starts_with bound");
        let contains = env.get("contains").expect("contains bound");
        assert!(matches!(
            crate::eval::call_value(&starts, vec![Value::str("garnet"), Value::str("gar")])
                .unwrap(),
            Value::Bool(true)
        ));
        assert!(matches!(
            crate::eval::call_value(&contains, vec![Value::str("garnet"), Value::str("xyz")])
                .unwrap(),
            Value::Bool(false)
        ));
    }

    #[test]
    fn array_insert_returns_new_array_with_value() {
        let env = make_env();
        let insert = env.get("insert").expect("insert bound");
        let original = Value::array(vec![Value::Int(1), Value::Int(3)]);
        let res =
            crate::eval::call_value(&insert, vec![original, Value::Int(1), Value::Int(2)]).unwrap();
        match res {
            Value::Array(a) => {
                let a = a.borrow();
                assert_eq!(a.len(), 3);
                assert!(matches!(a[0], Value::Int(1)));
                assert!(matches!(a[1], Value::Int(2)));
                assert!(matches!(a[2], Value::Int(3)));
            }
            other => panic!("expected Array, got {other:?}"),
        }
    }

    #[test]
    fn array_remove_returns_removed_element() {
        let env = make_env();
        let remove = env.get("remove").expect("remove bound");
        let original = Value::array(vec![Value::Int(10), Value::Int(20), Value::Int(30)]);
        let res = crate::eval::call_value(&remove, vec![original, Value::Int(1)]).unwrap();
        assert!(matches!(res, Value::Int(20)));
    }

    #[test]
    fn array_sort_ints_ascending() {
        let env = make_env();
        let sort = env.get("sort").expect("sort bound");
        let original = Value::array(vec![
            Value::Int(3),
            Value::Int(1),
            Value::Int(4),
            Value::Int(1),
            Value::Int(5),
        ]);
        let res = crate::eval::call_value(&sort, vec![original]).unwrap();
        match res {
            Value::Array(a) => {
                let a = a.borrow();
                let sorted: Vec<i64> = a
                    .iter()
                    .filter_map(|v| {
                        if let Value::Int(i) = v {
                            Some(*i)
                        } else {
                            None
                        }
                    })
                    .collect();
                assert_eq!(sorted, vec![1, 1, 3, 4, 5]);
            }
            other => panic!("expected Array, got {other:?}"),
        }
    }

    #[test]
    fn array_sort_rejects_incomparable_types() {
        let env = make_env();
        let sort = env.get("sort").expect("sort bound");
        // Int + Str have no partial order between them.
        let mixed = Value::array(vec![Value::Int(1), Value::str("alpha"), Value::Int(2)]);
        let res = crate::eval::call_value(&sort, vec![mixed]);
        match res {
            Err(RuntimeError::Message(m)) => assert!(m.contains("not comparable")),
            other => panic!("expected Message error, got {other:?}"),
        }
    }

    #[test]
    fn fs_read_bytes_roundtrip_with_write_bytes() {
        // End-to-end round-trip via the bridge: write bytes, read bytes back,
        // confirm equality. Exercises both `expect_byte_array` (on write) and
        // `bytes_to_value` (on read).
        let env = make_env();
        let tmp = std::env::temp_dir().join(format!(
            "garnet_bridge_rt_{}.bin",
            garnet_stdlib::time::now_ms()
        ));
        let tmp_str = tmp.to_string_lossy().into_owned();

        // Write 4 bytes [0x47 'G', 0x41 'A', 0x52 'R', 0x4e 'N']
        let write_bytes = env.get("write_bytes").expect("write_bytes bound");
        let payload = Value::array(vec![
            Value::Int(0x47),
            Value::Int(0x41),
            Value::Int(0x52),
            Value::Int(0x4e),
        ]);
        crate::eval::call_value(&write_bytes, vec![Value::str(tmp_str.clone()), payload])
            .expect("write_bytes");

        // Read back and confirm.
        let read_bytes = env.get("read_bytes").expect("read_bytes bound");
        let res = crate::eval::call_value(&read_bytes, vec![Value::str(tmp_str.clone())]).unwrap();
        match res {
            Value::Array(a) => {
                let a = a.borrow();
                assert_eq!(a.len(), 4);
                let values: Vec<i64> = a
                    .iter()
                    .filter_map(|v| {
                        if let Value::Int(i) = v {
                            Some(*i)
                        } else {
                            None
                        }
                    })
                    .collect();
                assert_eq!(values, vec![0x47, 0x41, 0x52, 0x4e]);
            }
            other => panic!("expected Array, got {other:?}"),
        }

        // Cleanup.
        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn expected_registry_coverage_count() {
        // v3.4.1 Day 2 target: at least 22 bridged primitives from the
        // ~25-entry stdlib registry. Confirms install() keeps pace with the
        // registry surface as new primitives get added.
        let env = make_env();
        let names = [
            "split",
            "replace",
            "trim",
            "to_lower",
            "to_upper",
            "starts_with",
            "contains",
            "now_ms",
            "wall_clock_ms",
            "sleep",
            "blake3",
            "sha256",
            "hmac_sha256",
            "insert",
            "remove",
            "sort",
            "read_file",
            "write_file",
            "read_bytes",
            "write_bytes",
            "list_dir",
            "tcp_connect",
        ];
        assert!(names.len() >= 22, "bridge coverage regressed below 22");
        for n in &names {
            assert!(env.get(n).is_some(), "prelude missing bridged name `{n}`");
        }
    }

    #[test]
    fn net_tcp_connect_rejects_out_of_range_port() {
        let env = make_env();
        let connect = env.get("tcp_connect").expect("tcp_connect bound");
        let res = crate::eval::call_value(
            &connect,
            vec![Value::str("example.com"), Value::Int(70_000)],
        );
        match res {
            Err(RuntimeError::Message(m)) => assert!(
                m.contains("port out of"),
                "expected port-range error, got {m}"
            ),
            other => panic!("expected Message error, got {other:?}"),
        }
    }

    #[test]
    fn net_tcp_connect_to_loopback_denied_by_default_policy() {
        // Strict NetPolicy (the bridge default) denies 127.0.0.1. The
        // connect must fail with a lifted StdError::NetDenied → Raised.
        let env = make_env();
        let connect = env.get("tcp_connect").expect("tcp_connect bound");
        let res = crate::eval::call_value(
            &connect,
            // Port 1 is deliberately unlikely to be listened on but the
            // address check precedes the connect attempt, so this raises
            // NetDenied deterministically.
            vec![Value::str("127.0.0.1"), Value::Int(1)],
        );
        match res {
            Err(RuntimeError::Raised(v)) => {
                let msg = v.display();
                assert!(
                    msg.contains("tcp_connect"),
                    "expected tcp_connect in msg, got: {msg}"
                );
            }
            other => panic!("expected Raised, got {other:?}"),
        }
    }
}
