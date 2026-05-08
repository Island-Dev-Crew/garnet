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
