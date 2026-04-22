//! Tests for the simple borrow-checker pass added in v3.1.
//!
//! The pass intentionally has limited scope: it tracks moves through `own`
//! parameters of top-level safe `fn` calls and flags use-after-move +
//! basic aliasing-XOR-mutation. The more powerful flow-sensitive checker
//! is v0.4 work; these tests pin the v3.1 contract.

use garnet_check::{borrow::check_borrows, CheckError};
use garnet_parser::parse_source;

fn diagnose(src: &str) -> Vec<CheckError> {
    let m = parse_source(src).expect("parse");
    check_borrows(&m)
}

// ── Use-after-move ──

#[test]
fn use_after_move_into_own_param_flagged() {
    let src = r#"
        fn consume(own x: Buffer) -> Int { 0 }
        fn caller(own b: Buffer) -> Int {
            consume(b)
            consume(b)
            0
        }
    "#;
    let d = diagnose(src);
    assert!(
        d.iter().any(|e| matches!(e, CheckError::SafeModeViolation(m) if m.contains("use-after-move"))),
        "expected use-after-move, got {d:?}"
    );
}

#[test]
fn no_diag_for_single_consume() {
    let src = r#"
        fn consume(own x: Buffer) -> Int { 0 }
        fn caller(own b: Buffer) -> Int {
            consume(b)
            0
        }
    "#;
    let d = diagnose(src);
    assert!(d.is_empty(), "expected no diagnostics, got {d:?}");
}

#[test]
fn borrow_param_does_not_move() {
    let src = r#"
        fn read(borrow x: Buffer) -> Int { 0 }
        fn caller(borrow b: Buffer) -> Int {
            read(b)
            read(b)
            read(b)
            0
        }
    "#;
    let d = diagnose(src);
    assert!(d.is_empty(), "borrow params should not be moved, got {d:?}");
}

#[test]
fn re_let_resets_moved_state() {
    let src = r#"
        fn consume(own x: Buffer) -> Int { 0 }
        fn fresh() -> Buffer { Buffer::new() }
        fn caller() -> Int {
            let b = fresh()
            consume(b)
            let b = fresh()
            consume(b)
            0
        }
    "#;
    let d = diagnose(src);
    assert!(d.is_empty(), "re-let should rebind, got {d:?}");
}

// ── Aliasing ──

#[test]
fn mut_with_other_arg_to_same_binding_flagged() {
    let src = r#"
        fn frob(mut a: Buffer, borrow b: Buffer) -> Int { 0 }
        fn caller(mut x: Buffer) -> Int {
            frob(x, x)
        }
    "#;
    let d = diagnose(src);
    assert!(
        d.iter().any(|e| matches!(e, CheckError::SafeModeViolation(m) if m.contains("aliasing"))),
        "expected aliasing violation, got {d:?}"
    );
}

#[test]
fn distinct_bindings_into_mut_and_borrow_ok() {
    let src = r#"
        fn frob(mut a: Buffer, borrow b: Buffer) -> Int { 0 }
        fn caller(mut x: Buffer, borrow y: Buffer) -> Int {
            frob(x, y)
        }
    "#;
    let d = diagnose(src);
    assert!(
        !d.iter().any(|e| matches!(e, CheckError::SafeModeViolation(m) if m.contains("aliasing"))),
        "no aliasing for distinct bindings: {d:?}"
    );
}

// ── Managed (def) functions are not borrow-checked ──

#[test]
fn managed_def_double_use_not_flagged_by_borrow_pass() {
    let src = r#"
        fn consume(own x: Buffer) -> Int { 0 }
        def caller(b) {
            consume(b)
            consume(b)
            0
        }
    "#;
    let d = diagnose(src);
    assert!(
        !d.iter().any(|e| matches!(e, CheckError::SafeModeViolation(m) if m.contains("use-after-move"))),
        "managed def is ARC, not affine: {d:?}"
    );
}

// ── Branches conservatively merge ──

#[test]
fn move_in_one_branch_propagates_after_if() {
    let src = r#"
        fn consume(own x: Buffer) -> Int { 0 }
        fn caller(own b: Buffer, c: Bool) -> Int {
            if c {
                consume(b)
            } else {
                0
            }
            consume(b)
        }
    "#;
    let d = diagnose(src);
    assert!(
        d.iter().any(|e| matches!(e, CheckError::SafeModeViolation(m) if m.contains("use-after-move"))),
        "moves from branches must merge, got {d:?}"
    );
}

// ── Closures and methods are deferred ──

#[test]
fn method_call_does_not_yet_track_moves() {
    let src = r#"
        fn caller(own b: Buffer) -> Int {
            b.consume()
            b.consume()
            0
        }
    "#;
    let d = diagnose(src);
    assert!(
        !d.iter().any(|e| matches!(e, CheckError::SafeModeViolation(m) if m.contains("use-after-move"))),
        "method calls deferred to v0.4: {d:?}"
    );
}
