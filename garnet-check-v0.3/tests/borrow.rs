//! Tests for the simple borrow-checker pass added in v3.1.
//!
//! The pass intentionally has limited scope: it tracks moves through `own`
//! parameters of top-level safe `fn` calls, plus unambiguous same-module
//! `self` method receivers, and flags use-after-move + basic
//! aliasing-XOR-mutation. The more powerful type-resolved checker is later
//! work; these tests pin the current contract.

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
        d.iter()
            .any(|e| matches!(e, CheckError::SafeModeViolation(m) if m.contains("use-after-move"))),
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
        d.iter()
            .any(|e| matches!(e, CheckError::SafeModeViolation(m) if m.contains("aliasing"))),
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
        !d.iter()
            .any(|e| matches!(e, CheckError::SafeModeViolation(m) if m.contains("aliasing"))),
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
        !d.iter()
            .any(|e| matches!(e, CheckError::SafeModeViolation(m) if m.contains("use-after-move"))),
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
        d.iter()
            .any(|e| matches!(e, CheckError::SafeModeViolation(m) if m.contains("use-after-move"))),
        "moves from branches must merge, got {d:?}"
    );
}

#[test]
fn move_in_returning_then_branch_does_not_propagate_after_if() {
    let src = r#"
        fn consume(own x: Buffer) -> Int { 0 }
        fn read(borrow x: Buffer) -> Int { 0 }
        fn caller(own b: Buffer, c: Bool) -> Int {
            if c {
                consume(b)
                return 0
            } else {
                0
            }
            read(b)
            0
        }
    "#;
    let d = diagnose(src);
    assert!(
        !d.iter()
            .any(|e| matches!(e, CheckError::SafeModeViolation(m) if m.contains("use-after-move"))),
        "a move in a returning branch should not poison continuing paths, got {d:?}"
    );
}

#[test]
fn move_in_returning_else_branch_does_not_propagate_after_if() {
    let src = r#"
        fn consume(own x: Buffer) -> Int { 0 }
        fn read(borrow x: Buffer) -> Int { 0 }
        fn caller(own b: Buffer, c: Bool) -> Int {
            if c {
                0
            } else {
                consume(b)
                return 0
            }
            read(b)
            0
        }
    "#;
    let d = diagnose(src);
    assert!(
        !d.iter()
            .any(|e| matches!(e, CheckError::SafeModeViolation(m) if m.contains("use-after-move"))),
        "a move in a returning else branch should not poison continuing paths, got {d:?}"
    );
}

#[test]
fn statements_after_return_are_not_borrow_checked() {
    let src = r#"
        fn consume(own x: Buffer) -> Int { 0 }
        fn read(borrow x: Buffer) -> Int { 0 }
        fn caller(own b: Buffer) -> Int {
            consume(b)
            return 0
            read(b)
            0
        }
    "#;
    let d = diagnose(src);
    assert!(
        !d.iter()
            .any(|e| matches!(e, CheckError::SafeModeViolation(m) if m.contains("use-after-move"))),
        "unreachable statements after return should not poison safe-mode liveness, got {d:?}"
    );
}

#[test]
fn move_in_returning_while_body_does_not_poison_after_loop() {
    let src = r#"
        fn consume(own x: Buffer) -> Int { 0 }
        fn read(borrow x: Buffer) -> Int { 0 }
        fn caller(own b: Buffer, c: Bool) -> Int {
            while c {
                consume(b)
                return 0
            }
            read(b)
            0
        }
    "#;
    let d = diagnose(src);
    assert!(
        !d.iter()
            .any(|e| matches!(e, CheckError::SafeModeViolation(m) if m.contains("use-after-move"))),
        "a move in a while body that returns should not poison paths where the loop body never runs, got {d:?}"
    );
}

#[test]
fn move_in_returning_for_body_does_not_poison_after_loop() {
    let src = r#"
        fn consume(own x: Buffer) -> Int { 0 }
        fn read(borrow x: Buffer) -> Int { 0 }
        fn caller(own b: Buffer, xs: Array<Int>) -> Int {
            for x in xs {
                consume(b)
                return 0
            }
            read(b)
            0
        }
    "#;
    let d = diagnose(src);
    assert!(
        !d.iter()
            .any(|e| matches!(e, CheckError::SafeModeViolation(m) if m.contains("use-after-move"))),
        "a move in a for body that returns should not poison paths where the loop body never runs, got {d:?}"
    );
}

