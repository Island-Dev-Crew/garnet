//! Conservative trait-coherence checks for Mini-Spec §11.5.

use garnet_check::{check_module, CheckError};
use garnet_parser::parse_source;

fn diagnose(src: &str) -> Vec<CheckError> {
    let m = parse_source(src).expect("parse");
    check_module(&m).errors
}

#[test]
fn duplicate_trait_impl_rejected() {
    let src = r#"
        trait Display {
            def render() -> String
        }

        struct Widget {
            name: String,
        }

        impl Widget for Display {
            def render() -> String { "one" }
        }

        impl Widget for Display {
            def render() -> String { "two" }
        }
    "#;
    let d = diagnose(src);
    assert!(
        d.iter().any(|e| matches!(
            e,
            CheckError::SafeModeViolation(m)
                if m.contains("trait coherence violation") && m.contains("duplicate impl")
        )),
        "duplicate trait impl must be rejected, got {d:?}"
    );
}

#[test]
fn orphan_trait_impl_rejected_when_trait_and_type_are_external() {
    let src = r#"
        impl ExternalWidget for ExternalDisplay {
            def render() -> String { "external" }
        }
    "#;
    let d = diagnose(src);
    assert!(
        d.iter().any(|e| matches!(
            e,
            CheckError::SafeModeViolation(m)
                if m.contains("trait coherence violation") && m.contains("orphan rule")
        )),
        "orphan trait impl must be rejected, got {d:?}"
    );
}

#[test]
fn local_trait_allows_external_type_impl() {
    let src = r#"
        trait LocalDisplay {
            def render() -> String
        }

        impl ExternalWidget for LocalDisplay {
            def render() -> String { "external" }
        }
    "#;
    let d = diagnose(src);
    assert!(
        !d.iter().any(|e| matches!(
            e,
            CheckError::SafeModeViolation(m) if m.contains("trait coherence violation")
        )),
        "local trait should satisfy orphan rule, got {d:?}"
    );
}

#[test]
fn local_type_allows_external_trait_impl() {
    let src = r#"
        struct LocalWidget {
            name: String,
        }

        impl LocalWidget for ExternalDisplay {
            def render() -> String { "local" }
        }
    "#;
    let d = diagnose(src);
    assert!(
        !d.iter().any(|e| matches!(
            e,
            CheckError::SafeModeViolation(m) if m.contains("trait coherence violation")
        )),
        "local type should satisfy orphan rule, got {d:?}"
    );
}

#[test]
fn generic_blanket_impl_overlaps_concrete_impl() {
    let src = r#"
        trait Renderable {
            def render() -> String
        }

        struct Box<T> {
            value: T,
        }

        impl<T> Box<T> for Renderable {
            def render() -> String { "generic" }
        }

        impl Box<String> for Renderable {
            def render() -> String { "string" }
        }
    "#;
    let d = diagnose(src);
    assert!(
        d.iter().any(|e| matches!(
            e,
            CheckError::SafeModeViolation(m)
                if m.contains("trait coherence violation") && m.contains("overlapping impl")
        )),
        "generic blanket impl must overlap concrete impl, got {d:?}"
    );
}

#[test]
fn renamed_generic_blanket_impls_overlap() {
    let src = r#"
        trait Renderable {
            def render() -> String
        }

        struct Box<T> {
            value: T,
        }

        impl<T> Box<T> for Renderable {
            def render() -> String { "left" }
        }

        impl<U> Box<U> for Renderable {
            def render() -> String { "right" }
        }
    "#;
    let d = diagnose(src);
    assert!(
        d.iter().any(|e| matches!(
            e,
            CheckError::SafeModeViolation(m)
                if m.contains("trait coherence violation") && m.contains("overlapping impl")
        )),
        "renamed generic blanket impls must overlap, got {d:?}"
    );
}

#[test]
fn qualified_external_type_does_not_satisfy_orphan_rule_by_short_name() {
    let src = r#"
        struct Widget {
            name: String,
        }

        impl Remote::Widget for ExternalRenderable {
            def render() -> String { "remote" }
        }
    "#;
    let d = diagnose(src);
    assert!(
        d.iter().any(|e| matches!(
            e,
            CheckError::SafeModeViolation(m)
                if m.contains("trait coherence violation") && m.contains("orphan rule")
        )),
        "qualified external type must not be local by short-name collision, got {d:?}"
    );
}

#[test]
fn qualified_local_module_type_satisfies_orphan_rule() {
    let src = r#"
        module LocalPkg {
            struct Widget {
                name: String,
            }
        }

        impl LocalPkg::Widget for ExternalRenderable {
            def render() -> String { "local" }
        }
    "#;
    let d = diagnose(src);
    assert!(
        !d.iter().any(|e| matches!(
            e,
            CheckError::SafeModeViolation(m) if m.contains("trait coherence violation")
        )),
        "qualified local module type should satisfy orphan rule, got {d:?}"
    );
}
