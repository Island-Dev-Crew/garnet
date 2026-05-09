use garnet_check::{check_module, CheckError};
use garnet_parser::parse_source;

fn check(src: &str) -> Vec<CheckError> {
    let module = parse_source(src).expect("parse ok");
    check_module(&module).errors
}

fn has_safe_violation(errs: &[CheckError], needle: &str) -> bool {
    errs.iter().any(|err| match err {
        CheckError::SafeModeViolation(message) => message.contains(needle),
        _ => false,
    })
}

#[test]
fn safe_bool_match_requires_true_and_false_without_catch_all() {
    let errs = check(
        r#"
        fn bool_code(flag: Bool) -> Int {
            match flag {
                true => 1
            }
        }
        "#,
    );

    assert!(
        has_safe_violation(&errs, "non-exhaustive match") && has_safe_violation(&errs, "false"),
        "expected missing false diagnostic, got {errs:?}"
    );
}

#[test]
fn safe_enum_match_requires_all_variants_without_catch_all() {
    let errs = check(
        r#"
        enum Status { Ready, Done }

        fn status_code(status: Status) -> Int {
            match status {
                Status::Ready => 1
            }
        }
        "#,
    );

    assert!(
        has_safe_violation(&errs, "non-exhaustive match")
            && has_safe_violation(&errs, "Status::Done"),
        "expected missing enum variant diagnostic, got {errs:?}"
    );
}

#[test]
fn safe_enum_match_accepts_all_variants() {
    let errs = check(
        r#"
        enum Status { Ready, Done }

        fn status_code(status: Status) -> Int {
            match status {
                Status::Ready => 1
                Status::Done => 2
            }
        }
        "#,
    );

    assert!(
        !has_safe_violation(&errs, "non-exhaustive match"),
        "complete enum match should not be rejected, got {errs:?}"
    );
}

#[test]
fn safe_match_uses_local_type_annotations_for_finite_domains() {
    let errs = check(
        r#"
        enum Status { Ready, Done }

        fn status_code() -> Int {
            let status: Status = Status::Ready()
            match status {
                Status::Ready => 1
            }
        }
        "#,
    );

    assert!(
        has_safe_violation(&errs, "non-exhaustive match")
            && has_safe_violation(&errs, "Status::Done"),
        "expected local annotation to drive enum coverage, got {errs:?}"
    );
}

#[test]
fn safe_match_uses_local_bool_initializer_for_finite_domain() {
    let errs = check(
        r#"
        fn bool_code() -> Int {
            let flag = true
            match flag {
                true => 1
            }
        }
        "#,
    );

    assert!(
        has_safe_violation(&errs, "non-exhaustive match") && has_safe_violation(&errs, "false"),
        "expected bool initializer to drive missing false diagnostic, got {errs:?}"
    );
}

#[test]
fn safe_match_uses_local_enum_initializer_for_finite_domain() {
    let errs = check(
        r#"
        enum Status { Ready, Done }

        fn status_code() -> Int {
            let status = Status::Ready()
            match status {
                Status::Ready => 1
            }
        }
        "#,
    );

    assert!(
        has_safe_violation(&errs, "non-exhaustive match")
            && has_safe_violation(&errs, "Status::Done"),
        "expected enum initializer to drive missing variant diagnostic, got {errs:?}"
    );
}

#[test]
fn safe_match_uses_unassigned_mutable_bool_initializer_for_finite_domain() {
    let errs = check(
        r#"
        fn bool_code() -> Int {
            let mut flag = true
            match flag {
                true => 1
            }
        }
        "#,
    );

    assert!(
        has_safe_violation(&errs, "non-exhaustive match") && has_safe_violation(&errs, "false"),
        "expected unassigned mutable bool initializer to drive missing false diagnostic, got {errs:?}"
    );
}

#[test]
fn safe_match_uses_unassigned_mutable_enum_initializer_for_finite_domain() {
    let errs = check(
        r#"
        enum Status { Ready, Done }

        fn status_code() -> Int {
            let mut status = Status::Ready()
            match status {
                Status::Ready => 1
            }
        }
        "#,
    );

    assert!(
        has_safe_violation(&errs, "non-exhaustive match")
            && has_safe_violation(&errs, "Status::Done"),
        "expected unassigned mutable enum initializer to drive missing variant diagnostic, got {errs:?}"
    );
}

#[test]
fn safe_match_updates_mutable_domain_after_finite_assignment() {
    let errs = check(
        r#"
        fn bool_code() -> Int {
            let mut flag = 1
            flag = true
            match flag {
                true => 1
            }
        }
        "#,
    );

    assert!(
        has_safe_violation(&errs, "non-exhaustive match") && has_safe_violation(&errs, "false"),
        "expected finite reassignment to drive missing false diagnostic, got {errs:?}"
    );
}