#[test]
fn for_loop_variable_shadowing_does_not_rebind_outer_move() {
    let src = r#"
        fn consume(own x: Buffer) -> Int { 0 }
        fn read(borrow x: Buffer) -> Int { 0 }
        fn caller(own item: Buffer, xs: Array<Int>) -> Int {
            consume(item)
            for item in xs {
                0
            }
            read(item)
            0
        }
    "#;
    let d = diagnose(src);
    assert!(
        d.iter()
            .any(|e| matches!(e, CheckError::SafeModeViolation(m) if m.contains("use-after-move"))),
        "a for-loop variable must not erase the moved state of an outer binding with the same name, got {d:?}"
    );
}

#[test]
fn match_pattern_shadow_move_does_not_poison_outer_binding() {
    let src = r#"
        fn consume(own x: Buffer) -> Int { 0 }
        fn read(borrow x: Buffer) -> Int { 0 }
        fn caller(borrow item: Buffer, own subject: Buffer) -> Int {
            match subject {
                item => consume(item)
            }
            read(item)
            0
        }
    "#;
    let d = diagnose(src);
    assert!(
        !d.iter()
            .any(|e| matches!(e, CheckError::SafeModeViolation(m) if m.contains("use-after-move"))),
        "a move of a match-arm pattern binding should not poison an outer binding with the same name, got {d:?}"
    );
}

#[test]
fn match_arm_outer_move_still_propagates_after_match() {
    let src = r#"
        fn consume(own x: Buffer) -> Int { 0 }
        fn read(borrow x: Buffer) -> Int { 0 }
        fn caller(own item: Buffer, n: Int) -> Int {
            match n {
                _ => consume(item)
            }
            read(item)
            0
        }
    "#;
    let d = diagnose(src);
    assert!(
        d.iter()
            .any(|e| matches!(e, CheckError::SafeModeViolation(m) if m.contains("use-after-move"))),
        "a move of an outer binding inside a match arm must still poison later uses, got {d:?}"
    );
}

#[test]
fn match_arm_block_statement_move_still_propagates_after_match() {
    let src = r#"
        fn consume(own x: Buffer) -> Int { 0 }
        fn read(borrow x: Buffer) -> Int { 0 }
        fn caller(own item: Buffer, n: Int) -> Int {
            match n {
                _ => {
                    consume(item)
                    0
                }
            }
            read(item)
            0
        }
    "#;
    let d = diagnose(src);
    assert!(
        d.iter()
            .any(|e| matches!(e, CheckError::SafeModeViolation(m) if m.contains("use-after-move"))),
        "a move inside a match-arm block statement must be preserved for later uses, got {d:?}"
    );
}

#[test]
fn match_guard_move_propagates_even_when_arm_body_returns() {
    let src = r#"
        fn consumes_false(own x: Buffer) -> Bool { false }
        fn read(borrow x: Buffer) -> Int { 0 }
        fn caller(own item: Buffer, n: Int) -> Int {
            match n {
                _ if consumes_false(item) => {
                    return 0
                },
                _ => 0
            }
            read(item)
            0
        }
    "#;
    let d = diagnose(src);
    assert!(
        d.iter()
            .any(|e| matches!(e, CheckError::SafeModeViolation(m) if m.contains("use-after-move"))),
        "a move in a match guard can continue when the guard is false, got {d:?}"
    );
}

// ── Method receivers ──

#[test]
fn method_call_own_self_tracks_moves() {
    let src = r#"
        impl Buffer {
            fn consume(own self) -> Int { 0 }
        }

        fn caller(own b: Buffer) -> Int {
            b.consume()
            b.consume()
            0
        }
    "#;
    let d = diagnose(src);
    assert!(
        d.iter()
            .any(|e| matches!(e, CheckError::SafeModeViolation(m) if m.contains("use-after-move"))),
        "expected method receiver use-after-move, got {d:?}"
    );
}

