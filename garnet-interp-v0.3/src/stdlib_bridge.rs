//! Stdlib ↔ Interpreter bridge (v3.4.1 — Day 1 scaffold).
//!
//! Wires `garnet_stdlib` host primitives into the interpreter's global env.
//! Each bridged primitive is a trampoline that:
//!   1. Unpacks `Vec<Value>` args into the stdlib function's native types.
//!   2. Invokes the stdlib primitive.
//!   3. Converts the result (or `StdError`) back to `Value` / `RuntimeError`.
//!
//! Capability enforcement is checker-first, with an interpreter runtime backstop
//! for host-authority trampolines. `garnet run` can execute without first running
//! `garnet check`, so env/process/fs/net/log-to-file bridges reject undeclared
//! authority from the active call chain; S92 additionally requires process launch
//! bridges to see `@caps(proc)` on the program entry point. Honest scope: this
//! is interpreter-scoped and does not imply VM `@caps` enforcement.
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
use crate::value::{MemoryBackend, NativeFnValue, Value};
use garnet_parser::ast::MemoryKind;
use garnet_stdlib::StdError;
use serde_json::Value as JsonValue;
use std::cell::RefCell;
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

    // ════════════════════════════════════════════════════════════════
    // S21 — Layer-0/1 qualified dispatch (S17 registry → runnable).
    //
    // Bound under their FULLY-QUALIFIED names so `eval_path`'s qualified-first
    // resolution finds them without colliding with bare prelude builtins that
    // share a last segment (`map` = Map ctor, `ok`/`err` = Result builders).
    // `core::math` / `std::base64` / `std::json` dispatch the `garnet_stdlib`
    // host functions directly; `core::iter` / `core::cmp` are bridged at the
    // Value layer (Garnet's dynamic `Value` can't be passed as the stdlib's
    // monomorphic generic `T`) with the stdlib generics as the Rust reference.
    // ════════════════════════════════════════════════════════════════

    // ── core::math (cap: none — total numeric, dispatches garnet_stdlib::math) ──
    define_native(global, "core::math::abs", Some(1), bridge_math_abs);
    define_native(global, "core::math::sqrt", Some(1), bridge_math_sqrt);
    define_native(global, "core::math::pow", Some(2), bridge_math_pow);
    define_native(global, "core::math::floor", Some(1), bridge_math_floor);
    define_native(global, "core::math::ceil", Some(1), bridge_math_ceil);
    define_native(global, "core::math::round", Some(1), bridge_math_round);

    // ── core::cmp (cap: none — Value-level ordering) ──
    define_native(global, "core::cmp::min", Some(2), bridge_cmp_min);
    define_native(global, "core::cmp::max", Some(2), bridge_cmp_max);
    define_native(global, "core::cmp::clamp", Some(3), bridge_cmp_clamp);
    define_native(global, "core::cmp::ordering", Some(2), bridge_cmp_ordering);

    // ── core::iter (cap: none — higher-order via call_value; Value-level) ──
    define_native(global, "core::iter::map", Some(2), bridge_iter_map);
    define_native(global, "core::iter::filter", Some(2), bridge_iter_filter);
    define_native(global, "core::iter::fold", Some(3), bridge_iter_fold);
    define_native(global, "core::iter::take", Some(2), bridge_iter_take);
    define_native(global, "core::iter::drop", Some(2), bridge_iter_drop);
    define_native(
        global,
        "core::iter::enumerate",
        Some(1),
        bridge_iter_enumerate,
    );
    // S28: the last three registered core::iter combinators (Value-level, not
    // higher-order). `zip`/`chain` pair/concat two arrays; `collect` materializes
    // a sequence (a `Range`, or an array passed through) into an owned Array.
    define_native(global, "core::iter::zip", Some(2), bridge_iter_zip);
    define_native(global, "core::iter::collect", Some(1), bridge_iter_collect);
    define_native(global, "core::iter::chain", Some(2), bridge_iter_chain);

    // ── core::result (cap: none — Value-level; map/and_then/or_else higher-order
    //    via call_value). Bound qualified so `core::result::map` does not collide
    //    with the bare `map` (Map constructor) on the last-segment fallback. ──
    define_native(global, "core::result::ok", Some(1), bridge_result_ok);
    define_native(global, "core::result::err", Some(1), bridge_result_err);
    define_native(global, "core::result::map", Some(2), bridge_result_map);
    define_native(
        global,
        "core::result::and_then",
        Some(2),
        bridge_result_and_then,
    );
    define_native(
        global,
        "core::result::or_else",
        Some(2),
        bridge_result_or_else,
    );
    define_native(
        global,
        "core::result::unwrap_or",
        Some(2),
        bridge_result_unwrap_or,
    );

    // ── core::option (cap: none — Value-level; map/and_then higher-order via
    //    call_value). Same qualified-binding rationale as core::result. ──
    define_native(global, "core::option::some", Some(1), bridge_option_some);
    define_native(global, "core::option::none", Some(0), bridge_option_none);
    define_native(global, "core::option::map", Some(2), bridge_option_map);
    define_native(
        global,
        "core::option::and_then",
        Some(2),
        bridge_option_and_then,
    );
    define_native(
        global,
        "core::option::unwrap_or",
        Some(2),
        bridge_option_unwrap_or,
    );

    // ── std::base64 (cap: none — dispatches garnet_stdlib::base64) ──
    define_native(global, "std::base64::encode", Some(1), bridge_base64_encode);
    define_native(global, "std::base64::decode", Some(1), bridge_base64_decode);

    // ── S22: remaining S17 Layer-1 runtime dispatch ──
    define_native(global, "std::json::parse", Some(1), bridge_json_parse);
    define_native(
        global,
        "std::json::stringify",
        Some(1),
        bridge_json_stringify,
    );
    define_native(global, "std::json::get", Some(2), bridge_json_get);
    define_native(global, "std::json::set", Some(3), bridge_json_set);

    define_native(global, "std::regex::compile", Some(1), bridge_regex_compile);
    define_native(global, "std::regex::match", Some(2), bridge_regex_match);
    define_native(
        global,
        "std::regex::find_all",
        Some(2),
        bridge_regex_find_all,
    );
    define_native(global, "std::regex::replace", Some(3), bridge_regex_replace);

    define_native(global, "std::uuid::new_v4", Some(0), bridge_uuid_new_v4);
    define_native(global, "std::uuid::new_v5", Some(2), bridge_uuid_new_v5);
    define_native(global, "std::uuid::new_v7", Some(0), bridge_uuid_new_v7);

    define_native(global, "std::env::get", Some(1), bridge_env_get);
    define_native(global, "std::env::set", Some(2), bridge_env_set);
    define_native(global, "std::env::vars", Some(0), bridge_env_vars);

    define_native(global, "std::process::spawn", Some(1), bridge_process_spawn);
    define_native(
        global,
        "std::process::spawn_args",
        Some(2),
        bridge_process_spawn_args,
    );
    define_native(
        global,
        "std::process::output",
        Some(2),
        bridge_process_output,
    );
    define_native(global, "std::process::wait", Some(1), bridge_process_wait);
    define_native(
        global,
        "std::process::exit_code",
        Some(1),
        bridge_process_exit_code,
    );

    define_native(global, "std::log::info", Some(1), bridge_log_info);
    define_native(global, "std::log::warn", Some(1), bridge_log_warn);
    define_native(global, "std::log::error", Some(1), bridge_log_error);
    define_native(global, "std::log::debug", Some(1), bridge_log_debug);
    define_native(global, "std::log::to_file", Some(3), bridge_log_to_file);

    define_native(global, "memory::working", Some(1), bridge_memory_working);
    define_native(global, "memory::episodic", Some(1), bridge_memory_episodic);
    define_native(global, "memory::semantic", Some(1), bridge_memory_semantic);
    define_native(
        global,
        "memory::procedural",
        Some(1),
        bridge_memory_procedural,
    );
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
    crate::eval::require_capability("fs", "fs::read_file")?;
    let path = expect_str("read_file", &args, 0)?;
    garnet_stdlib::fs::read_file(path)
        .map(Value::str)
        .map_err(|e| lift_std_error("read_file", e))
}

fn bridge_fs_write_file(args: Vec<Value>) -> Result<Value, RuntimeError> {
    crate::eval::require_capability("fs", "fs::write_file")?;
    let path = expect_str("write_file", &args, 0)?;
    let contents = expect_str("write_file", &args, 1)?;
    garnet_stdlib::fs::write_file(path, contents)
        .map(|_| Value::Nil)
        .map_err(|e| lift_std_error("write_file", e))
}

