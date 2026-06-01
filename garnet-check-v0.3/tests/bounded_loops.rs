use garnet_check::{bounded_loop_report, check_module, CheckError};
use garnet_parser::parse_source;

fn parse(src: &str) -> garnet_parser::ast::Module {
    parse_source(src).expect("parse")
}

#[test]
fn safe_for_loop_over_literal_range_is_static_bounded() {
    let module = parse(
        r#"
        fn sum5() -> Int {
            let mut total = 0
            for i in 0..5 {
                total += i
            }
            total
        }
        "#,
    );
    let report = bounded_loop_report(&module);
    assert!(report.ok(), "expected static proof, got {report:?}");
    assert_eq!(report.proven_loops, 1);
    assert_eq!(report.uncheckable_loops.len(), 0);
}

#[test]
fn safe_for_loop_over_literal_array_is_static_bounded() {
    let module = parse(
        r#"
        fn count() -> Int {
            let mut total = 0
            for value in [1, 2, 3] {
                total += value
            }
            total
        }
        "#,
    );
    let report = bounded_loop_report(&module);
    assert!(report.ok(), "expected static proof, got {report:?}");
    assert_eq!(report.proven_loops, 1);
}

#[test]
fn safe_for_loop_with_immediate_return_is_static_bounded() {
    let module = parse(
        r#"
        fn first_or_zero(xs: Array<Int>) -> Int {
            for value in xs {
                return value
            }
            0
        }
        "#,
    );
    let report = bounded_loop_report(&module);
    assert!(
        report.ok(),
        "a for loop whose body exits before continuing is bounded to at most one turn: {report:?}"
    );
    assert_eq!(report.proven_loops, 1);
}

#[test]
fn safe_while_loop_with_nonliteral_condition_is_rejected() {
    let module = parse(
        r#"
        fn countdown(n: Int) -> Int {
            let mut current = n
            while current > 0 {
                current -= 1
            }
            current
        }
        "#,
    );
    let report = bounded_loop_report(&module);
    assert!(!report.ok(), "uncheckable safe while loop must fail");
    assert_eq!(report.uncheckable_loops.len(), 1);
    assert!(report.uncheckable_loops[0].message.contains("while"));

    let check = check_module(&module);
    assert!(
        check.errors.iter().any(|err| {
            matches!(
                err,
                CheckError::BoundedLoop(message)
                    if message.contains("static bounded-loop verifier")
            )
        }),
        "garnet check must surface the fatal bounded-loop diagnostic, got {:?}",
        check.errors
    );
}

#[test]
fn safe_while_loop_with_immediate_return_is_static_bounded() {
    let module = parse(
        r#"
        fn side_effect_free() -> Int {
            0
        }

        fn maybe_consume(flag: Bool) -> Int {
            while flag {
                side_effect_free()
                return 1
            }
            0
        }
        "#,
    );
    let report = bounded_loop_report(&module);
    assert!(
        report.ok(),
        "a while loop whose body exits before continuing is bounded to at most one turn: {report:?}"
    );
    assert_eq!(report.proven_loops, 1);
}

#[test]
fn safe_counter_while_loop_with_literal_limit_is_static_bounded() {
    let module = parse(
        r#"
        fn count_three() -> Int {
            let mut i = 0
            let mut total = 0
            while i < 3 {
                total += i
                i += 1
            }
            total
        }
        "#,
    );
    let report = bounded_loop_report(&module);
    assert!(
        report.ok(),
        "literal counter while loops should be statically bounded: {report:?}"
    );
    assert_eq!(report.proven_loops, 1);
}

#[test]
fn safe_counter_while_loop_with_false_literal_condition_is_static_bounded() {
    let module = parse(
        r#"
        fn already_done() -> Int {
            let mut i = 5
            while i < 3 {
                i
            }
            0
        }
        "#,
    );
    let report = bounded_loop_report(&module);
    assert!(
        report.ok(),
        "a literal counter condition that starts false has a zero-iteration bound: {report:?}"
    );
    assert_eq!(report.proven_loops, 1);
}

#[test]
fn managed_loop_without_safe_or_bounded_annotation_is_out_of_scope() {
    let module = parse(
        r#"
        @caps()
        def main() {
            let mut current = 3
            while current > 0 {
                current -= 1
            }
            current
        }
        "#,
    );
    let report = bounded_loop_report(&module);
    assert!(
        report.ok(),
        "managed loops are outside S93 safe-subset scope"
    );
    assert_eq!(report.skipped_functions, 1);
}

#[test]
fn bounded_managed_function_enters_static_loop_scope() {
    let module = parse(
        r#"
        @bounded(100)
        def budgeted(n) {
            while n > 0 {
                n
            }
            0
        }
        "#,
    );
    let report = bounded_loop_report(&module);
    assert!(
        !report.ok(),
        "@bounded functions must reject uncheckable loops"
    );
    assert_eq!(report.checked_functions, 1);
}