#[test]
fn method_call_mut_self_alias_flagged() {
    let src = r#"
        impl Buffer {
            fn splice(mut self, borrow other: Buffer) -> Int { 0 }
        }

        fn caller(mut b: Buffer) -> Int {
            b.splice(b)
        }
    "#;
    let d = diagnose(src);
    assert!(
        d.iter()
            .any(|e| matches!(e, CheckError::SafeModeViolation(m) if m.contains("aliasing"))),
        "expected method receiver aliasing violation, got {d:?}"
    );
}

#[test]
fn typed_receiver_resolves_same_named_method_signatures() {
    let src = r#"
        struct Buffer {}
        struct Socket {}

        impl Buffer {
            fn close(own self) -> Int { 0 }
        }

        impl Socket {
            fn close(borrow self) -> Int { 0 }
        }

        fn caller(own b: Buffer) -> Int {
            b.close()
            b.close()
            0
        }
    "#;
    let d = diagnose(src);
    assert!(
        d.iter()
            .any(|e| matches!(e, CheckError::SafeModeViolation(m) if m.contains("use-after-move"))),
        "typed receivers should resolve same-named methods, got {d:?}"
    );
}

#[test]
fn typed_receiver_does_not_fallback_to_other_type_method() {
    let src = r#"
        struct Buffer {}
        struct Socket {}

        impl Buffer {
            fn close(own self) -> Int { 0 }
        }

        fn caller(own s: Socket) -> Int {
            s.close()
            s.close()
            0
        }
    "#;
    let d = diagnose(src);
    assert!(
        !d.iter()
            .any(|e| matches!(e, CheckError::SafeModeViolation(m) if m.contains("use-after-move"))),
        "typed receiver with no matching impl should not borrow-check another type's method: {d:?}"
    );
}

// ── Place-granular field projections ──

#[test]
fn same_field_mut_and_borrow_alias_flagged() {
    let src = r#"
        struct Pair {
            left: Buffer,
            right: Buffer,
        }

        fn frob(mut a: Buffer, borrow b: Buffer) -> Int { 0 }

        fn caller(mut p: Pair) -> Int {
            frob(p.left, p.left)
        }
    "#;
    let d = diagnose(src);
    assert!(
        d.iter()
            .any(|e| matches!(e, CheckError::SafeModeViolation(m) if m.contains("aliasing"))),
        "same field projection must be alias-checked, got {d:?}"
    );
}

#[test]
fn distinct_field_mut_and_borrow_are_not_aliases() {
    let src = r#"
        struct Pair {
            left: Buffer,
            right: Buffer,
        }

        fn frob(mut a: Buffer, borrow b: Buffer) -> Int { 0 }

        fn caller(mut p: Pair) -> Int {
            frob(p.left, p.right)
        }
    "#;
    let d = diagnose(src);
    assert!(
        !d.iter()
            .any(|e| matches!(e, CheckError::SafeModeViolation(m) if m.contains("aliasing"))),
        "distinct fields should not be treated as the same place: {d:?}"
    );
}

#[test]
fn parent_and_child_places_alias() {
    let src = r#"
        struct Pair {
            left: Buffer,
            right: Buffer,
        }

        fn frob(mut p: Pair, borrow b: Buffer) -> Int { 0 }

        fn caller(mut p: Pair) -> Int {
            frob(p, p.left)
        }
    "#;
    let d = diagnose(src);
    assert!(
        d.iter()
            .any(|e| matches!(e, CheckError::SafeModeViolation(m) if m.contains("aliasing"))),
        "parent and child places overlap and must alias, got {d:?}"
    );
}

#[test]
fn moving_field_rejects_same_field_but_allows_sibling() {
    let src = r#"
        struct Pair {
            left: Buffer,
            right: Buffer,
        }

        fn consume(own x: Buffer) -> Int { 0 }
        fn read(borrow x: Buffer) -> Int { 0 }

        fn caller(mut p: Pair) -> Int {
            consume(p.left)
            read(p.right)
            read(p.left)
            0
        }
    "#;
    let d = diagnose(src);
    assert!(
        d.iter()
            .any(|e| matches!(e, CheckError::SafeModeViolation(m) if m.contains("use-after-move"))),
        "moved field must reject later same-field use while sibling stays available, got {d:?}"
    );
}