fn bridge_fs_read_bytes(args: Vec<Value>) -> Result<Value, RuntimeError> {
    crate::eval::require_capability("fs", "fs::read_bytes")?;
    let path = expect_str("read_bytes", &args, 0)?;
    garnet_stdlib::fs::read_bytes(path)
        .map(bytes_to_value)
        .map_err(|e| lift_std_error("read_bytes", e))
}

fn bridge_fs_write_bytes(args: Vec<Value>) -> Result<Value, RuntimeError> {
    crate::eval::require_capability("fs", "fs::write_bytes")?;
    let path = expect_str("write_bytes", &args, 0)?;
    let data = expect_byte_array("write_bytes", &args, 1)?;
    garnet_stdlib::fs::write_bytes(path, &data)
        .map(|_| Value::Nil)
        .map_err(|e| lift_std_error("write_bytes", e))
}

fn bridge_fs_list_dir(args: Vec<Value>) -> Result<Value, RuntimeError> {
    crate::eval::require_capability("fs", "fs::list_dir")?;
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
    crate::eval::require_capability("net", "net::tcp_connect")?;
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

// ════════════════════════════════════════════════════════════════════
// S21 — Layer-0/1 qualified-dispatch trampolines.
// ════════════════════════════════════════════════════════════════════

/// Accept an `Int` or `Float` argument as an `f64` (numbers widen to float).
fn expect_f64(prim: &str, args: &[Value], idx: usize) -> Result<f64, RuntimeError> {
    match args.get(idx) {
        Some(Value::Float(f)) => Ok(*f),
        Some(Value::Int(i)) => Ok(*i as f64),
        Some(other) => Err(RuntimeError::type_err(
            &format!("{prim}: number (Int|Float) arg at position {idx}"),
            other,
        )),
        None => Err(RuntimeError::msg(format!(
            "{prim}: missing argument at position {idx}"
        ))),
    }
}

/// Clone the arg at `idx`, or a missing-argument error.
fn expect_value(prim: &str, args: &[Value], idx: usize) -> Result<Value, RuntimeError> {
    args.get(idx)
        .cloned()
        .ok_or_else(|| RuntimeError::msg(format!("{prim}: missing argument at position {idx}")))
}

// ── core::math (dispatches garnet_stdlib::math) ──

fn bridge_math_abs(args: Vec<Value>) -> Result<Value, RuntimeError> {
    Ok(Value::Float(garnet_stdlib::math::abs(expect_f64(
        "core::math::abs",
        &args,
        0,
    )?)))
}

fn bridge_math_sqrt(args: Vec<Value>) -> Result<Value, RuntimeError> {
    let x = expect_f64("core::math::sqrt", &args, 0)?;
    garnet_stdlib::math::sqrt(x)
        .map(Value::Float)
        .map_err(|e| lift_std_error("core::math::sqrt", e))
}

fn bridge_math_pow(args: Vec<Value>) -> Result<Value, RuntimeError> {
    let base = expect_f64("core::math::pow", &args, 0)?;
    let exp = expect_f64("core::math::pow", &args, 1)?;
    Ok(Value::Float(garnet_stdlib::math::pow(base, exp)))
}

fn bridge_math_floor(args: Vec<Value>) -> Result<Value, RuntimeError> {
    Ok(Value::Float(garnet_stdlib::math::floor(expect_f64(
        "core::math::floor",
        &args,
        0,
    )?)))
}

fn bridge_math_ceil(args: Vec<Value>) -> Result<Value, RuntimeError> {
    Ok(Value::Float(garnet_stdlib::math::ceil(expect_f64(
        "core::math::ceil",
        &args,
        0,
    )?)))
}

fn bridge_math_round(args: Vec<Value>) -> Result<Value, RuntimeError> {
    Ok(Value::Float(garnet_stdlib::math::round(expect_f64(
        "core::math::round",
        &args,
        0,
    )?)))
}

// ── core::cmp (Value-level; garnet_stdlib::cmp is the Rust reference) ──

fn cmp_pair(
    prim: &str,
    args: &[Value],
) -> Result<(Value, Value, std::cmp::Ordering), RuntimeError> {
    let a = expect_value(prim, args, 0)?;
    let b = expect_value(prim, args, 1)?;
    match a.partial_compare(&b) {
        Some(ord) => Ok((a, b, ord)),
        None => Err(RuntimeError::msg(format!(
            "{prim}: values not comparable ({} vs {})",
            a.type_name(),
            b.type_name()
        ))),
    }
}

fn bridge_cmp_min(args: Vec<Value>) -> Result<Value, RuntimeError> {
    let (a, b, ord) = cmp_pair("core::cmp::min", &args)?;
    Ok(if ord == std::cmp::Ordering::Greater {
        b
    } else {
        a
    })
}

fn bridge_cmp_max(args: Vec<Value>) -> Result<Value, RuntimeError> {
    let (a, b, ord) = cmp_pair("core::cmp::max", &args)?;
    Ok(if ord == std::cmp::Ordering::Less {
        b
    } else {
        a
    })
}

fn bridge_cmp_ordering(args: Vec<Value>) -> Result<Value, RuntimeError> {
    let (_, _, ord) = cmp_pair("core::cmp::ordering", &args)?;
    Ok(Value::Int(match ord {
        std::cmp::Ordering::Less => -1,
        std::cmp::Ordering::Equal => 0,
        std::cmp::Ordering::Greater => 1,
    }))
}

fn bridge_cmp_clamp(args: Vec<Value>) -> Result<Value, RuntimeError> {
    let v = expect_value("core::cmp::clamp", &args, 0)?;
    let lo = expect_value("core::cmp::clamp", &args, 1)?;
    let hi = expect_value("core::cmp::clamp", &args, 2)?;
    let below = v.partial_compare(&lo).ok_or_else(|| {
        RuntimeError::msg("core::cmp::clamp: value and lower bound not comparable".to_string())
    })?;
    if below == std::cmp::Ordering::Less {
        return Ok(lo);
    }
    let above = v.partial_compare(&hi).ok_or_else(|| {
        RuntimeError::msg("core::cmp::clamp: value and upper bound not comparable".to_string())
    })?;
    if above == std::cmp::Ordering::Greater {
        return Ok(hi);
    }
    Ok(v)
}

// ── core::iter (Value-level; higher-order via call_value) ──

fn bridge_iter_map(args: Vec<Value>) -> Result<Value, RuntimeError> {
    let items = expect_array_clone("core::iter::map", &args, 0)?;
    let f = expect_value("core::iter::map", &args, 1)?;
    let mut out = Vec::with_capacity(items.len());
    for item in items {
        out.push(crate::eval::call_value(&f, vec![item])?);
    }
    Ok(Value::array(out))
}

fn bridge_iter_filter(args: Vec<Value>) -> Result<Value, RuntimeError> {
    let items = expect_array_clone("core::iter::filter", &args, 0)?;
    let pred = expect_value("core::iter::filter", &args, 1)?;
    let mut out = Vec::new();
    for item in items {
        if crate::eval::call_value(&pred, vec![item.clone()])?.truthy() {
            out.push(item);
        }
    }
    Ok(Value::array(out))
}

fn bridge_iter_fold(args: Vec<Value>) -> Result<Value, RuntimeError> {
    let items = expect_array_clone("core::iter::fold", &args, 0)?;
    let mut acc = expect_value("core::iter::fold", &args, 1)?;
    let f = expect_value("core::iter::fold", &args, 2)?;
    for item in items {
        acc = crate::eval::call_value(&f, vec![acc, item])?;
    }
    Ok(acc)
}

fn bridge_iter_take(args: Vec<Value>) -> Result<Value, RuntimeError> {
    let items = expect_array_clone("core::iter::take", &args, 0)?;
    let n = expect_usize("core::iter::take", &args, 1)?;
    Ok(Value::array(items.into_iter().take(n).collect()))
}

fn bridge_iter_drop(args: Vec<Value>) -> Result<Value, RuntimeError> {
    let items = expect_array_clone("core::iter::drop", &args, 0)?;
    let n = expect_usize("core::iter::drop", &args, 1)?;
    Ok(Value::array(items.into_iter().skip(n).collect()))
}

fn bridge_iter_enumerate(args: Vec<Value>) -> Result<Value, RuntimeError> {
    let items = expect_array_clone("core::iter::enumerate", &args, 0)?;
    Ok(Value::array(
        items
            .into_iter()
            .enumerate()
            .map(|(i, v)| Value::array(vec![Value::Int(i as i64), v]))
            .collect(),
    ))
}

fn bridge_iter_zip(args: Vec<Value>) -> Result<Value, RuntimeError> {
    let a = expect_array_clone("core::iter::zip", &args, 0)?;
    let b = expect_array_clone("core::iter::zip", &args, 1)?;
    // `zip` stops at the shorter sequence; each element is a 2-element pair array.
    Ok(Value::array(
        a.into_iter()
            .zip(b)
            .map(|(x, y)| Value::array(vec![x, y]))
            .collect(),
    ))
}

fn bridge_iter_chain(args: Vec<Value>) -> Result<Value, RuntimeError> {
    let mut a = expect_array_clone("core::iter::chain", &args, 0)?;
    let b = expect_array_clone("core::iter::chain", &args, 1)?;
    a.extend(b);
    Ok(Value::array(a))
}

fn bridge_iter_collect(args: Vec<Value>) -> Result<Value, RuntimeError> {
    // Materialize a sequence into an owned Array. A `Range` is expanded to its
    // integers (exclusive or inclusive); an Array is materialized (cloned). Other
    // values are a type error — there is no other lazy sequence in managed mode.
    match args.first() {
        Some(Value::Array(items)) => Ok(Value::array(items.borrow().clone())),
        Some(Value::Range {
            start,
            end,
            inclusive,
        }) => {
            let out: Vec<Value> = if *inclusive {
                (*start..=*end).map(Value::Int).collect()
            } else {
                (*start..*end).map(Value::Int).collect()
            };
            Ok(Value::array(out))
        }
        Some(other) => Err(RuntimeError::type_err(
            "core::iter::collect: Array or Range arg at position 0",
            other,
        )),
        None => Err(RuntimeError::msg("core::iter::collect: missing argument")),
    }
}

// ── core::result (Value-level; Ok/Err are `Value::Variant` with path ["Result"],
//    matching the prelude's `ok`/`err` builders so pattern-matching and `?` agree) ──

/// Build a `Result::Ok(value)` Variant identical to the prelude `ok` builder.
fn result_ok(value: Value) -> Value {
    Value::Variant {
        path: Rc::new(vec!["Result".to_string()]),
        variant: Rc::new("Ok".to_string()),
        fields: Rc::new(vec![value]),
    }
}

/// Build a `Result::Err(value)` Variant identical to the prelude `err` builder.
fn result_err(value: Value) -> Value {
    Value::Variant {
        path: Rc::new(vec!["Result".to_string()]),
        variant: Rc::new("Err".to_string()),
        fields: Rc::new(vec![value]),
    }
}

enum ResultView {
    Ok(Value),
    Err(Value),
}

/// Classify the argument at `idx` as a `Result` Variant, or raise a type error.
fn expect_result(prim: &str, args: &[Value], idx: usize) -> Result<ResultView, RuntimeError> {
    match args.get(idx) {
        Some(Value::Variant {
            path,
            variant,
            fields,
        }) if path.len() == 1 && path[0] == "Result" => {
            let inner = fields.first().cloned().unwrap_or(Value::Nil);
            match variant.as_str() {
                "Ok" => Ok(ResultView::Ok(inner)),
                "Err" => Ok(ResultView::Err(inner)),
                other => Err(RuntimeError::msg(format!(
                    "{prim}: unknown Result variant `{other}`"
                ))),
            }
        }
        Some(other) => Err(RuntimeError::type_err(
            &format!("{prim}: Result arg at position {idx}"),
            other,
        )),
        None => Err(RuntimeError::msg(format!(
            "{prim}: missing Result argument"
        ))),
    }
}

fn bridge_result_ok(args: Vec<Value>) -> Result<Value, RuntimeError> {
    Ok(result_ok(args.into_iter().next().unwrap_or(Value::Nil)))
}

fn bridge_result_err(args: Vec<Value>) -> Result<Value, RuntimeError> {
    Ok(result_err(args.into_iter().next().unwrap_or(Value::Nil)))
}

fn bridge_result_map(args: Vec<Value>) -> Result<Value, RuntimeError> {
    let f = expect_value("core::result::map", &args, 1)?;
    match expect_result("core::result::map", &args, 0)? {
        ResultView::Ok(v) => Ok(result_ok(crate::eval::call_value(&f, vec![v])?)),
        ResultView::Err(e) => Ok(result_err(e)),
    }
}

fn bridge_result_and_then(args: Vec<Value>) -> Result<Value, RuntimeError> {
    let f = expect_value("core::result::and_then", &args, 1)?;
    match expect_result("core::result::and_then", &args, 0)? {
        // `f` is expected to return a Result; pass its value through unchanged
        // (Garnet is dynamically typed, so we trust the callee like Rust's `?`).
        ResultView::Ok(v) => crate::eval::call_value(&f, vec![v]),
        ResultView::Err(e) => Ok(result_err(e)),
    }
}

fn bridge_result_or_else(args: Vec<Value>) -> Result<Value, RuntimeError> {
    let f = expect_value("core::result::or_else", &args, 1)?;
    match expect_result("core::result::or_else", &args, 0)? {
        ResultView::Ok(v) => Ok(result_ok(v)),
        ResultView::Err(e) => crate::eval::call_value(&f, vec![e]),
    }
}

fn bridge_result_unwrap_or(args: Vec<Value>) -> Result<Value, RuntimeError> {
    let default = expect_value("core::result::unwrap_or", &args, 1)?;
    match expect_result("core::result::unwrap_or", &args, 0)? {
        ResultView::Ok(v) => Ok(v),
        ResultView::Err(_) => Ok(default),
    }
}

// ── core::option (Value-level; Some/None are `Value::Variant` with path
//    ["Option"], matching the prelude's `some`/`none` builders) ──

/// Build an `Option::Some(value)` Variant identical to the prelude `some` builder.
fn option_some(value: Value) -> Value {
    Value::Variant {
        path: Rc::new(vec!["Option".to_string()]),
        variant: Rc::new("Some".to_string()),
        fields: Rc::new(vec![value]),
    }
}

/// Build the `Option::None` Variant identical to the prelude `none` builder.
fn option_none() -> Value {
    Value::Variant {
        path: Rc::new(vec!["Option".to_string()]),
        variant: Rc::new("None".to_string()),
        fields: Rc::new(vec![]),
    }
}

enum OptionView {
    Some(Value),
    None,
}

/// Classify the argument at `idx` as an `Option` Variant, or raise a type error.
fn expect_option(prim: &str, args: &[Value], idx: usize) -> Result<OptionView, RuntimeError> {
    match args.get(idx) {
        Some(Value::Variant {
            path,
            variant,
            fields,
        }) if path.len() == 1 && path[0] == "Option" => match variant.as_str() {
            "Some" => Ok(OptionView::Some(
                fields.first().cloned().unwrap_or(Value::Nil),
            )),
            "None" => Ok(OptionView::None),
            other => Err(RuntimeError::msg(format!(
                "{prim}: unknown Option variant `{other}`"
            ))),
        },
        Some(other) => Err(RuntimeError::type_err(
            &format!("{prim}: Option arg at position {idx}"),
            other,
        )),
        None => Err(RuntimeError::msg(format!(
            "{prim}: missing Option argument"
        ))),
    }
}

fn bridge_option_some(args: Vec<Value>) -> Result<Value, RuntimeError> {
    Ok(option_some(args.into_iter().next().unwrap_or(Value::Nil)))
}

fn bridge_option_none(_args: Vec<Value>) -> Result<Value, RuntimeError> {
    Ok(option_none())
}

fn bridge_option_map(args: Vec<Value>) -> Result<Value, RuntimeError> {
    let f = expect_value("core::option::map", &args, 1)?;
    match expect_option("core::option::map", &args, 0)? {
        OptionView::Some(v) => Ok(option_some(crate::eval::call_value(&f, vec![v])?)),
        OptionView::None => Ok(option_none()),
    }
}

fn bridge_option_and_then(args: Vec<Value>) -> Result<Value, RuntimeError> {
    let f = expect_value("core::option::and_then", &args, 1)?;
    match expect_option("core::option::and_then", &args, 0)? {
        // `f` is expected to return an Option; pass it through unchanged.
        OptionView::Some(v) => crate::eval::call_value(&f, vec![v]),
        OptionView::None => Ok(option_none()),
    }
}

fn bridge_option_unwrap_or(args: Vec<Value>) -> Result<Value, RuntimeError> {
    let default = expect_value("core::option::unwrap_or", &args, 1)?;
    match expect_option("core::option::unwrap_or", &args, 0)? {
        OptionView::Some(v) => Ok(v),
        OptionView::None => Ok(default),
    }
}

// ── std::base64 (dispatches garnet_stdlib::base64) ──

fn bridge_base64_encode(args: Vec<Value>) -> Result<Value, RuntimeError> {
    // Encode the UTF-8 bytes of a string into a standard base64 string.
    let s = expect_str("std::base64::encode", &args, 0)?;
    Ok(Value::str(garnet_stdlib::base64::encode(s.as_bytes())))
}

fn bridge_base64_decode(args: Vec<Value>) -> Result<Value, RuntimeError> {
    // Decode a base64 string into a byte array (Array of Int 0..=255).
    let s = expect_str("std::base64::decode", &args, 0)?;
    garnet_stdlib::base64::decode(s)
        .map(bytes_to_value)
        .map_err(|e| lift_std_error("std::base64::decode", e))
}

// ── S22: std::json (serde_json <-> managed Value) ──

fn json_to_value(value: JsonValue) -> Result<Value, RuntimeError> {
    match value {
        JsonValue::Null => Ok(Value::Nil),
        JsonValue::Bool(b) => Ok(Value::Bool(b)),
        JsonValue::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(Value::Int(i))
            } else if let Some(u) = n.as_u64() {
                if let Ok(i) = i64::try_from(u) {
                    Ok(Value::Int(i))
                } else {
                    Ok(Value::Float(u as f64))
                }
            } else if let Some(f) = n.as_f64() {
                Ok(Value::Float(f))
            } else {
                Err(RuntimeError::msg("std::json: unsupported number"))
            }
        }
        JsonValue::String(s) => Ok(Value::str(s)),
        JsonValue::Array(items) => Ok(Value::array(
            items
                .into_iter()
                .map(json_to_value)
                .collect::<Result<Vec<_>, _>>()?,
        )),
        JsonValue::Object(map) => Ok(Value::map(
            map.into_iter()
                .map(|(k, v)| json_to_value(v).map(|v| (k, v)))
                .collect::<Result<Vec<_>, _>>()?,
        )),
    }
}