#[test]
fn safe_match_invalidates_mutable_domain_after_non_finite_assignment() {
    let errs = check(
        r#"
        fn bool_code() -> Int {
            let mut flag = true
            flag = 1
            match flag {
                true => 1
            }
        }
        "#,
    );

    assert!(
        !has_safe_violation(&errs, "non-exhaustive match"),
        "non-finite mutable reassignment should clear inferred finite domain, got {errs:?}"
    );
}

#[test]
fn managed_match_non_exhaustiveness_is_not_rejected_by_safe_pass() {
    let errs = check(
        r#"
        def bool_code(flag) {
            match flag {
                true => 1
            }
        }
        "#,
    );

    assert!(
        !has_safe_violation(&errs, "non-exhaustive match"),
        "managed matches should stay outside the safe-mode hard-error gate, got {errs:?}"
    );
}

#[test]
fn guarded_enum_arm_does_not_make_safe_match_exhaustive() {
    let errs = check(
        r#"
        enum Status { Ready, Done }

        fn status_code(status: Status, ok: Bool) -> Int {
            match status {
                Status::Ready if ok => 1
                Status::Done => 2
            }
        }
        "#,
    );

    assert!(
        has_safe_violation(&errs, "non-exhaustive match")
            && has_safe_violation(&errs, "Status::Ready"),
        "guarded arms must not count as exhaustive coverage, got {errs:?}"
    );
}

#[test]
fn true_guarded_enum_arm_counts_as_safe_match_coverage() {
    let errs = check(
        r#"
        enum Status { Ready, Done }

        fn status_code(status: Status) -> Int {
            match status {
                Status::Ready if true => 1
                Status::Done => 2
            }
        }
        "#,
    );

    assert!(
        !has_safe_violation(&errs, "non-exhaustive match")
            && !has_safe_violation(&errs, "unreachable match arm"),
        "literal true guards should count as coverage, got {errs:?}"
    );
}

#[test]
fn false_guarded_enum_arm_is_unreachable_and_not_coverage() {
    let errs = check(
        r#"
        enum Status { Ready, Done }

        fn status_code(status: Status) -> Int {
            match status {
                Status::Ready if false => 1
                Status::Done => 2
            }
        }
        "#,
    );

    assert!(
        has_safe_violation(&errs, "statically false guard")
            && has_safe_violation(&errs, "non-exhaustive match")
            && has_safe_violation(&errs, "Status::Ready"),
        "literal false guards should be unreachable and non-covering, got {errs:?}"
    );
}

#[test]
fn safe_open_literal_match_rejects_duplicate_literal_arm() {
    let errs = check(
        r#"
        fn classify(value: Int) -> Int {
            match value {
                1 => 10
                1 => 11
                _ => 0
            }
        }
        "#,
    );

    assert!(
        has_safe_violation(&errs, "unreachable match arm") && has_safe_violation(&errs, "`1`"),
        "expected duplicate open-domain literal arm to be unreachable, got {errs:?}"
    );
}

#[test]
fn safe_open_match_rejects_arm_after_catch_all() {
    let errs = check(
        r#"
        fn classify(value: Int) -> Int {
            match value {
                _ => 0
                1 => 10
            }
        }
        "#,
    );

    assert!(
        has_safe_violation(&errs, "unreachable match arm")
            && has_safe_violation(&errs, "covered by prior catch-all"),
        "expected open-domain arm after catch-all to be unreachable, got {errs:?}"
    );
}

#[test]
fn safe_open_literal_match_keeps_unknown_guard_non_covering() {
    let errs = check(
        r#"
        fn classify(value: Int, ok: Bool) -> Int {
            match value {
                1 if ok => 10
                1 => 11
                _ => 0
            }
        }
        "#,
    );

    assert!(
        !has_safe_violation(&errs, "unreachable match arm"),
        "unknown guards should not cover open-domain literals, got {errs:?}"
    );
}

#[test]
fn safe_match_rejects_duplicate_covered_arm() {
    let errs = check(
        r#"
        enum Status { Ready, Done }

        fn status_code(status: Status) -> Int {
            match status {
                Status::Ready => 1
                Status::Ready => 2
                Status::Done => 3
            }
        }
        "#,
    );

    assert!(
        has_safe_violation(&errs, "unreachable match arm")
            && has_safe_violation(&errs, "Status::Ready"),
        "expected duplicate variant to be unreachable, got {errs:?}"
    );
}

#[test]
fn safe_match_rejects_arm_after_unguarded_catch_all() {
    let errs = check(
        r#"
        enum Status { Ready, Done }

        fn status_code(status: Status) -> Int {
            match status {
                _ => 0
                Status::Ready => 1
            }
        }
        "#,
    );

    assert!(
        has_safe_violation(&errs, "unreachable match arm")
            && has_safe_violation(&errs, "covered by prior catch-all"),
        "expected arm after wildcard to be unreachable, got {errs:?}"
    );
}

