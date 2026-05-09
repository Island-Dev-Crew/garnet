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
fn safe_match_invalidates_mutable_domain_after_compound_assignment() {
    let errs = check(
        r#"
        fn bool_code() -> Int {
            let mut flag = true
            flag += 1
            match flag {
                true => 1
            }
        }
        "#,
    );

    assert!(
        !has_safe_violation(&errs, "non-exhaustive match"),
        "compound assignment should clear inferred finite domain, got {errs:?}"
    );
}

#[test]
fn safe_match_joins_bool_domain_after_if_else_assignments() {
    let errs = check(
        r#"
        fn bool_code(cond: Bool) -> Int {
            let mut flag = 1
            if cond {
                flag = true
            } else {
                flag = false
            }
            match flag {
                true => 1
            }
        }
        "#,
    );

    assert!(
        has_safe_violation(&errs, "non-exhaustive match") && has_safe_violation(&errs, "false"),
        "expected if/else Bool assignments to join into a finite-domain diagnostic, got {errs:?}"
    );
}

#[test]
fn safe_match_invalidates_domain_after_mixed_if_else_assignments() {
    let errs = check(
        r#"
        fn bool_code(cond: Bool) -> Int {
            let mut flag = true
            if cond {
                flag = false
            } else {
                flag = 1
            }
            match flag {
                true => 1
            }
        }
        "#,
    );

    assert!(
        !has_safe_violation(&errs, "non-exhaustive match"),
        "mixed finite/non-finite branch assignments should clear inferred domain, got {errs:?}"
    );
}

#[test]
fn safe_match_invalidates_domain_after_if_else_compound_assignments() {
    let errs = check(
        r#"
        fn bool_code(cond: Bool) -> Int {
            let mut flag = true
            if cond {
                flag += 1
            } else {
                flag += 1
            }
            match flag {
                true => 1
            }
        }
        "#,
    );

    assert!(
        !has_safe_violation(&errs, "non-exhaustive match"),
        "compound assignments in all branches should clear inferred domain, got {errs:?}"
    );
}

#[test]
fn safe_match_invalidates_domain_after_while_assignment() {
    let errs = check(
        r#"
        fn bool_code(cond: Bool) -> Int {
            let mut flag = true
            while cond {
                flag = 1
            }
            match flag {
                true => 1
            }
        }
        "#,
    );

    assert!(
        !has_safe_violation(&errs, "non-exhaustive match"),
        "assignment in a possible while iteration should clear inferred finite domain, got {errs:?}"
    );
}

#[test]
fn safe_match_invalidates_domain_after_for_assignment() {
    let errs = check(
        r#"
        fn bool_code() -> Int {
            let mut flag = true
            for item in [1] {
                flag = item
            }
            match flag {
                true => 1
            }
        }
        "#,
    );

    assert!(
        !has_safe_violation(&errs, "non-exhaustive match"),
        "assignment in a possible for iteration should clear inferred finite domain, got {errs:?}"
    );
}

#[test]
fn safe_match_invalidates_domain_after_conditional_loop_assignment() {
    let errs = check(
        r#"
        fn bool_code(first: Bool, second: Bool) -> Int {
            let mut flag = true
            while first {
                if second {
                    flag = 1
                }
            }
            match flag {
                true => 1
            }
        }
        "#,
    );

    assert!(
        !has_safe_violation(&errs, "non-exhaustive match"),
        "conditional assignment in a loop body should clear inferred finite domain, got {errs:?}"
    );
}

#[test]
fn safe_match_invalidates_loop_assignment_before_shadowing_binding() {
    let errs = check(
        r#"
        fn bool_code(cond: Bool) -> Int {
            let mut flag = true
            while cond {
                flag = 1
                let flag = true
            }
            match flag {
                true => 1
            }
        }
        "#,
    );

    assert!(
        !has_safe_violation(&errs, "non-exhaustive match"),
        "assignment before a loop-local shadow should still clear the outer finite domain, got {errs:?}"
    );
}

#[test]
fn safe_match_does_not_invalidate_outer_domain_for_loop_local_binding() {
    let errs = check(
        r#"
        fn bool_code(cond: Bool) -> Int {
            let mut flag = true
            while cond {
                let flag = 1
            }
            match flag {
                true => 1
            }
        }
        "#,
    );

    assert!(
        has_safe_violation(&errs, "non-exhaustive match") && has_safe_violation(&errs, "false"),
        "loop-local bindings should not clear the outer finite domain, got {errs:?}"
    );
}

