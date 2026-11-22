//! Regression tests for v0.8.4 checker soundness fixes:
//! - FE-F2b: capability calls inside match arm guards must be walked by CapsGraph.
//! - FE-F4: `let`-bound (and `return`/`assign`/guard/condition) `try`/`rescue` inside `@safe fn`
//!   must produce a fatal `SafeModeViolation`.

use garnet_check::{check_module, CheckError};

#[test]
fn fe_f2b_match_guard_capability_call_is_rejected_without_caps() {
    let src = r#"
@caps()
def main() {
    match 1 {
        n if fs::read_file("README.md") != "" => n
        _ => 0
    }
}
"#;
    let module = garnet_parser::parse_source(src).expect("parse failed");
    let report = check_module(&module);
    assert!(
        !report.ok(),
        "expected undeclared fs call in match guard to fail check, got {:?}",
        report.errors
    );
    assert!(
        report
            .errors
            .iter()
            .any(|e| matches!(e, CheckError::CapsCoverage { .. })),
        "expected CapsCoverage, got {:?}",
        report.errors
    );
}

#[test]
fn fe_f2b_match_guard_capability_call_passes_when_declared() {
    let src = r#"
@caps(fs)
def main() {
    match 1 {
        n if fs::read_file("README.md") != "" => n
        _ => 0
    }
}
"#;
    let module = garnet_parser::parse_source(src).expect("parse failed");
    let report = check_module(&module);
    assert!(
        report.ok(),
        "expected declared @caps(fs) in match guard to pass, got {:?}",
        report.errors
    );
}

#[test]
fn fe_f4_let_bound_try_in_safe_fn_is_fatal_safe_mode_violation() {
    let src = r#"
@safe
@caps()
fn bad_safe(x: Int) -> Int {
    let y = try { x } rescue e { 0 }
    y
}
"#;
    let module = garnet_parser::parse_source(src).expect("parse failed");
    let report = check_module(&module);
    assert!(
        !report.ok(),
        "expected let-bound try in @safe fn to fail check, got {:?}",
        report.errors
    );
    assert!(
        report
            .errors
            .iter()
            .any(|e| matches!(e, CheckError::SafeModeViolation(msg) if msg.contains("try"))),
        "expected SafeModeViolation mentioning try, got {:?}",
        report.errors
    );
}

#[test]
fn fe_f4_return_and_guard_try_in_safe_fn_are_rejected() {
    let src = r#"
@safe
@caps()
fn bad_return(x: Int) -> Int {
    return try { x } rescue e { 0 }
}
"#;
    let module = garnet_parser::parse_source(src).expect("parse failed");
    let report = check_module(&module);
    assert!(
        report
            .errors
            .iter()
            .any(|e| matches!(e, CheckError::SafeModeViolation(msg) if msg.contains("try"))),
        "expected SafeModeViolation for return try, got {:?}",
        report.errors
    );
}
