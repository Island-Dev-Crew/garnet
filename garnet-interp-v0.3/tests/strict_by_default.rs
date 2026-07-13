//! P3-T1 (S114 acceptance, cond. #5) — embedder Interpreters are strict
//! (deny-by-default) unless they opt into permissive. No process-global latch
//! is involved here; this is per-instance behavior on the calling thread.

use garnet_interp::{Interpreter, Value};

/// The trap a frame-less host-authority call raises under strict mode.
const FS_TRAP: &str = "requires @caps(fs)";

/// A default `Interpreter::new()` refuses a frame-less host-authority call —
/// the embedder cannot reach undeclared OS authority just by loading/evaluating
/// untrusted source outside a program entry frame.
#[test]
fn default_interpreter_denies_frameless_host_authority() {
    let interp = Interpreter::new();
    let r = interp.eval_expr_src("read_file(\"/garnet_strict_default_absent\")");
    let msg = format!("{r:?}");
    assert!(r.is_err(), "the call must fail");
    assert!(
        msg.contains(FS_TRAP),
        "default Interpreter must deny a frame-less host call (strict-by-default): {msg}"
    );
}

/// `new_permissive()` restores the pre-S114 direct-call default: a frame-less
/// host call is allowed (it fails only because the path is absent — NOT a trap).
#[test]
fn permissive_interpreter_allows_frameless_host_authority() {
    let interp = Interpreter::new_permissive();
    let r = interp.eval_expr_src("read_file(\"/garnet_strict_default_absent\")");
    let msg = format!("{r:?}");
    assert!(r.is_err(), "absent file → IO error");
    assert!(
        !msg.contains(FS_TRAP),
        "new_permissive() must preserve the permissive direct-call default: {msg}"
    );
}

/// Strict-by-default does not break normal framed programs: a checked
/// `@caps(fs)` program loaded and run through the entry frame still works.
#[test]
fn default_interpreter_still_runs_framed_programs() {
    let mut interp = Interpreter::new();
    interp
        .load_source_with_entry_caps("@caps(fs)\ndef main() -> int { 0 }\n", "main")
        .expect("framed load of a checked @caps program");
    let v = interp.call_entry("main", vec![]).expect("run main");
    assert!(matches!(v, Value::Int(0)), "framed program returns 0");
}

/// Strict-by-default does not affect pure computation (no host authority
/// reached), so a bare embedded `call` of a pure function still works.
#[test]
fn default_interpreter_runs_pure_computation() {
    let mut interp = Interpreter::new();
    interp
        .load_source("def add() -> int { 1 + 2 }\n")
        .expect("load");
    let v = interp.call("add", vec![]).expect("call add");
    assert!(matches!(v, Value::Int(3)));
}