fn value_to_json(value: &Value) -> Result<JsonValue, RuntimeError> {
    match value {
        Value::Nil => Ok(JsonValue::Null),
        Value::Bool(b) => Ok(JsonValue::Bool(*b)),
        Value::Int(i) => Ok(JsonValue::Number((*i).into())),
        Value::Float(f) => serde_json::Number::from_f64(*f)
            .map(JsonValue::Number)
            .ok_or_else(|| RuntimeError::msg("std::json: float must be finite")),
        Value::Str(s) => Ok(JsonValue::String(s.to_string())),
        Value::Array(items) => Ok(JsonValue::Array(
            items
                .borrow()
                .iter()
                .map(value_to_json)
                .collect::<Result<Vec<_>, _>>()?,
        )),
        Value::Tuple(items) => Ok(JsonValue::Array(
            items
                .iter()
                .map(value_to_json)
                .collect::<Result<Vec<_>, _>>()?,
        )),
        Value::Map(map) => {
            let mut out = serde_json::Map::new();
            for (key, value) in map.borrow().iter() {
                out.insert(key.clone(), value_to_json(value)?);
            }
            Ok(JsonValue::Object(out))
        }
        other => Err(RuntimeError::msg(format!(
            "std::json: cannot serialize {}",
            other.type_name()
        ))),
    }
}

