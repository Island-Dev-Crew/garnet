//! RB-2 — integer-overflow abort conversion (crash-surface sweep).
//!
//! `i64::MIN / -1` (and `% -1`) overflows and was a PROCESS ABORT in both
//! the expression path (`eval.rs`) and the compound-assign path (`stmt.rs`).
//! RB-2 converts the abort into a `RuntimeError` diagnostic. These tests are
//! the red→green proof: before the conversion they fail by panicking.
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
