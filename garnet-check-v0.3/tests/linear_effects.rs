//! S96 tests for the first linear/effect safe-mode seed.
//!
//! This is intentionally a narrow static checker: it does not claim whole-
//! language linear typing, VM enforcement, or OS sandbox enforcement.

use garnet_check::{
    caps_graph, check_module, linear_effect_report, CheckError, LinearParam, Severity,
};
use garnet_parser::{ast::Ownership, parse_source};

fn parse(src: &str) -> garnet_parser::ast::Module {
    parse_source(src).expect("parse")
}

#[test]
fn effectful_safe_helper_without_linear_param_is_fatal() {
    let module = parse(
        r#"
        @caps(fs)
        fn read_secret() -> String {
            fs::read_file("secret.txt")
        }
        "#,
    );

    let report = check_module(&module);
    assert!(
        report.errors.iter().any(|error| matches!(
            error,
            CheckError::LinearEffect(message)
                if message.contains("read_secret")
                    && message.contains("fs")
                    && message.contains("ownership-qualified parameter")
        )),
        "expected an S96 linear/effect violation, got {:?}",
        report.errors
    );
    assert!(!report.ok(), "linear/effect violations must be fatal");
}

#[test]
fn effectful_safe_helper_with_borrow_param_records_boundary_and_passes() {
    let module = parse(
        r#"
        @caps(fs)
        fn read_secret(borrow path: String) -> String {
            fs::read_file(path)
        }
        "#,
    );

    let caps = caps_graph::check_caps_coverage(&module);
    let report = linear_effect_report(&module, &caps);
    assert!(
        report.violations.is_empty(),
        "borrow boundary should satisfy S96 rule, got {:?}",
        report.violations
    );
    let summary = report
        .functions
        .iter()
        .find(|summary| summary.fn_name == "read_secret")
        .expect("read_secret summary");
    assert!(summary.required_caps.contains("fs"));
    assert_eq!(
        summary.linear_params,
        vec![LinearParam {
            name: "path".to_string(),
            ownership: Ownership::Borrow,
        }]
    );
}

#[test]
fn pure_safe_helper_without_linear_param_remains_allowed() {
    let module = parse(
        r#"
        fn answer() -> Int {
            42
        }
        "#,
    );

    let caps = caps_graph::check_caps_coverage(&module);
    let report = linear_effect_report(&module, &caps);
    assert!(
        report.violations.is_empty(),
        "pure safe helpers should not need an artificial authority boundary: {:?}",
        report.violations
    );
}

#[test]
fn main_is_the_program_authority_boundary_for_s96() {
    let module = parse(
        r#"
        @caps(fs)
        fn main() -> String {
            fs::read_file("secret.txt")
        }
        "#,
    );

    let report = check_module(&module);
    assert!(
        !report
            .errors
            .iter()
            .any(|error| matches!(error, CheckError::LinearEffect(_))),
        "S96 should not reject main as a helper lacking params: {:?}",
        report.errors
    );
}

#[test]
fn linear_effect_error_has_canonical_code_and_severity() {
    let error = CheckError::LinearEffect("linear/effect violation: x".to_string());
    assert_eq!(error.severity(), Severity::Error);
    assert_eq!(error.code(), "check.linear_effect");

    let report = garnet_check::CheckReport {
        errors: vec![error],
        ..Default::default()
    };
    assert!(!report.ok());
}