fn bridge_json_parse(args: Vec<Value>) -> Result<Value, RuntimeError> {
    let input = expect_str("std::json::parse", &args, 0)?;
    garnet_stdlib::json::parse(input)
        .map_err(|e| lift_std_error("std::json::parse", e))
        .and_then(json_to_value)
}

fn bridge_json_stringify(args: Vec<Value>) -> Result<Value, RuntimeError> {
    let value = expect_value("std::json::stringify", &args, 0)?;
    let json = value_to_json(&value)?;
    Ok(Value::str(garnet_stdlib::json::stringify(&json)))
}

fn bridge_json_get(args: Vec<Value>) -> Result<Value, RuntimeError> {
    let value = expect_value("std::json::get", &args, 0)?;
    let key = expect_str("std::json::get", &args, 1)?;
    let json = value_to_json(&value)?;
    match garnet_stdlib::json::get(&json, key) {
        Some(child) => json_to_value(child),
        None => Ok(Value::Nil),
    }
}

fn bridge_json_set(args: Vec<Value>) -> Result<Value, RuntimeError> {
    let target = expect_value("std::json::set", &args, 0)?;
    let key = expect_str("std::json::set", &args, 1)?;
    let value = expect_value("std::json::set", &args, 2)?;
    let target = value_to_json(&target)?;
    let value = value_to_json(&value)?;
    garnet_stdlib::json::set(&target, key, value)
        .map_err(|e| lift_std_error("std::json::set", e))
        .and_then(json_to_value)
}

// ── S22: std::regex ──

fn bridge_regex_compile(args: Vec<Value>) -> Result<Value, RuntimeError> {
    let pattern = expect_str("std::regex::compile", &args, 0)?;
    garnet_stdlib::regex::compile(pattern)
        .map(|_| Value::Nil)
        .map_err(|e| lift_std_error("std::regex::compile", e))
}

fn bridge_regex_match(args: Vec<Value>) -> Result<Value, RuntimeError> {
    let pattern = expect_str("std::regex::match", &args, 0)?;
    let input = expect_str("std::regex::match", &args, 1)?;
    garnet_stdlib::regex::is_match(pattern, input)
        .map(Value::Bool)
        .map_err(|e| lift_std_error("std::regex::match", e))
}

fn bridge_regex_find_all(args: Vec<Value>) -> Result<Value, RuntimeError> {
    let pattern = expect_str("std::regex::find_all", &args, 0)?;
    let input = expect_str("std::regex::find_all", &args, 1)?;
    garnet_stdlib::regex::find_all(pattern, input)
        .map(|items| Value::array(items.into_iter().map(Value::str).collect()))
        .map_err(|e| lift_std_error("std::regex::find_all", e))
}

fn bridge_regex_replace(args: Vec<Value>) -> Result<Value, RuntimeError> {
    let pattern = expect_str("std::regex::replace", &args, 0)?;
    let input = expect_str("std::regex::replace", &args, 1)?;
    let replacement = expect_str("std::regex::replace", &args, 2)?;
    garnet_stdlib::regex::replace(pattern, input, replacement)
        .map(Value::str)
        .map_err(|e| lift_std_error("std::regex::replace", e))
}

// ── S22: std::uuid ──

fn parse_uuid_namespace(input: &str) -> Result<[u8; 16], RuntimeError> {
    let compact: String = input.chars().filter(|c| *c != '-').collect();
    if compact.len() != 32 || !compact.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(RuntimeError::msg(format!(
            "std::uuid::new_v5: namespace must be 32 hex chars or canonical UUID, got {input:?}"
        )));
    }
    let mut bytes = [0u8; 16];
    for (idx, byte) in bytes.iter_mut().enumerate() {
        let start = idx * 2;
        *byte = u8::from_str_radix(&compact[start..start + 2], 16).map_err(|e| {
            RuntimeError::msg(format!("std::uuid::new_v5: invalid namespace byte: {e}"))
        })?;
    }
    Ok(bytes)
}

fn bridge_uuid_new_v4(_args: Vec<Value>) -> Result<Value, RuntimeError> {
    Ok(Value::str(garnet_stdlib::uuid::new_v4()))
}

fn bridge_uuid_new_v5(args: Vec<Value>) -> Result<Value, RuntimeError> {
    let namespace = parse_uuid_namespace(expect_str("std::uuid::new_v5", &args, 0)?)?;
    let name = expect_str("std::uuid::new_v5", &args, 1)?;
    Ok(Value::str(garnet_stdlib::uuid::new_v5(&namespace, name)))
}

fn bridge_uuid_new_v7(_args: Vec<Value>) -> Result<Value, RuntimeError> {
    Ok(Value::str(garnet_stdlib::uuid::new_v7()))
}

// ── S22: std::env and std::process ──