#[test]
fn safe_match_invalidates_domain_after_try_body_assignment() {
    let errs = check(
        r#"
        fn bool_code() -> Int {
            let mut flag = true
            try {
                flag = 1
            } rescue e {
                0
            }
            match flag {
                true => 1
            }
        }
        "#,
    );

    assert!(
        !has_safe_violation(&errs, "non-exhaustive match"),
        "assignment in a possible try body should clear inferred finite domain, got {errs:?}"
    );
}

#[test]
fn safe_match_invalidates_domain_after_try_rescue_assignment() {
    let errs = check(
        r#"
        fn bool_code() -> Int {
            let mut flag = true
            try {
                0
            } rescue e {
                flag = 1
            }
            match flag {
                true => 1
            }
        }
        "#,
    );

    assert!(
        !has_safe_violation(&errs, "non-exhaustive match"),
        "assignment in a possible rescue body should clear inferred finite domain, got {errs:?}"
    );
}

#[test]
fn safe_match_invalidates_domain_after_try_ensure_assignment() {
    let errs = check(
        r#"
        fn bool_code() -> Int {
            let mut flag = true
            try {
                0
            } ensure {
                flag = 1
            }
            match flag {
                true => 1
            }
        }
        "#,
    );

    assert!(
        !has_safe_violation(&errs, "non-exhaustive match"),
        "assignment in an ensure body should clear inferred finite domain, got {errs:?}"
    );
}

#[test]
fn safe_match_does_not_merge_uninvoked_closure_assignment_domains() {
    let errs = check(
        r#"
        fn bool_code(cond: Bool) -> Int {
            let mut flag = 1
            let updater = |value| if cond {
                flag = true
            } else {
                flag = false
            }
            match flag {
                true => 1
            }
        }
        "#,
    );

    assert!(
        !has_safe_violation(&errs, "non-exhaustive match"),
        "uncalled closure body assignments should not merge into the surrounding finite domain, got {errs:?}"
    );
}

#[test]
fn safe_match_invalidates_domain_after_immediate_closure_block_assignment() {
    let errs = check(
        r#"
        fn bool_code() -> Int {
            let mut flag = true
            (|value| {
                flag = value
            })(1)
            match flag {
                true => 1
            }
        }
        "#,
    );

    assert!(
        !has_safe_violation(&errs, "non-exhaustive match"),
        "immediately invoked closure block assignment should clear inferred finite domain, got {errs:?}"
    );
}

#[test]
fn safe_match_invalidates_domain_after_immediate_closure_expr_assignment() {
    let errs = check(
        r#"
        fn bool_code(cond: Bool) -> Int {
            let mut flag = true
            (|value| if cond {
                flag = value
            } else {
                flag = false
            })(1)
            match flag {
                true => 1
            }
        }
        "#,
    );

    assert!(
        !has_safe_violation(&errs, "non-exhaustive match"),
        "immediately invoked closure expression assignments should clear inferred finite domain, got {errs:?}"
    );
}

#[test]
fn safe_match_invalidates_domain_after_local_closure_call_assignment() {
    let errs = check(
        r#"
        fn bool_code() -> Int {
            let mut flag = true
            let updater = |value| {
                flag = value
            }
            updater(1)
            match flag {
                true => 1
            }
        }
        "#,
    );

    assert!(
        !has_safe_violation(&errs, "non-exhaustive match"),
        "local closure call assignment should clear inferred finite domain, got {errs:?}"
    );
}

#[test]
fn safe_match_invalidates_domain_after_branch_joined_local_closure_call_assignment() {
    let errs = check(
        r#"
        fn bool_code(cond: Bool) -> Int {
            let mut flag = true
            let updater = if cond {
                |value| {
                    flag = value
                }
            } else {
                |value| {
                    flag = false
                }
            }
            updater(1)
            match flag {
                true => 1
            }
        }
        "#,
    );

    assert!(
        !has_safe_violation(&errs, "non-exhaustive match"),
        "branch-joined local closure call assignment should clear inferred finite domain, got {errs:?}"
    );
}

