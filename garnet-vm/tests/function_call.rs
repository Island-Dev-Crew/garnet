//! S14 — Bytecode VM v0.2 function-call lowering tests.
//!
//! The headline case is deep recursion: before S14 the VM recursed in the
//! host (Rust) language inside `execute()`, so deep Garnet recursion
//! overflowed the Rust stack and aborted. After S14 the VM runs an explicit
//! heap-allocated call-frame stack, so the same program runs to completion.

use garnet_interp::{Interpreter, Value};
use garnet_vm::{
    compile_source, deserialize_program, run_function_with_options, serialize_program, RunOptions,
};

fn quiet() -> RunOptions {
    RunOptions { emit_stdout: false }
}

fn interp_value(src: &str, entry: &str, args: Vec<Value>) -> Value {
    let mut interp = Interpreter::new();
    interp.load_source(src).expect("interp load");
    interp.call(entry, args).expect("interp call")
}

#[test]
fn deep_recursion_does_not_overflow_the_host_stack() {
    // 200_000 frames reliably overflows the Rust stack under the old
    // recursive `execute()`; the explicit frame stack runs it on the heap.
    let src = r#"
        def countdown(n) {
            if n <= 0 { 0 } else { countdown(n - 1) }
        }
        def main() { countdown(200000) }
    "#;
    let result = run_function_with_options(src, "main", vec![], quiet()).expect("vm run");
    assert_eq!("0", result.value.display());
    assert_eq!(0, result.summary.fallback_function_calls);
    assert!(result.summary.native_function_calls >= 200000);
}

#[test]
fn mutual_recursion_runs_natively_at_depth() {
    // VM-only: mutual recursion to depth 500 runs entirely on heap frames.
    // No tree-walk comparison here — the interpreter reference overflows its
    // (smaller, ~2 MB) test-thread stack well before this depth in debug
    // builds. VM/interpreter parity for mutual recursion is covered at a safe
    // depth by `mutual_recursion_matches_interpreter_shallow`.
    let src = r#"
        def is_even(n) { if n == 0 { true } else { is_odd(n - 1) } }
        def is_odd(n) { if n == 0 { false } else { is_even(n - 1) } }
        def main() { is_even(500) }
    "#;
    let vm = run_function_with_options(src, "main", vec![], quiet()).expect("vm run");
    assert!(vm.value.eq_deep(&Value::Bool(true)));
    assert_eq!(0, vm.summary.fallback_function_calls);
    assert!(vm.summary.native_function_calls >= 500);
}

#[test]
fn mutual_recursion_matches_interpreter_shallow() {
    // Shallow depth so the tree-walk reference stays within its stack budget;
    // proves VM/interpreter parity for the mutual-recursion shape specifically.
    let src = r#"
        def is_even(n) { if n == 0 { true } else { is_odd(n - 1) } }
        def is_odd(n) { if n == 0 { false } else { is_even(n - 1) } }
        def main() { is_even(20) }
    "#;
    let vm = run_function_with_options(src, "main", vec![], quiet()).expect("vm run");
    assert!(vm.value.eq_deep(&interp_value(src, "main", vec![])));
    assert!(vm.value.eq_deep(&Value::Bool(true)));
}

#[test]
fn mixed_arity_calls_match_interpreter() {
    let src = r#"
        def zero() { 42 }
        def one(a) { a + 1 }
        def two(a, b) { a + b }
        def three(a, b, c) { a + b + c }
        def main() { zero() + one(1) + two(2, 3) + three(4, 5, 6) }
    "#;
    let vm = run_function_with_options(src, "main", vec![], quiet()).expect("vm run");
    // 42 + 2 + 5 + 15 = 64
    assert_eq!("64", vm.value.display());
    assert!(vm.value.eq_deep(&interp_value(src, "main", vec![])));
    assert_eq!(0, vm.summary.fallback_function_calls);
}

#[test]
fn nested_calls_return_values_through_frames() {
    // A returns into B returns into main — exercises the return-value
    // hand-off across frames.
    let src = r#"
        def inner(x) { x * 2 }
        def middle(x) { inner(x) + 1 }
        def outer(x) { middle(x) + middle(x) }
        def main() { outer(10) }
    "#;
    let vm = run_function_with_options(src, "main", vec![], quiet()).expect("vm run");
    // inner(10)=20, middle(10)=21, outer(10)=42
    assert_eq!("42", vm.value.display());
    assert!(vm.value.eq_deep(&interp_value(src, "main", vec![])));
    assert_eq!(0, vm.summary.fallback_function_calls);
}

#[test]
fn abi_v0_3_round_trips_and_is_versioned() {
    let artifact = compile_source("def add(a, b) { a + b }\ndef main() { add(1, 2) }").unwrap();
    let bytes = serialize_program(&artifact.program);
    // Magic header is the v0.3 marker (S99 added the `@max_depth` ceiling field).
    assert_eq!(&bytes[0..8], b"GARNVM03");
    // Deterministic + lossless round trip.
    assert_eq!(bytes, serialize_program(&artifact.program));
    let decoded = deserialize_program(&bytes).unwrap();
    assert_eq!(artifact.program, decoded);
}

#[test]
fn max_depth_ceiling_survives_round_trip() {
    // S99: the `@max_depth(N)` ceiling must survive serialize+deserialize, else a
    // deserialized-from-disk program would silently stop enforcing the ceiling.
    let artifact = compile_source(
        "@max_depth(4)\ndef deep(n) { if n <= 0 { 0 } else { 1 + deep(n - 1) } }\ndef main() { deep(1) }",
    )
    .unwrap();
    assert_eq!(
        artifact.program.function("deep").unwrap().max_depth_ceiling,
        Some(4)
    );
    let decoded = deserialize_program(&serialize_program(&artifact.program)).unwrap();
    assert_eq!(
        decoded.function("deep").unwrap().max_depth_ceiling,
        Some(4),
        "the @max_depth ceiling must round-trip through the codec"
    );
    assert_eq!(
        decoded.function("main").unwrap().max_depth_ceiling,
        None,
        "an unannotated function stays uncapped"
    );
}

#[test]
fn abi_v0_2_rejects_arity_mismatch() {
    // `a + b` lowers to LoadLocal/LoadLocal/Binary/Return — no constants —
    // so the byte layout is deterministic:
    //   magic(8) + const_count u32(4)=0 + fn_count u32(4)=1
    //   + name_len u32(4)=3 + "add"(3)  => the explicit arity u32 starts here.
    let artifact = compile_source("def add(a, b) { a + b }").unwrap();
    assert!(
        artifact.program.constants.is_empty(),
        "test assumes no constants; layout math depends on it"
    );
    let mut bytes = serialize_program(&artifact.program);
    let arity_offset = 8 + 4 + 4 + 4 + "add".len();
    bytes[arity_offset] = 0xFF; // declared arity 255 != params length 2
    let decoded = deserialize_program(&bytes);
    assert!(
        decoded.is_err(),
        "ABI v0.2 must reject an arity/params mismatch: {decoded:?}"
    );
}

#[test]
fn abi_v0_2_rejects_truncated_stream() {
    let artifact = compile_source("def main() { 1 + 2 }").unwrap();
    let mut bytes = serialize_program(&artifact.program);
    bytes.truncate(bytes.len() - 1);
    assert!(deserialize_program(&bytes).is_err());
}
