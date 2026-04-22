//! CapCaps tests (v3.4 Security Layer 2).
//!
//! Closes the ambient-authority class: a program MUST declare the OS
//! authority it requires via `@caps(...)` on `main`, and that set must
//! transitively cover every primitive it calls. v3.4.0 implements the
//! declaration + validation layer; full call-graph propagation lands
//! in v3.4.x once stdlib primitives are gated.

use garnet_check::{check_module, CheckError};
use garnet_parser::parse_source;

fn check(src: &str) -> Vec<CheckError> {
    let module = parse_source(src).expect("parse ok");
    check_module(&module).errors
}

fn has_annotation_error(errs: &[CheckError], substring: &str) -> bool {
    errs.iter().any(|e| match e {
        CheckError::AnnotationError(m) => m.contains(substring),
        _ => false,
    })
}

// ─── main MUST declare caps ───

#[test]
fn main_without_caps_is_rejected() {
    let src = r#"
        def main() {
            42
        }
    "#;
    let errs = check(src);
    assert!(
        has_annotation_error(&errs, "`main` function must declare its required capabilities"),
        "expected missing-caps error on main, got: {errs:?}"
    );
}

#[test]
fn main_with_empty_caps_is_accepted() {
    let src = r#"
        @caps()
        def main() {
            42
        }
    "#;
    let errs = check(src);
    assert!(
        !has_annotation_error(&errs, "main` function must declare"),
        "unexpected missing-caps error when @caps() is present: {errs:?}"
    );
}

#[test]
fn main_with_fs_caps_is_accepted() {
    let src = r#"
        @caps(fs)
        def main() {
            42
        }
    "#;
    let errs = check(src);
    assert!(
        !has_annotation_error(&errs, "main` function must declare"),
        "unexpected missing-caps error: {errs:?}"
    );
}

#[test]
fn main_with_multiple_caps_is_accepted() {
    let src = r#"
        @caps(fs, net, time)
        def main() {
            42
        }
    "#;
    let errs = check(src);
    assert!(
        !has_annotation_error(&errs, "main` function must declare"),
        "unexpected missing-caps error with multi-cap list: {errs:?}"
    );
}

// ─── Unknown cap identifiers are rejected ───

#[test]
fn unknown_cap_is_rejected() {
    let src = r#"
        @caps(fs, bogus_cap)
        def main() {
            42
        }
    "#;
    let errs = check(src);
    assert!(
        has_annotation_error(&errs, "unknown capability 'bogus_cap'"),
        "expected unknown-cap error, got: {errs:?}"
    );
}

// ─── Duplicate @caps annotations are rejected ───

#[test]
fn duplicate_caps_annotation_is_rejected() {
    let src = r#"
        @caps(fs)
        @caps(net)
        def main() {
            42
        }
    "#;
    let errs = check(src);
    assert!(
        has_annotation_error(&errs, "multiple @caps(...) annotations"),
        "expected duplicate-caps error, got: {errs:?}"
    );
}

// ─── Non-main functions may declare caps ───

#[test]
fn non_main_function_with_caps_is_accepted() {
    let src = r#"
        @caps(fs)
        def read_config(path) {
            path
        }

        @caps(fs)
        def main() {
            read_config("config.toml")
        }
    "#;
    let errs = check(src);
    assert!(
        !has_annotation_error(&errs, "unknown capability"),
        "unexpected unknown-cap error: {errs:?}"
    );
    assert!(
        !has_annotation_error(&errs, "main` function must declare"),
        "unexpected main-missing-caps: {errs:?}"
    );
}

// ─── Wildcard in safe mode is rejected ───

#[test]
fn wildcard_caps_in_safe_function_is_rejected() {
    let src = r#"
        @safe
        module FastPath {
            @caps(*)
            fn go() -> Int {
                42
            }
        }
    "#;
    let errs = check(src);
    assert!(
        has_annotation_error(&errs, "may not use @caps(*) wildcard"),
        "expected wildcard rejection in safe mode, got: {errs:?}"
    );
}

// ─── Out-of-bounds @mailbox is rejected ───

#[test]
fn mailbox_zero_is_rejected() {
    let src = r#"
        @mailbox(0)
        @caps()
        def main() { 0 }
    "#;
    let errs = check(src);
    assert!(
        has_annotation_error(&errs, "@mailbox"),
        "expected @mailbox(0) rejection, got: {errs:?}"
    );
}

#[test]
fn mailbox_2m_is_rejected() {
    let src = r#"
        @mailbox(2000000)
        @caps()
        def main() { 0 }
    "#;
    let errs = check(src);
    assert!(
        has_annotation_error(&errs, "@mailbox"),
        "expected @mailbox(2M) rejection above cap, got: {errs:?}"
    );
}

#[test]
fn mailbox_reasonable_size_is_accepted() {
    let src = r#"
        @mailbox(4096)
        @caps()
        def main() { 0 }
    "#;
    let errs = check(src);
    assert!(
        !has_annotation_error(&errs, "@mailbox"),
        "unexpected @mailbox error for reasonable size: {errs:?}"
    );
}