#[test]
fn indexed_places_conflict_conservatively() {
    let src = r#"
        fn frob(mut a: Buffer, borrow b: Buffer) -> Int { 0 }

        fn caller(mut items: Buffers) -> Int {
            frob(items[0], items[1])
        }
    "#;
    let d = diagnose(src);
    assert!(
        d.iter()
            .any(|e| matches!(e, CheckError::SafeModeViolation(m) if m.contains("aliasing"))),
        "index projections on the same receiver should conservatively alias, got {d:?}"
    );
}

#[test]
fn indexed_places_under_distinct_fields_do_not_alias() {
    let src = r#"
        struct Pair {
            left: Buffers,
            right: Buffers,
        }

        fn frob(mut a: Buffer, borrow b: Buffer) -> Int { 0 }

        fn caller(mut p: Pair) -> Int {
            frob(p.left[0], p.right[0])
        }
    "#;
    let d = diagnose(src);
    assert!(
        !d.iter()
            .any(|e| matches!(e, CheckError::SafeModeViolation(m) if m.contains("aliasing"))),
        "indexes under distinct sibling fields should not alias: {d:?}"
    );
}

#[test]
fn moving_indexed_place_rejects_same_index_family() {
    let src = r#"
        fn consume(own x: Buffer) -> Int { 0 }
        fn read(borrow x: Buffer) -> Int { 0 }

        fn caller(mut items: Buffers) -> Int {
            consume(items[0])
            read(items[1])
            0
        }
    "#;
    let d = diagnose(src);
    assert!(
        d.iter()
            .any(|e| matches!(e, CheckError::SafeModeViolation(m) if m.contains("use-after-move"))),
        "indexed moves on the same receiver should conservatively poison the index family, got {d:?}"
    );
}

#[test]
fn nested_index_checks_inner_index_expression() {
    let src = r#"
        fn consume(own x: Int) -> Int { 0 }
        fn read(borrow x: Buffer) -> Int { 0 }

        fn caller(mut items: Matrix, own i: Int) -> Int {
            consume(i)
            read(items[i][0])
            0
        }
    "#;
    let d = diagnose(src);
    assert!(
        d.iter()
            .any(|e| matches!(e, CheckError::SafeModeViolation(m) if m.contains("use-after-move"))),
        "nested index receiver must still check its inner index expression, got {d:?}"
    );
}

// ── Drop discipline ──

#[test]
fn double_own_of_same_binding_in_one_call_is_rejected() {
    let src = r#"
        fn consume_pair(own left: Buffer, own right: Buffer) -> Int { 0 }

        fn caller(own b: Buffer) -> Int {
            consume_pair(b, b)
        }
    "#;
    let d = diagnose(src);
    assert!(
        d.iter().any(
            |e| matches!(e, CheckError::SafeModeViolation(m) if m.contains("drop discipline"))
        ),
        "double-own in one call would drop the same binding twice, got {d:?}"
    );
}

#[test]
fn double_own_of_parent_and_child_place_is_rejected() {
    let src = r#"
        struct Pair {
            left: Buffer,
            right: Buffer,
        }

        fn consume_pair(own whole: Pair, own left: Buffer) -> Int { 0 }

        fn caller(own p: Pair) -> Int {
            consume_pair(p, p.left)
        }
    "#;
    let d = diagnose(src);
    assert!(
        d.iter().any(
            |e| matches!(e, CheckError::SafeModeViolation(m) if m.contains("drop discipline"))
        ),
        "owning a parent and child place in one call would double-drop, got {d:?}"
    );
}

#[test]
fn double_own_of_distinct_fields_is_allowed() {
    let src = r#"
        struct Pair {
            left: Buffer,
            right: Buffer,
        }

        fn consume_pair(own left: Buffer, own right: Buffer) -> Int { 0 }

        fn caller(own p: Pair) -> Int {
            consume_pair(p.left, p.right)
            0
        }
    "#;
    let d = diagnose(src);
    assert!(
        !d.iter().any(
            |e| matches!(e, CheckError::SafeModeViolation(m) if m.contains("drop discipline"))
        ),
        "distinct sibling fields should not violate drop discipline: {d:?}"
    );
}
