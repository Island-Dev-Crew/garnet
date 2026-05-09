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