#[test]
fn safe_match_invalidates_domain_after_branch_rebound_local_closure_call_assignment() {
    let errs = check(
        r#"
        fn bool_code(cond: Bool) -> Int {
            let mut flag = true
            let mut updater = |value| {
                value
            }
            if cond {
                updater = |value| {
                    flag = value
                }
            } else {
                updater = |value| {
                    flag = false
                }
            }
            updater(1)
            match flag {
                true => 1
            }
        }
        "#,
    );

    assert!(
        !has_safe_violation(&errs, "non-exhaustive match"),
        "branch-rebound local closure call assignment should clear inferred finite domain, got {errs:?}"
    );
}

#[test]
fn safe_match_invalidates_domain_after_local_closure_alias_call_assignment() {
    let errs = check(
        r#"
        fn bool_code() -> Int {
            let mut flag = true
            let updater = |value| {
                flag = value
            }
            let alias = updater
            alias(1)
            match flag {
                true => 1
            }
        }
        "#,
    );

    assert!(
        !has_safe_violation(&errs, "non-exhaustive match"),
        "local closure alias call assignment should clear inferred finite domain, got {errs:?}"
    );
}

#[test]
fn safe_match_invalidates_domain_after_branch_joined_local_closure_alias_call_assignment() {
    let errs = check(
        r#"
        fn bool_code(cond: Bool) -> Int {
            let mut flag = true
            let update_from_arg = |value| {
                flag = value
            }
            let update_to_false = |value| {
                flag = false
            }
            let alias = if cond {
                update_from_arg
            } else {
                update_to_false
            }
            alias(1)
            match flag {
                true => 1
            }
        }
        "#,
    );

    assert!(
        !has_safe_violation(&errs, "non-exhaustive match"),
        "branch-joined local closure alias call assignment should clear inferred finite domain, got {errs:?}"
    );
}

#[test]
fn safe_match_invalidates_domain_after_direct_branch_selected_closure_call_assignment() {
    let errs = check(
        r#"
        fn bool_code(cond: Bool) -> Int {
            let mut flag = true
            let update_from_arg = |value| {
                flag = value
            }
            let update_to_false = |value| {
                flag = false
            }
            (if cond {
                update_from_arg
            } else {
                update_to_false
            })(1)
            match flag {
                true => 1
            }
        }
        "#,
    );

    assert!(
        !has_safe_violation(&errs, "non-exhaustive match"),
        "direct branch-selected closure call assignment should clear inferred finite domain, got {errs:?}"
    );
}

#[test]
fn safe_match_keeps_domain_when_branch_alias_tail_shadows_known_closure() {
    let errs = check(
        r#"
        fn bool_code(cond: Bool) -> Int {
            let mut flag = true
            let updater = |value| {
                flag = value
            }
            let alias = if cond {
                let updater = 1
                updater
            } else {
                updater
            }
            alias(1)
            match flag {
                true => 1
            }
        }
        "#,
    );

    assert!(
        has_safe_violation(&errs, "non-exhaustive match"),
        "shadowed branch alias tail should stay unknown and preserve finite domain, got {errs:?}"
    );
}

#[test]
fn safe_match_keeps_domain_when_direct_branch_call_tail_shadows_known_closure() {
    let errs = check(
        r#"
        fn bool_code(cond: Bool) -> Int {
            let mut flag = true
            let updater = |value| {
                flag = value
            }
            (if cond {
                let updater = 1
                updater
            } else {
                updater
            })(1)
            match flag {
                true => 1
            }
        }
        "#,
    );

    assert!(
        has_safe_violation(&errs, "non-exhaustive match"),
        "shadowed direct branch call tail should stay unknown and preserve finite domain, got {errs:?}"
    );
}

#[test]
fn safe_match_joins_branch_assignment_before_shadowing_binding() {
    let errs = check(
        r#"
        fn bool_code(cond: Bool) -> Int {
            let mut flag = true
            if cond {
                flag = false
                let flag = true
            } else {
                flag = false
            }
            match flag {
                true => 1
            }
        }
        "#,
    );

    assert!(
        has_safe_violation(&errs, "non-exhaustive match") && has_safe_violation(&errs, "false"),
        "assignment before a branch-local shadow should still join the outer finite domain, got {errs:?}"
    );
}

#[test]
fn safe_match_does_not_infer_branch_assignment_without_else_path() {
    let errs = check(
        r#"
        fn bool_code(cond: Bool) -> Int {
            let mut flag = 1
            if cond {
                flag = true
            }
            match flag {
                true => 1
            }
        }
        "#,
    );

    assert!(
        !has_safe_violation(&errs, "non-exhaustive match"),
        "finite assignment on only one possible branch should not infer a closed domain, got {errs:?}"
    );
}

