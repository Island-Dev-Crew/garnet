//! D-107 — enum variant construction is checked at `garnet check` time in safe
//! functions.
//!
//! Before this cure the checker accepted `Status::Missing`, `Status::Ready(1)`,
//! a payload variant used with no payload, and a payload variant given the
//! wrong number of fields. The interpreter's `eval_path` builds a bare
//! variant for any name that exists and the `Call` wrapper fills in whatever
//! arguments arrive, so every one of these shapes either ran with a
//! malformed value or failed only at run time. Safe mode fails closed: the
//! four probe shapes below must be `check.safe_mode_violation` errors, and
//! the well-formed controls must stay green.
//!
//! Scope (disclosed): the walk runs where match coverage runs — safe
//! functions (`fn`, `@safe`, or a `safe module`). Managed `def` bodies are not
//! walked here, matching the existing safe-mode contract of this checker.

use garnet_check::{check_module, CheckError};
use garnet_parser::parse_source;

fn check(src: &str) -> Vec<CheckError> {
    let module = parse_source(src).expect("parse ok");
    check_module(&module).errors
}

fn safe_violations(errs: &[CheckError]) -> Vec<&str> {
    errs.iter()
        .filter_map(|err| match err {
            CheckError::SafeModeViolation(message) => Some(message.as_str()),
            _ => None,
        })
        .collect()
}

fn has_safe_violation(errs: &[CheckError], needle: &str) -> bool {
    safe_violations(errs).iter().any(|m| m.contains(needle))
}

const SHAPES: &str = r#"
    enum Shape { Circle(Float), Rect(Float, Float), Empty }
"#;

/// q1 — a unit variant is given a payload.
#[test]
fn safe_unit_variant_given_payload_is_rejected() {
    let errs = check(&format!(
        "{SHAPES}
        fn make() -> Shape {{
            Shape::Empty(1.0)
        }}
        "
    ));
    assert!(
        has_safe_violation(&errs, "unit variant `Shape::Empty` takes no payload")
            && has_safe_violation(&errs, "given 1")
            && has_safe_violation(&errs, "safe function 'make'"),
        "expected unit-variant payload diagnostic, got {errs:?}"
    );
}

/// q2 — a payload variant is used bare, with no payload at all.
#[test]
fn safe_payload_variant_used_without_payload_is_rejected() {
    let errs = check(&format!(
        "{SHAPES}
        fn make() -> Shape {{
            let s = Shape::Circle
            s
        }}
        "
    ));
    assert!(
        has_safe_violation(
            &errs,
            "payload variant `Shape::Circle` requires 1 field but is used without a payload"
        ),
        "expected bare payload-variant diagnostic, got {errs:?}"
    );
}

/// q3 — a payload variant is given the wrong number of fields.
#[test]
fn safe_payload_variant_arity_mismatch_is_rejected() {
    let errs = check(&format!(
        "{SHAPES}
        fn make() -> Shape {{
            Shape::Circle(1.0, 2.0)
        }}
        "
    ));
    assert!(
        has_safe_violation(
            &errs,
            "payload variant `Shape::Circle` expects 1 field, given 2"
        ),
        "expected arity diagnostic, got {errs:?}"
    );

    let errs = check(&format!(
        "{SHAPES}
        fn make() -> Shape {{
            Shape::Rect(1.0)
        }}
        "
    ));
    assert!(
        has_safe_violation(
            &errs,
            "payload variant `Shape::Rect` expects 2 fields, given 1"
        ),
        "expected arity diagnostic for Rect, got {errs:?}"
    );
}

/// q7 — the variant does not exist on the enum.
#[test]
fn safe_unknown_variant_is_rejected() {
    let errs = check(&format!(
        "{SHAPES}
        fn make() -> Shape {{
            Shape::Triangle(1.0)
        }}
        fn flag() -> Shape {{
            Shape::Nowhere
        }}
        "
    ));
    assert!(
        has_safe_violation(&errs, "enum `Shape` has no variant `Triangle`")
            && has_safe_violation(&errs, "enum `Shape` has no variant `Nowhere`"),
        "expected unknown-variant diagnostics, got {errs:?}"
    );
}

/// Control — every well-formed construction stays green, including an
/// impl-associated function reached through the enum path.
#[test]
fn safe_well_formed_constructions_and_impl_fns_are_accepted() {
    let errs = check(&format!(
        "{SHAPES}
        impl Shape {{
            fn unit_circle() -> Shape {{ Shape::Circle(1.0) }}
        }}
        fn make() -> Shape {{
            let a = Shape::Circle(2.0)
            let b = Shape::Rect(1.0, 2.0)
            let c = Shape::Empty
            let d = Shape::unit_circle()
            d
        }}
        "
    ));
    assert!(
        safe_violations(&errs).is_empty(),
        "well-formed constructions must not be rejected, got {errs:?}"
    );
}

/// Control — the enum path resolves through modules and `use` aliases the same
/// way match arms do; a wrong construction through the alias is still caught.
#[test]
fn safe_module_qualified_and_aliased_constructions_are_checked() {
    let errs = check(
        r#"
        module Geo {
            enum Shape { Circle(Float), Empty }
        }

        use Geo::Shape

        fn ok() -> Shape {
            let a = Geo::Shape::Circle(1.0)
            Shape::Empty
        }

        fn bad() -> Shape {
            Shape::Circle()
        }
        "#,
    );
    let violations = safe_violations(&errs);
    assert_eq!(
        violations.len(),
        1,
        "exactly the aliased mis-construction must be rejected, got {errs:?}"
    );
    assert!(
        violations[0].contains("payload variant `Shape::Circle` expects 1 field, given 0")
            && violations[0].contains("safe function 'bad'"),
        "got {violations:?}"
    );
}

/// Scope pin — managed `def` bodies are outside the safe-mode walk, so this
/// check does not fire there. The pin keeps the disclosed scope honest; widening
/// it is a deliberate contract change, not drift.
#[test]
fn managed_def_bodies_are_outside_the_safe_mode_walk() {
    let errs = check(&format!(
        "{SHAPES}
        def make() -> Shape {{
            Shape::Triangle(1.0)
        }}
        "
    ));
    assert!(
        safe_violations(&errs).is_empty(),
        "managed bodies are not walked by the safe-mode checker, got {errs:?}"
    );
}

/// D-107b (cross-family review of 4ce90eb6) — an `impl` on one enum must not
/// vouch for a missing variant on a *different* enum that happens to share its
/// last path segment. The impl exemption is keyed by the resolved enum path,
/// so `Target::Shape::ghost()` is rejected while `Other::Shape::ghost()` (the
/// enum that actually owns the associated function) stays accepted.
#[test]
fn safe_impl_on_a_same_basename_enum_does_not_vouch_for_another_enum() {
    let errs = check(
        r#"
        module Other {
            enum Shape { Circle(Float), Square(Float) }
            impl Shape {
                fn ghost() -> Shape { Shape::Circle(1.0) }
            }
        }
        module Target {
            enum Shape { Circle(Float), Square(Float) }
        }

        fn owned() -> Other::Shape {
            Other::Shape::ghost()
        }

        fn borrowed() -> Target::Shape {
            Target::Shape::ghost()
        }
        "#,
    );
    let violations = safe_violations(&errs);
    assert_eq!(
        violations.len(),
        1,
        "exactly the cross-enum construction must be rejected, got {errs:?}"
    );
    assert!(
        violations[0].contains("enum `Shape` has no variant `ghost`")
            && violations[0].contains("safe function 'borrowed'")
            && violations[0].contains("`Target::Shape::ghost` does not exist"),
        "got {violations:?}"
    );
}