#[test]
fn safe_nested_enum_match_requires_all_finite_payload_cases() {
    let errs = check(
        r#"
        enum Inner { Left, Right }
        enum Outer { Wrap(Inner), Empty }

        fn nested_code(outer: Outer) -> Int {
            match outer {
                Outer::Wrap(Inner::Left) => 1
                Outer::Empty => 0
            }
        }
        "#,
    );

    assert!(
        has_safe_violation(&errs, "non-exhaustive match")
            && has_safe_violation(&errs, "Outer::Wrap(Inner::Right)"),
        "expected missing nested enum payload diagnostic, got {errs:?}"
    );
}

#[test]
fn safe_nested_enum_match_accepts_all_finite_payload_cases() {
    let errs = check(
        r#"
        enum Inner { Left, Right }
        enum Outer { Wrap(Inner), Empty }

        fn nested_code(outer: Outer) -> Int {
            match outer {
                Outer::Wrap(Inner::Left) => 1
                Outer::Wrap(Inner::Right) => 2
                Outer::Empty => 0
            }
        }
        "#,
    );

    assert!(
        !has_safe_violation(&errs, "non-exhaustive match")
            && !has_safe_violation(&errs, "unreachable match arm"),
        "complete nested enum match should not be rejected, got {errs:?}"
    );
}

#[test]
fn safe_nested_enum_match_allows_payload_wildcard_coverage() {
    let errs = check(
        r#"
        enum Inner { Left, Right }
        enum Outer { Wrap(Inner), Empty }

        fn nested_code(outer: Outer) -> Int {
            match outer {
                Outer::Wrap(_) => 1
                Outer::Empty => 0
            }
        }
        "#,
    );

    assert!(
        !has_safe_violation(&errs, "non-exhaustive match"),
        "wildcard payload should cover the nested finite payload domain, got {errs:?}"
    );
}

#[test]
fn safe_imported_named_enum_match_accepts_alias_qualified_arms() {
    let errs = check(
        r#"
        module Types {
            enum Status { Ready, Done }
        }

        use Types::{Status}

        fn status_code(status: Status) -> Int {
            match status {
                Status::Ready => 1
                Status::Done => 2
            }
        }
        "#,
    );

    assert!(
        !has_safe_violation(&errs, "non-exhaustive match")
            && !has_safe_violation(&errs, "unreachable match arm"),
        "complete imported enum match should not be rejected, got {errs:?}"
    );
}

#[test]
fn safe_imported_named_enum_match_rejects_missing_alias_case() {
    let errs = check(
        r#"
        module Types {
            enum Status { Ready, Done }
        }

        use Types::{Status}

        fn status_code(status: Status) -> Int {
            match status {
                Status::Ready => 1
            }
        }
        "#,
    );

    assert!(
        has_safe_violation(&errs, "non-exhaustive match")
            && has_safe_violation(&errs, "Types::Status::Done")
            && !has_safe_violation(&errs, "Types::Status::Ready"),
        "expected imported missing case only, got {errs:?}"
    );
}

#[test]
fn safe_imported_glob_enum_match_accepts_alias_qualified_arms() {
    let errs = check(
        r#"
        module Types {
            enum Status { Ready, Done }
        }

        use Types::*

        fn status_code(status: Status) -> Int {
            match status {
                Status::Ready => 1
                Status::Done => 2
            }
        }
        "#,
    );

    assert!(
        !has_safe_violation(&errs, "non-exhaustive match")
            && !has_safe_violation(&errs, "unreachable match arm"),
        "complete glob-imported enum match should not be rejected, got {errs:?}"
    );
}

#[test]
fn safe_imported_module_alias_enum_match_accepts_module_qualified_arms() {
    let errs = check(
        r#"
        module Library {
            module Types {
                enum Status { Ready, Done }
            }
        }

        use Library::Types

        fn status_code(status: Types::Status) -> Int {
            match status {
                Types::Status::Ready => 1
                Types::Status::Done => 2
            }
        }
        "#,
    );

    assert!(
        !has_safe_violation(&errs, "non-exhaustive match")
            && !has_safe_violation(&errs, "unreachable match arm"),
        "complete module-alias enum match should not be rejected, got {errs:?}"
    );
}

#[test]
fn safe_imported_nested_module_relative_enum_match_accepts_alias_qualified_arms() {
    let errs = check(
        r#"
        module App {
            module Types {
                enum Status { Ready, Done }
            }

            use Types::{Status}

            fn status_code(status: Status) -> Int {
                match status {
                    Status::Ready => 1
                    Status::Done => 2
                }
            }
        }
        "#,
    );

    assert!(
        !has_safe_violation(&errs, "non-exhaustive match")
            && !has_safe_violation(&errs, "unreachable match arm"),
        "complete nested relative imported enum match should not be rejected, got {errs:?}"
    );
}
