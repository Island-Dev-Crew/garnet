//! Extended safe-mode checker tests beyond the smoke-test set in
//! `src/lib.rs`. These exercise edge cases that the v0.4 borrow checker
//! will build on top of.

use garnet_check::{check_module, CheckError};
use garnet_parser::parse_source;

fn check(src: &str) -> garnet_check::CheckReport {
    let m = parse_source(src).expect("parse");
    check_module(&m)
}

// ── Mode tagging ──

#[test]
fn def_function_tagged_managed() {
    let r = check("def f() { 1 }");
    assert_eq!(r.mode_map.len(), 1);
    assert_eq!(r.mode_map[0].0, "f");
    assert!(matches!(
        r.mode_map[0].1,
        garnet_parser::ast::FnMode::Managed
    ));
}

#[test]
fn fn_function_tagged_safe() {
    let r = check("fn f(x: Int) -> Int { x }");
    assert_eq!(r.mode_map.len(), 1);
    assert!(matches!(r.mode_map[0].1, garnet_parser::ast::FnMode::Safe));
}

#[test]
fn multiple_functions_each_tagged() {
    let r = check("def m1() { 1 }\nfn s1(x: Int) -> Int { x }\ndef m2() { 2 }");
    assert_eq!(r.mode_map.len(), 3);
}

// ── Annotation bound checks ──

#[test]
fn max_depth_in_range_passes() {
    let r = check("@max_depth(8) def f() { 1 }");
    assert!(!r
        .errors
        .iter()
        .any(|e| matches!(e, CheckError::AnnotationError(_))));
}

#[test]
fn max_depth_zero_fails() {
    let r = check("@max_depth(0) def f() { 1 }");
    assert!(r
        .errors
        .iter()
        .any(|e| matches!(e, CheckError::AnnotationError(_))));
}

#[test]
fn max_depth_overflow_fails() {
    let r = check("@max_depth(100) def f() { 1 }");
    assert!(r
        .errors
        .iter()
        .any(|e| matches!(e, CheckError::AnnotationError(_))));
}

#[test]
fn fan_out_in_range_passes() {
    let r = check("@fan_out(50) def f() { 1 }");
    assert!(!r
        .errors
        .iter()
        .any(|e| matches!(e, CheckError::AnnotationError(_))));
}

#[test]
fn fan_out_zero_fails() {
    let r = check("@fan_out(0) def f() { 1 }");
    assert!(r
        .errors
        .iter()
        .any(|e| matches!(e, CheckError::AnnotationError(_))));
}

#[test]
fn fan_out_overflow_fails() {
    let r = check("@fan_out(2000) def f() { 1 }");
    assert!(r
        .errors
        .iter()
        .any(|e| matches!(e, CheckError::AnnotationError(_))));
}

// ── Safe-mode discipline ──

#[test]
fn safe_module_var_use_flagged() {
    let r = check("@safe\ndef bad() { var x = 1\n x }");
    assert!(r
        .errors
        .iter()
        .any(|e| matches!(e, CheckError::SafeModeViolation(m) if m.contains("var"))));
}

#[test]
fn safe_module_raise_flagged() {
    let r = check(
        r#"@safe
        def bad() {
            raise "oops"
        }
    "#,
    );
    assert!(r
        .errors
        .iter()
        .any(|e| matches!(e, CheckError::SafeModeViolation(m) if m.contains("raise"))));
}

#[test]
fn safe_module_try_flagged() {
    let r = check(
        r#"@safe
        def bad() {
            try { 1 } rescue e { 2 }
        }
    "#,
    );
    assert!(r
        .errors
        .iter()
        .any(|e| matches!(e, CheckError::SafeModeViolation(m) if m.contains("try"))));
}

#[test]
fn managed_module_var_allowed() {
    let r = check("def ok() { var x = 1\n x }");
    assert!(!r
        .errors
        .iter()
        .any(|e| matches!(e, CheckError::SafeModeViolation(_))));
}

#[test]
fn managed_module_raise_allowed() {
    let r = check(r#"def ok() { raise "fine" }"#);
    assert!(!r
        .errors
        .iter()
        .any(|e| matches!(e, CheckError::SafeModeViolation(_))));
}

#[test]
fn safe_fn_with_full_signature_passes() {
    let r = check("fn good(x: Int, y: Int) -> Int { x + y }");
    assert!(!r
        .errors
        .iter()
        .any(|e| matches!(e, CheckError::SafeModeViolation(_))));
}

// ── Boundary call site detection ──

#[test]
fn boundary_count_increments_per_call() {
    let r = check(
        "def main() { foo() + bar() + baz() }\n\
         def foo() { 1 } def bar() { 2 } def baz() { 3 }",
    );
    // 3 calls in main; foo/bar/baz have no calls.
    assert!(r.boundary_call_sites >= 3);
}

#[test]
fn boundary_count_zero_for_pure_arithmetic() {
    let r = check("def f() { 1 + 2 + 3 + 4 }");
    assert_eq!(r.boundary_call_sites, 0);
}

#[test]
fn boundary_count_increments_for_method_call() {
    let r = check(
        r#"
        def f() {
            let s = "hello"
            s.upcase()
        }
    "#,
    );
    assert!(r.boundary_call_sites >= 1);
}

// ── Report.ok() ──

#[test]
fn report_ok_true_for_clean_module() {
    let r = check("def f() { 1 }");
    assert!(r.ok());
}

#[test]
fn report_ok_false_for_safe_violation() {
    let r = check("@safe\ndef bad() { var x = 1\n x }");
    assert!(!r.ok());
}

#[test]
fn report_ok_false_for_annotation_error() {
    let r = check("@max_depth(0) def f() { 1 }");
    assert!(!r.ok());
}
