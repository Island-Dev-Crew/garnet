//! RB-2 + RFC-0002 — integer-overflow abort conversion (crash-surface sweep).
//!
//! RB-2 converted `i64::MIN / -1` and `% -1` from a PROCESS ABORT into a
//! `RuntimeError` diagnostic in both the expression path (`eval.rs`) and the
//! compound-assign path (`stmt.rs`). RFC-0002 ("integer arithmetic is checked
//! by default", accepted 2026-06-12) extends that same discipline to `+`, `-`,
//! `*`, and unary `-`: an i64 overflow is a deterministic diagnostic, never a
//! silent wrap (release) and never a host panic (debug). These tests are the
//! red→green proof: before the checked-arithmetic conversion the `+`/`-`/`*`/
//! unary-`-` cases fail by panicking (`attempt to {add,subtract,multiply,
//! negate} with overflow`).
//!
//! `i64::MIN` is spelled `0 - 9223372036854775807 - 1` because the bare
//! literal `9223372036854775808` does not fit in `i64`.

use garnet_interp::{Interpreter, Value};

const MIN: &str = "(0 - 9223372036854775807 - 1)";

fn eval_err(src: &str) -> String {
    let interp = Interpreter::new();
    match interp.eval_expr_src(src) {
        Ok(v) => panic!("expected runtime error, got {v:?}"),
        Err(e) => e.to_string(),
    }
}

fn run_main_err(body: &str) -> String {
    let mut interp = Interpreter::new();
    let src = format!("@caps()\ndef main() {{\n{body}\n}}\n");
    interp.load_source(&src).expect("program loads");
    match interp.call("main", vec![]) {
        Ok(v) => panic!("expected runtime error, got {v:?}"),
        Err(e) => e.to_string(),
    }
}

#[test]
fn div_min_by_neg_one_is_a_diagnostic_not_an_abort() {
    let msg = eval_err(&format!("{MIN} / (0 - 1)"));
    assert!(
        msg.contains("integer overflow"),
        "expected integer-overflow diagnostic, got: {msg}"
    );
}

#[test]
fn rem_min_by_neg_one_is_a_diagnostic_not_an_abort() {
    let msg = eval_err(&format!("{MIN} % (0 - 1)"));
    assert!(
        msg.contains("integer overflow"),
        "expected integer-overflow diagnostic, got: {msg}"
    );
}

#[test]
fn compound_div_assign_min_by_neg_one_is_a_diagnostic_not_an_abort() {
    let msg = run_main_err(&format!("    var a = {MIN}\n    a /= 0 - 1\n    a"));
    assert!(
        msg.contains("integer overflow"),
        "expected integer-overflow diagnostic, got: {msg}"
    );
}

#[test]
fn compound_rem_assign_min_by_neg_one_is_a_diagnostic_not_an_abort() {
    let msg = run_main_err(&format!("    var a = {MIN}\n    a %= 0 - 1\n    a"));
    assert!(
        msg.contains("integer overflow"),
        "expected integer-overflow diagnostic, got: {msg}"
    );
}

// ── RFC-0002: +, -, *, unary - are checked too ──────────────────────────

const MAX: &str = "9223372036854775807"; // i64::MAX (parses; MAX+1 does not)

#[test]
fn add_max_plus_one_is_a_diagnostic_not_an_abort() {
    let msg = eval_err(&format!("{MAX} + 1"));
    assert!(
        msg.contains("integer overflow"),
        "expected integer-overflow diagnostic, got: {msg}"
    );
}

#[test]
fn sub_min_minus_one_is_a_diagnostic_not_an_abort() {
    let msg = eval_err(&format!("{MIN} - 1"));
    assert!(
        msg.contains("integer overflow"),
        "expected integer-overflow diagnostic, got: {msg}"
    );
}

#[test]
fn mul_overflow_is_a_diagnostic_not_an_abort() {
    // 3037000500^2 = 9223372037000250000 > i64::MAX.
    let msg = eval_err("3037000500 * 3037000500");
    assert!(
        msg.contains("integer overflow"),
        "expected integer-overflow diagnostic, got: {msg}"
    );
}

#[test]
fn neg_min_is_a_diagnostic_not_an_abort() {
    // -(i64::MIN) overflows; -i64::MIN has no i64 representation.
    let msg = eval_err(&format!("0 - ({MIN})"));
    assert!(
        msg.contains("integer overflow"),
        "expected integer-overflow diagnostic, got: {msg}"
    );
    // The unary-negation path specifically (not the binary Sub above).
    let unary = eval_err(&format!("-{MIN}"));
    assert!(
        unary.contains("integer overflow"),
        "expected integer-overflow diagnostic from unary neg, got: {unary}"
    );
}

#[test]
fn compound_add_assign_overflow_is_a_diagnostic_not_an_abort() {
    let msg = run_main_err(&format!("    var a = {MAX}\n    a += 1\n    a"));
    assert!(
        msg.contains("integer overflow"),
        "expected integer-overflow diagnostic, got: {msg}"
    );
}

#[test]
fn compound_sub_assign_overflow_is_a_diagnostic_not_an_abort() {
    let msg = run_main_err(&format!("    var a = {MIN}\n    a -= 1\n    a"));
    assert!(
        msg.contains("integer overflow"),
        "expected integer-overflow diagnostic, got: {msg}"
    );
}

#[test]
fn compound_mul_assign_overflow_is_a_diagnostic_not_an_abort() {
    let msg = run_main_err("    var a = 3037000500\n    a *= 3037000500\n    a");
    assert!(
        msg.contains("integer overflow"),
        "expected integer-overflow diagnostic, got: {msg}"
    );
}

#[test]
fn ordinary_multiplication_still_works() {
    let interp = Interpreter::new();
    let v = interp
        .eval_expr_src("1000000 * 1000000")
        .expect("plain multiplication evals");
    assert!(matches!(v, Value::Int(1000000000000)), "got {v:?}");
}

#[test]
fn ordinary_division_still_works() {
    let interp = Interpreter::new();
    let v = interp
        .eval_expr_src("10 / 3")
        .expect("plain division evals");
    assert!(matches!(v, Value::Int(3)), "got {v:?}");
}

#[test]
fn division_by_zero_message_unchanged() {
    let msg = eval_err("1 / 0");
    assert_eq!(msg, "division by zero");
}