fn bridge_env_get(args: Vec<Value>) -> Result<Value, RuntimeError> {
    crate::eval::require_capability("env", "std::env::get")?;
    let key = expect_str("std::env::get", &args, 0)?;
    validate_env_key("std::env::get", key)?;
    match std::env::var_os(key) {
        Some(value) => value
            .into_string()
            .map(Value::str)
            .map_err(|_| RuntimeError::msg("std::env::get: value is not valid Unicode")),
        None => Ok(Value::Nil),
    }
}

fn bridge_env_set(args: Vec<Value>) -> Result<Value, RuntimeError> {
    crate::eval::require_capability("env", "std::env::set")?;
    let key = expect_str("std::env::set", &args, 0)?;
    let value = expect_str("std::env::set", &args, 1)?;
    validate_env_key("std::env::set", key)?;
    validate_env_value("std::env::set", value)?;
    garnet_stdlib::env::set(key, value);
    Ok(Value::Nil)
}

fn bridge_env_vars(_args: Vec<Value>) -> Result<Value, RuntimeError> {
    crate::eval::require_capability("env", "std::env::vars")?;
    let mut vars = Vec::new();
    for (key, value) in std::env::vars_os() {
        let key = key
            .into_string()
            .map_err(|_| RuntimeError::msg("std::env::vars: key is not valid Unicode"))?;
        let value = value
            .into_string()
            .map_err(|_| RuntimeError::msg("std::env::vars: value is not valid Unicode"))?;
        vars.push(Value::array(vec![Value::str(key), Value::str(value)]));
    }
    Ok(Value::array(vars))
}

fn validate_env_key(prim: &str, key: &str) -> Result<(), RuntimeError> {
    if key.is_empty() {
        return Err(RuntimeError::msg(format!("{prim}: key must not be empty")));
    }
    if key.bytes().any(|b| b == b'=' || b == 0) {
        return Err(RuntimeError::msg(format!(
            "{prim}: key must not contain '=' or NUL"
        )));
    }
    Ok(())
}

fn validate_env_value(prim: &str, value: &str) -> Result<(), RuntimeError> {
    if value.bytes().any(|b| b == 0) {
        return Err(RuntimeError::msg(format!(
            "{prim}: value must not contain NUL"
        )));
    }
    Ok(())
}

fn bridge_process_spawn(args: Vec<Value>) -> Result<Value, RuntimeError> {
    crate::eval::require_capability("proc", "std::process::spawn")?;
    crate::eval::require_entry_capability("proc", "std::process::spawn")?;
    let cmdline = expect_str("std::process::spawn", &args, 0)?;
    garnet_stdlib::process::spawn(cmdline)
        .map(|proc| Value::Process(Rc::new(RefCell::new(Some(proc)))))
        .map_err(|e| lift_std_error("std::process::spawn", e))
}

fn bridge_process_wait(args: Vec<Value>) -> Result<Value, RuntimeError> {
    crate::eval::require_capability("proc", "std::process::wait")?;
    let process = match args.first() {
        Some(Value::Process(process)) => Rc::clone(process),
        Some(other) => {
            return Err(RuntimeError::type_err(
                "std::process::wait: Process arg at position 0",
                other,
            ))
        }
        None => return Err(RuntimeError::msg("std::process::wait: missing process")),
    };
    let proc = process
        .borrow_mut()
        .take()
        .ok_or_else(|| RuntimeError::msg("std::process::wait: process already waited"))?;
    garnet_stdlib::process::wait(proc)
        .map(Value::ProcessStatus)
        .map_err(|e| lift_std_error("std::process::wait", e))
}

fn bridge_process_exit_code(args: Vec<Value>) -> Result<Value, RuntimeError> {
    crate::eval::require_capability("proc", "std::process::exit_code")?;
    match args.first() {
        Some(Value::ProcessStatus(status)) => Ok(garnet_stdlib::process::exit_code(status)
            .map(|code| Value::Int(code as i64))
            .unwrap_or(Value::Nil)),
        Some(other) => Err(RuntimeError::type_err(
            "std::process::exit_code: ProcessStatus arg at position 0",
            other,
        )),
        None => Err(RuntimeError::msg(
            "std::process::exit_code: missing process status",
        )),
    }
}

/// Unpack a `Value::Array` of `Value::Str` into an owned `Vec<String>` for an
/// explicit argv. Every element must be a String.
fn expect_string_array(
    prim: &str,
    args: &[Value],
    idx: usize,
) -> Result<Vec<String>, RuntimeError> {
    let items = expect_array_clone(prim, args, idx)?;
    items
        .into_iter()
        .enumerate()
        .map(|(i, value)| match value {
            Value::Str(s) => Ok((*s).clone()),
            other => Err(RuntimeError::type_err(
                &format!("{prim}: argv element {i} must be a String"),
                &other,
            )),
        })
        .collect()
}

fn bridge_process_spawn_args(args: Vec<Value>) -> Result<Value, RuntimeError> {
    crate::eval::require_capability("proc", "std::process::spawn_args")?;
    crate::eval::require_entry_capability("proc", "std::process::spawn_args")?;
    let program = expect_str("std::process::spawn_args", &args, 0)?.to_string();
    let argv = expect_string_array("std::process::spawn_args", &args, 1)?;
    garnet_stdlib::process::spawn_args(&program, &argv)
        .map(|proc| Value::Process(Rc::new(RefCell::new(Some(proc)))))
        .map_err(|e| lift_std_error("std::process::spawn_args", e))
}

fn bridge_process_output(args: Vec<Value>) -> Result<Value, RuntimeError> {
    crate::eval::require_capability("proc", "std::process::output")?;
    crate::eval::require_entry_capability("proc", "std::process::output")?;
    let program = expect_str("std::process::output", &args, 0)?.to_string();
    let argv = expect_string_array("std::process::output", &args, 1)?;
    let out = garnet_stdlib::process::output(&program, &argv)
        .map_err(|e| lift_std_error("std::process::output", e))?;
    let code = out
        .code()
        .map(|c| Value::Int(c as i64))
        .unwrap_or(Value::Nil);
    Ok(Value::map(vec![
        ("code".to_string(), code),
        ("stdout".to_string(), Value::str(out.stdout())),
        ("stderr".to_string(), Value::str(out.stderr())),
    ]))
}

// ── S22: std::log formatting ──

fn bridge_log_info(args: Vec<Value>) -> Result<Value, RuntimeError> {
    let message = expect_str("std::log::info", &args, 0)?;
    Ok(Value::str(garnet_stdlib::log::info(message)))
}

fn bridge_log_warn(args: Vec<Value>) -> Result<Value, RuntimeError> {
    let message = expect_str("std::log::warn", &args, 0)?;
    Ok(Value::str(garnet_stdlib::log::warn(message)))
}

fn bridge_log_error(args: Vec<Value>) -> Result<Value, RuntimeError> {
    let message = expect_str("std::log::error", &args, 0)?;
    Ok(Value::str(garnet_stdlib::log::error(message)))
}

fn bridge_log_debug(args: Vec<Value>) -> Result<Value, RuntimeError> {
    let message = expect_str("std::log::debug", &args, 0)?;
    Ok(Value::str(garnet_stdlib::log::debug(message)))
}

// ── S24: std::log file sink (cap: fs) ──

fn bridge_log_to_file(args: Vec<Value>) -> Result<Value, RuntimeError> {
    crate::eval::require_capability("fs", "std::log::to_file")?;
    let path = expect_str("std::log::to_file", &args, 0)?.to_string();
    let level = expect_str("std::log::to_file", &args, 1)?.to_string();
    let message = expect_str("std::log::to_file", &args, 2)?;
    garnet_stdlib::log::to_file(&path, &level, message)
        .map(Value::str)
        .map_err(|e| lift_std_error("std::log::to_file", e))
}

// ── S22: memory:: constructors (live Mnemos handles) ──

fn memory_store(kind: MemoryKind, name: String) -> Value {
    Value::MemoryStore {
        kind,
        name,
        backend: MemoryBackend::for_kind(kind),
    }
}

fn bridge_memory_kind(
    prim: &str,
    kind: MemoryKind,
    args: Vec<Value>,
) -> Result<Value, RuntimeError> {
    let name = expect_str(prim, &args, 0)?;
    Ok(memory_store(kind, name.to_string()))
}

fn bridge_memory_working(args: Vec<Value>) -> Result<Value, RuntimeError> {
    bridge_memory_kind("memory::working", MemoryKind::Working, args)
}

fn bridge_memory_episodic(args: Vec<Value>) -> Result<Value, RuntimeError> {
    bridge_memory_kind("memory::episodic", MemoryKind::Episodic, args)
}