#[test]
fn safe_match_joins_bool_domain_after_elsif_assignments() {
    let errs = check(
        r#"
        fn bool_code(first: Bool, second: Bool) -> Int {
            let mut flag = 1
            if first {
                flag = true
            } elsif second {
                flag = false
            } else {
                flag = true
            }
            match flag {
                true => 1
            }
        }
        "#,
    );

    assert!(
        has_safe_violation(&errs, "non-exhaustive match") && has_safe_violation(&errs, "false"),
        "expected if/elsif/else Bool assignments to join into a finite-domain diagnostic, got {errs:?}"
    );
}

#[test]
fn safe_match_does_not_join_branch_local_bindings_after_if_else() {
    let errs = check(
        r#"
        fn bool_code(cond: Bool) -> Int {
            if cond {
                let flag = true
            } else {
                let flag = false
            }
            match flag {
                true => 1
            }
        }
        "#,
    );

    assert!(
        !has_safe_violation(&errs, "non-exhaustive match"),
        "branch-local bindings should not seed a post-if finite domain, got {errs:?}"
    );
}

#[test]
fn safe_match_joins_nested_if_assignment_domains() {
    let errs = check(
        r#"
        fn bool_code(first: Bool, second: Bool) -> Int {
            let mut flag = 1
            if first {
                if second {
                    flag = true
                } else {
                    flag = false
                }
            } else {
                flag = true
            }
            match flag {
                true => 1
            }
        }
        "#,
    );

    assert!(
        has_safe_violation(&errs, "non-exhaustive match") && has_safe_violation(&errs, "false"),
        "expected nested if/else assignments to join into a finite-domain diagnostic, got {errs:?}"
    );
}

#[test]
fn safe_match_does_not_join_nested_if_assignment_without_else() {
    let errs = check(
        r#"
        fn bool_code(first: Bool, second: Bool) -> Int {
            let mut flag = 1
            if first {
                if second {
                    flag = true
                }
            } else {
                flag = false
            }
            match flag {
                true => 1
            }
        }
        "#,
    );

    assert!(
        !has_safe_violation(&errs, "non-exhaustive match"),
        "nested if assignment with a missing else path should not infer a closed domain, got {errs:?}"
    );
}

#[test]
fn safe_match_joins_enum_domain_after_if_else_assignments() {
    let errs = check(
        r#"
        enum Status { Ready, Done }

        fn status_code(cond: Bool) -> Int {
            let mut status = 1
            if cond {
                status = Status::Ready()
            } else {
                status = Status::Done()
            }
            match status {
                Status::Ready => 1
            }
        }
        "#,
    );

    assert!(
        has_safe_violation(&errs, "non-exhaustive match")
            && has_safe_violation(&errs, "Status::Done"),
        "expected if/else enum assignments to join into a finite-domain diagnostic, got {errs:?}"
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
fn true_local_bool_guard_counts_as_safe_match_coverage() {
    let errs = check(
        r#"
        enum Status { Ready, Done }

        fn status_code(status: Status) -> Int {
            let always = true
            match status {
                Status::Ready if always => 1
                Status::Done => 2
            }
        }
        "#,
    );

    assert!(
        !has_safe_violation(&errs, "non-exhaustive match")
            && !has_safe_violation(&errs, "unreachable match arm"),
        "local true guards should count as coverage, got {errs:?}"
    );
}

#[test]
fn mutable_bool_guard_does_not_count_as_static_match_coverage() {
    let errs = check(
        r#"
        enum Status { Ready, Done }

        fn status_code(status: Status) -> Int {
            let mut always = true
            match status {
                Status::Ready if always => 1
                Status::Done => 2
            }
        }
        "#,
    );

    assert!(
        has_safe_violation(&errs, "non-exhaustive match")
            && has_safe_violation(&errs, "Status::Ready"),
        "mutable guard facts must stay unknown, got {errs:?}"
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
fn false_local_bool_guard_is_unreachable_and_not_coverage() {
    let errs = check(
        r#"
        enum Status { Ready, Done }

        fn status_code(status: Status) -> Int {
            let never = false
            match status {
                Status::Ready if never => 1
                Status::Done => 2
            }
        }
        "#,
    );

    assert!(
        has_safe_violation(&errs, "statically false guard")
            && has_safe_violation(&errs, "non-exhaustive match")
            && has_safe_violation(&errs, "Status::Ready"),
        "local false guards should be unreachable and non-covering, got {errs:?}"
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