fn bridge_memory_semantic(args: Vec<Value>) -> Result<Value, RuntimeError> {
    bridge_memory_kind("memory::semantic", MemoryKind::Semantic, args)
}

fn bridge_memory_procedural(args: Vec<Value>) -> Result<Value, RuntimeError> {
    bridge_memory_kind("memory::procedural", MemoryKind::Procedural, args)
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

    // ── S21 qualified-dispatch tests ──

    /// A test-only native that doubles an Int — stands in for a Garnet callable
    /// passed to the higher-order iterator combinators.
    fn double_fn() -> Value {
        fn double(args: Vec<Value>) -> Result<Value, RuntimeError> {
            match args.first() {
                Some(Value::Int(i)) => Ok(Value::Int(i * 2)),
                other => Err(RuntimeError::msg(format!(
                    "double: int expected, got {other:?}"
                ))),
            }
        }
        Value::NativeFn(Rc::new(NativeFnValue {
            name: "double",
            arity: Some(1),
            ptr: double,
        }))
    }

    fn call(env: &Env, name: &str, args: Vec<Value>) -> Result<Value, RuntimeError> {
        let f = env.get(name).unwrap_or_else(|| panic!("{name} not bound"));
        crate::eval::call_value(&f, args)
    }

    #[test]
    fn s21_math_sqrt_pow_round() {
        let env = make_env();
        assert!(
            matches!(call(&env, "core::math::sqrt", vec![Value::Float(16.0)]).unwrap(), Value::Float(f) if (f - 4.0).abs() < 1e-12)
        );
        assert!(
            matches!(call(&env, "core::math::pow", vec![Value::Int(2), Value::Int(10)]).unwrap(), Value::Float(f) if (f - 1024.0).abs() < 1e-9)
        );
        assert!(
            matches!(call(&env, "core::math::round", vec![Value::Float(2.5)]).unwrap(), Value::Float(f) if (f - 3.0).abs() < 1e-12)
        );
    }

    #[test]
    fn s21_math_sqrt_negative_raises() {
        let env = make_env();
        match call(&env, "core::math::sqrt", vec![Value::Float(-1.0)]) {
            Err(RuntimeError::Raised(v)) => assert!(v.display().contains("sqrt")),
            other => panic!("expected Raised, got {other:?}"),
        }
    }

    #[test]
    fn s21_cmp_min_ordering_clamp_preserve_type() {
        let env = make_env();
        assert!(matches!(
            call(&env, "core::cmp::min", vec![Value::Int(3), Value::Int(7)]).unwrap(),
            Value::Int(3)
        ));
        assert!(matches!(
            call(
                &env,
                "core::cmp::ordering",
                vec![Value::Int(2), Value::Int(5)]
            )
            .unwrap(),
            Value::Int(-1)
        ));
        assert!(matches!(
            call(
                &env,
                "core::cmp::clamp",
                vec![Value::Int(99), Value::Int(0), Value::Int(10)]
            )
            .unwrap(),
            Value::Int(10)
        ));
    }

    #[test]
    fn s21_iter_map_applies_callable() {
        let env = make_env();
        let arr = Value::array(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
        match call(&env, "core::iter::map", vec![arr, double_fn()]).unwrap() {
            Value::Array(a) => {
                let got: Vec<i64> = a
                    .borrow()
                    .iter()
                    .filter_map(|v| {
                        if let Value::Int(i) = v {
                            Some(*i)
                        } else {
                            None
                        }
                    })
                    .collect();
                assert_eq!(got, vec![2, 4, 6]);
            }
            other => panic!("expected Array, got {other:?}"),
        }
    }

    #[test]
    fn s21_iter_take_and_enumerate() {
        let env = make_env();
        let arr = Value::array(vec![
            Value::Int(10),
            Value::Int(20),
            Value::Int(30),
            Value::Int(40),
        ]);
        match call(&env, "core::iter::take", vec![arr.clone(), Value::Int(2)]).unwrap() {
            Value::Array(a) => assert_eq!(a.borrow().len(), 2),
            other => panic!("expected Array, got {other:?}"),
        }
        match call(&env, "core::iter::enumerate", vec![arr]).unwrap() {
            Value::Array(a) => match &a.borrow()[0] {
                Value::Array(pair) => {
                    let pair = pair.borrow();
                    assert!(matches!(pair[0], Value::Int(0)));
                    assert!(matches!(pair[1], Value::Int(10)));
                }
                other => panic!("expected pair Array, got {other:?}"),
            },
            other => panic!("expected Array, got {other:?}"),
        }
    }

    #[test]
    fn s21_base64_roundtrip() {
        let env = make_env();
        match call(&env, "std::base64::encode", vec![Value::str("hi")]).unwrap() {
            Value::Str(s) => assert_eq!(&*s, "aGk="),
            other => panic!("expected Str, got {other:?}"),
        }
        match call(&env, "std::base64::decode", vec![Value::str("aGk=")]).unwrap() {
            Value::Array(a) => {
                let bytes: Vec<i64> = a
                    .borrow()
                    .iter()
                    .filter_map(|v| {
                        if let Value::Int(i) = v {
                            Some(*i)
                        } else {
                            None
                        }
                    })
                    .collect();
                assert_eq!(bytes, vec![104, 105]); // 'h', 'i'
            }
            other => panic!("expected Array, got {other:?}"),
        }
    }

    #[test]
    fn s21_all_qualified_names_bound() {
        let env = make_env();
        for n in [
            "core::math::abs",
            "core::math::sqrt",
            "core::math::pow",
            "core::math::floor",
            "core::math::ceil",
            "core::math::round",
            "core::cmp::min",
            "core::cmp::max",
            "core::cmp::clamp",
            "core::cmp::ordering",
            "core::iter::map",
            "core::iter::filter",
            "core::iter::fold",
            "core::iter::take",
            "core::iter::drop",
            "core::iter::enumerate",
            "std::base64::encode",
            "std::base64::decode",
        ] {
            assert!(env.get(n).is_some(), "qualified prim `{n}` not bound");
        }
    }

    #[test]
    fn s22_all_qualified_names_bound() {
        let env = make_env();
        for n in [
            "std::json::parse",
            "std::json::stringify",
            "std::json::get",
            "std::json::set",
            "std::regex::compile",
            "std::regex::match",
            "std::regex::find_all",
            "std::regex::replace",
            "std::uuid::new_v4",
            "std::uuid::new_v5",
            "std::uuid::new_v7",
            "std::env::get",
            "std::env::set",
            "std::env::vars",
            "std::process::spawn",
            "std::process::wait",
            "std::process::exit_code",
            "std::log::info",
            "std::log::warn",
            "std::log::error",
            "std::log::debug",
            "memory::working",
            "memory::episodic",
            "memory::semantic",
            "memory::procedural",
        ] {
            assert!(env.get(n).is_some(), "S22 prim `{n}` not bound");
        }
    }

    #[test]
    fn s22_json_and_regex_error_paths_are_raised() {
        let env = make_env();
        match call(&env, "std::json::parse", vec![Value::str("{bad json}")]) {
            Err(RuntimeError::Raised(v)) => assert!(v.display().contains("std::json::parse")),
            other => panic!("expected JSON parse Raised error, got {other:?}"),
        }
        match call(&env, "std::regex::compile", vec![Value::str("(unclosed")]) {
            Err(RuntimeError::Raised(v)) => assert!(v.display().contains("std::regex::compile")),
            other => panic!("expected regex compile Raised error, got {other:?}"),
        }
    }

    #[test]
    fn s22_env_invalid_inputs_surface_runtime_errors() {
        let env = make_env();
        match call(&env, "std::env::get", vec![Value::str("BAD=KEY")]) {
            Err(RuntimeError::Message(msg)) => assert!(msg.contains("must not contain")),
            other => panic!("expected invalid key Message error, got {other:?}"),
        }
        match call(&env, "std::env::set", vec![Value::str(""), Value::str("x")]) {
            Err(RuntimeError::Message(msg)) => assert!(msg.contains("must not be empty")),
            other => panic!("expected empty key Message error, got {other:?}"),
        }
        match call(
            &env,
            "std::env::set",
            vec![Value::str("GARNET_S22_BAD_VALUE"), Value::str("bad\0value")],
        ) {
            Err(RuntimeError::Message(msg)) => assert!(msg.contains("value must not contain")),
            other => panic!("expected invalid value Message error, got {other:?}"),
        }
    }

    #[test]
    fn s22_process_wait_consumes_handle_once() {
        let env = make_env();
        let cmd = if cfg!(windows) {
            "cmd /c exit 0"
        } else {
            "true"
        };
        let proc = call(&env, "std::process::spawn", vec![Value::str(cmd)]).unwrap();
        let status = call(&env, "std::process::wait", vec![proc.clone()]).unwrap();
        assert!(matches!(
            call(&env, "std::process::exit_code", vec![status]).unwrap(),
            Value::Int(0)
        ));
        match call(&env, "std::process::wait", vec![proc]) {
            Err(RuntimeError::Message(msg)) => assert!(msg.contains("already waited")),
            other => panic!("expected already-waited error, got {other:?}"),
        }
    }

    #[test]
    fn s22_memory_constructors_return_kind_specific_stores() {
        let env = make_env();
        for (name, expected) in [
            ("memory::working", "WorkingStore"),
            ("memory::episodic", "EpisodeStore"),
            ("memory::semantic", "VectorIndex"),
            ("memory::procedural", "WorkflowStore"),
        ] {
            match call(&env, name, vec![Value::str("scratch")]).unwrap() {
                Value::MemoryStore { backend, .. } => assert_eq!(backend.kind_name(), expected),
                other => panic!("expected MemoryStore from {name}, got {other:?}"),
            }
        }
    }

    // ── S23: structured argv + output capture ──

    /// (program, argv-array) that echoes `marker` and exits 0, per host.
    fn echo_argv(marker: &str) -> (Value, Value) {
        if cfg!(windows) {
            (
                Value::str("cmd"),
                Value::array(vec![
                    Value::str("/c"),
                    Value::str("echo"),
                    Value::str(marker),
                ]),
            )
        } else {
            (Value::str("echo"), Value::array(vec![Value::str(marker)]))
        }
    }

    #[test]
    fn s23_process_prims_bound() {
        let env = make_env();
        for n in ["std::process::spawn_args", "std::process::output"] {
            assert!(env.get(n).is_some(), "S23 prim `{n}` not bound");
        }
    }

    #[test]
    fn s23_output_returns_map_with_stdout_and_exit_code() {
        let env = make_env();
        let (prog, argv) = echo_argv("garnet-s23-bridge");
        let result = call(&env, "std::process::output", vec![prog, argv]).unwrap();
        match result {
            Value::Map(m) => {
                let m = m.borrow();
                match m.get("stdout") {
                    Some(Value::Str(s)) => assert!(
                        s.contains("garnet-s23-bridge"),
                        "stdout should contain the marker: {s:?}"
                    ),
                    other => panic!("expected stdout String, got {other:?}"),
                }
                assert!(matches!(m.get("code"), Some(Value::Int(0))));
                assert!(m.contains_key("stderr"));
            }
            other => panic!("expected Map from std::process::output, got {other:?}"),
        }
    }

    #[test]
    fn s23_spawn_args_runs_explicit_argv_and_exits_zero() {
        let env = make_env();
        let (prog, argv) = echo_argv("ignored");
        let proc = call(&env, "std::process::spawn_args", vec![prog, argv]).unwrap();
        let status = call(&env, "std::process::wait", vec![proc]).unwrap();
        assert!(matches!(
            call(&env, "std::process::exit_code", vec![status]).unwrap(),
            Value::Int(0)
        ));
    }

    #[test]
    fn s23_output_argv_must_be_strings() {
        let env = make_env();
        match call(
            &env,
            "std::process::output",
            vec![Value::str("echo"), Value::array(vec![Value::Int(7)])],
        ) {
            Err(err) => {
                let rendered = format!("{err:?}");
                assert!(
                    rendered.contains("argv element 0 must be a String"),
                    "expected argv type error, got: {rendered}"
                );
            }
            Ok(v) => panic!("expected argv type error, got Ok({v:?})"),
        }
    }

    // ── S24: std::log file sink ──

    #[test]
    fn s24_log_to_file_bound_and_writes_line() {
        let env = make_env();
        assert!(
            env.get("std::log::to_file").is_some(),
            "S24 prim `std::log::to_file` not bound"
        );

        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("garnet_s24_bridge_{nanos}.log"));
        let p = path.to_str().unwrap();

        let line = call(
            &env,
            "std::log::to_file",
            vec![Value::str(p), Value::str("INFO"), Value::str("bridge")],
        )
        .unwrap();
        match line {
            Value::Str(s) => assert_eq!(s.as_str(), "[INFO] bridge"),
            other => panic!("expected formatted line String, got {other:?}"),
        }
        assert_eq!(
            std::fs::read_to_string(&path).unwrap(),
            "[INFO] bridge\n",
            "to_file should have appended the formatted line"
        );
        std::fs::remove_file(&path).ok();
    }

    // ── S26: core::result dispatch ──

    /// A test-only native returning `Ok(int + 1)` — stands in for a Garnet
    /// Result-returning callable passed to `and_then` / `or_else`.
    fn ok_succ_fn() -> Value {
        fn ok_succ(args: Vec<Value>) -> Result<Value, RuntimeError> {
            match args.first() {
                Some(Value::Int(i)) => Ok(result_ok(Value::Int(i + 1))),
                other => Err(RuntimeError::msg(format!(
                    "ok_succ: int expected, got {other:?}"
                ))),
            }
        }
        Value::NativeFn(Rc::new(NativeFnValue {
            name: "ok_succ",
            arity: Some(1),
            ptr: ok_succ,
        }))
    }

    fn result_parts(value: &Value) -> (String, Value) {
        match value {
            Value::Variant {
                path,
                variant,
                fields,
            } if path.len() == 1 && path[0] == "Result" => (
                variant.to_string(),
                fields.first().cloned().unwrap_or(Value::Nil),
            ),
            other => panic!("expected Result Variant, got {other:?}"),
        }
    }

    #[test]
    fn s26_result_prims_bound() {
        let env = make_env();
        for n in [
            "core::result::ok",
            "core::result::err",
            "core::result::map",
            "core::result::and_then",
            "core::result::or_else",
            "core::result::unwrap_or",
        ] {
            assert!(env.get(n).is_some(), "S26 prim `{n}` not bound");
        }
    }

    #[test]
    fn s26_result_map_transforms_ok_and_passes_err() {
        let env = make_env();
        let ok = call(&env, "core::result::ok", vec![Value::Int(5)]).unwrap();
        let mapped = call(&env, "core::result::map", vec![ok, double_fn()]).unwrap();
        let (variant, inner) = result_parts(&mapped);
        assert_eq!(variant, "Ok");
        assert!(matches!(inner, Value::Int(10)));

        let err = call(&env, "core::result::err", vec![Value::str("boom")]).unwrap();
        let mapped_err = call(&env, "core::result::map", vec![err, double_fn()]).unwrap();
        let (variant, inner) = result_parts(&mapped_err);
        assert_eq!(variant, "Err", "map must not touch Err");
        assert_eq!(inner.display(), "boom");
    }

    #[test]
    fn s26_result_and_then_chains_ok_and_short_circuits_err() {
        let env = make_env();
        let ok = call(&env, "core::result::ok", vec![Value::Int(5)]).unwrap();
        let chained = call(&env, "core::result::and_then", vec![ok, ok_succ_fn()]).unwrap();
        let (variant, inner) = result_parts(&chained);
        assert_eq!(variant, "Ok");
        assert!(matches!(inner, Value::Int(6)), "and_then applies fn to Ok");

        let err = call(&env, "core::result::err", vec![Value::str("stop")]).unwrap();
        let short = call(&env, "core::result::and_then", vec![err, ok_succ_fn()]).unwrap();
        let (variant, inner) = result_parts(&short);
        assert_eq!(variant, "Err", "and_then short-circuits on Err");
        assert_eq!(inner.display(), "stop");
    }

    #[test]
    fn s26_result_or_else_recovers_err_and_passes_ok() {
        let env = make_env();
        let err = call(&env, "core::result::err", vec![Value::Int(0)]).unwrap();
        let recovered = call(&env, "core::result::or_else", vec![err, ok_succ_fn()]).unwrap();
        let (variant, inner) = result_parts(&recovered);
        assert_eq!(variant, "Ok", "or_else recovers Err via fn");
        assert!(matches!(inner, Value::Int(1)));

        let ok = call(&env, "core::result::ok", vec![Value::Int(7)]).unwrap();
        let passed = call(&env, "core::result::or_else", vec![ok, ok_succ_fn()]).unwrap();
        let (variant, inner) = result_parts(&passed);
        assert_eq!(variant, "Ok", "or_else leaves Ok untouched");
        assert!(matches!(inner, Value::Int(7)));
    }

    #[test]
    fn s26_result_unwrap_or_returns_value_or_default() {
        let env = make_env();
        let ok = call(&env, "core::result::ok", vec![Value::Int(42)]).unwrap();
        assert!(matches!(
            call(&env, "core::result::unwrap_or", vec![ok, Value::Int(0)]).unwrap(),
            Value::Int(42)
        ));
        let err = call(&env, "core::result::err", vec![Value::str("e")]).unwrap();
        assert!(matches!(
            call(&env, "core::result::unwrap_or", vec![err, Value::Int(99)]).unwrap(),
            Value::Int(99)
        ));
    }

    #[test]
    fn s26_result_map_rejects_non_result() {
        let env = make_env();
        match call(&env, "core::result::map", vec![Value::Int(3), double_fn()]) {
            Err(e) => assert!(
                format!("{e:?}").contains("Result arg at position 0"),
                "expected Result type error, got {e:?}"
            ),
            Ok(v) => panic!("expected type error, got Ok({v:?})"),
        }
    }

    // ── S27: core::option dispatch ──

    /// A test-only native returning `Some(int + 1)` — a Garnet Option-returning
    /// callable for `and_then`.
    fn some_succ_fn() -> Value {
        fn some_succ(args: Vec<Value>) -> Result<Value, RuntimeError> {
            match args.first() {
                Some(Value::Int(i)) => Ok(option_some(Value::Int(i + 1))),
                other => Err(RuntimeError::msg(format!(
                    "some_succ: int expected, got {other:?}"
                ))),
            }
        }
        Value::NativeFn(Rc::new(NativeFnValue {
            name: "some_succ",
            arity: Some(1),
            ptr: some_succ,
        }))
    }

    /// `(variant_name, inner_or_nil)` for an Option Variant.
    fn option_parts(value: &Value) -> (String, Value) {
        match value {
            Value::Variant {
                path,
                variant,
                fields,
            } if path.len() == 1 && path[0] == "Option" => (
                variant.to_string(),
                fields.first().cloned().unwrap_or(Value::Nil),
            ),
            other => panic!("expected Option Variant, got {other:?}"),
        }
    }

    #[test]
    fn s27_option_prims_bound() {
        let env = make_env();
        for n in [
            "core::option::some",
            "core::option::none",
            "core::option::map",
            "core::option::and_then",
            "core::option::unwrap_or",
        ] {
            assert!(env.get(n).is_some(), "S27 prim `{n}` not bound");
        }
    }

    #[test]
    fn s27_option_none_constructor_is_none_variant() {
        let env = make_env();
        let none = call(&env, "core::option::none", vec![]).unwrap();
        let (variant, _) = option_parts(&none);
        assert_eq!(variant, "None");
    }

    #[test]
    fn s27_option_map_transforms_some_and_passes_none() {
        let env = make_env();
        let some = call(&env, "core::option::some", vec![Value::Int(5)]).unwrap();
        let mapped = call(&env, "core::option::map", vec![some, double_fn()]).unwrap();
        let (variant, inner) = option_parts(&mapped);
        assert_eq!(variant, "Some");
        assert!(matches!(inner, Value::Int(10)));

        let none = call(&env, "core::option::none", vec![]).unwrap();
        let mapped_none = call(&env, "core::option::map", vec![none, double_fn()]).unwrap();
        assert_eq!(
            option_parts(&mapped_none).0,
            "None",
            "map must not touch None"
        );
    }

    #[test]
    fn s27_option_and_then_chains_some_and_short_circuits_none() {
        let env = make_env();
        let some = call(&env, "core::option::some", vec![Value::Int(5)]).unwrap();
        let chained = call(&env, "core::option::and_then", vec![some, some_succ_fn()]).unwrap();
        let (variant, inner) = option_parts(&chained);
        assert_eq!(variant, "Some");
        assert!(matches!(inner, Value::Int(6)));

        let none = call(&env, "core::option::none", vec![]).unwrap();
        let short = call(&env, "core::option::and_then", vec![none, some_succ_fn()]).unwrap();
        assert_eq!(
            option_parts(&short).0,
            "None",
            "and_then short-circuits on None"
        );
    }

    #[test]
    fn s27_option_unwrap_or_returns_value_or_default() {
        let env = make_env();
        let some = call(&env, "core::option::some", vec![Value::Int(42)]).unwrap();
        assert!(matches!(
            call(&env, "core::option::unwrap_or", vec![some, Value::Int(0)]).unwrap(),
            Value::Int(42)
        ));
        let none = call(&env, "core::option::none", vec![]).unwrap();
        assert!(matches!(
            call(&env, "core::option::unwrap_or", vec![none, Value::Int(99)]).unwrap(),
            Value::Int(99)
        ));
    }

    #[test]
    fn s27_option_map_rejects_non_option() {
        let env = make_env();
        match call(&env, "core::option::map", vec![Value::Int(3), double_fn()]) {
            Err(e) => assert!(
                format!("{e:?}").contains("Option arg at position 0"),
                "expected Option type error, got {e:?}"
            ),
            Ok(v) => panic!("expected type error, got Ok({v:?})"),
        }
    }

    // ── S28: core::iter completion (zip / collect / chain) ──

    fn ints(values: &[i64]) -> Value {
        Value::array(values.iter().copied().map(Value::Int).collect())
    }

    fn int_vec(value: &Value) -> Vec<i64> {
        match value {
            Value::Array(a) => a
                .borrow()
                .iter()
                .map(|v| match v {
                    Value::Int(i) => *i,
                    other => panic!("expected Int, got {other:?}"),
                })
                .collect(),
            other => panic!("expected Array, got {other:?}"),
        }
    }

    #[test]
    fn s28_iter_prims_bound() {
        let env = make_env();
        for n in [
            "core::iter::zip",
            "core::iter::collect",
            "core::iter::chain",
        ] {
            assert!(env.get(n).is_some(), "S28 prim `{n}` not bound");
        }
    }

    #[test]
    fn s28_iter_zip_pairs_to_shorter_length() {
        let env = make_env();
        let zipped = call(
            &env,
            "core::iter::zip",
            vec![ints(&[1, 2, 3]), ints(&[10, 20])],
        )
        .unwrap();
        match zipped {
            Value::Array(a) => {
                let a = a.borrow();
                assert_eq!(a.len(), 2, "zip stops at the shorter sequence");
                assert_eq!(int_vec(&a[0]), vec![1, 10]);
                assert_eq!(int_vec(&a[1]), vec![2, 20]);
            }
            other => panic!("expected Array of pairs, got {other:?}"),
        }
    }

    #[test]
    fn s28_iter_chain_concatenates() {
        let env = make_env();
        let chained = call(
            &env,
            "core::iter::chain",
            vec![ints(&[1, 2]), ints(&[3, 4, 5])],
        )
        .unwrap();
        assert_eq!(int_vec(&chained), vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn s28_iter_collect_expands_range_and_materializes_array() {
        let env = make_env();
        // Exclusive range 1..5 -> [1,2,3,4].
        let excl = call(
            &env,
            "core::iter::collect",
            vec![Value::Range {
                start: 1,
                end: 5,
                inclusive: false,
            }],
        )
        .unwrap();
        assert_eq!(int_vec(&excl), vec![1, 2, 3, 4]);

        // Inclusive range 1..=3 -> [1,2,3].
        let incl = call(
            &env,
            "core::iter::collect",
            vec![Value::Range {
                start: 1,
                end: 3,
                inclusive: true,
            }],
        )
        .unwrap();
        assert_eq!(int_vec(&incl), vec![1, 2, 3]);

        // Array passes through (materialized).
        let arr = call(&env, "core::iter::collect", vec![ints(&[7, 8])]).unwrap();
        assert_eq!(int_vec(&arr), vec![7, 8]);
    }

    #[test]
    fn s28_iter_collect_rejects_non_sequence() {
        let env = make_env();
        match call(&env, "core::iter::collect", vec![Value::Int(3)]) {
            Err(e) => assert!(
                format!("{e:?}").contains("Array or Range arg at position 0"),
                "expected sequence type error, got {e:?}"
            ),
            Ok(v) => panic!("expected type error, got Ok({v:?})"),
        }
    }
}
