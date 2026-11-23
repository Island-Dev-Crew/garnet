//! Regression tests for v0.8.4 checker soundness fixes:
//! - FE-F2b: capability calls inside match arm guards must be walked by CapsGraph.
//! - FE-F4: `let`-bound (and `return`/`assign`/guard/condition) `try`/`rescue` inside `@safe fn`
//!   must produce a fatal `SafeModeViolation`.

use garnet_check::{check_module, CheckError};

#[test]
fn fe_f2b_match_guard_capability_call_is_rejected_without_caps() {
    let src = r#"
@caps()
def main() {
    match 1 {
        n if fs::read_file("README.md") != "" => n
        _ => 0
    }
}
"#;
    let module = garnet_parser::parse_source(src).expect("parse failed");
    let report = check_module(&module);
    assert!(
        !report.ok(),
        "expected undeclared fs call in match guard to fail check, got {:?}",
        report.errors
    );
    assert!(
        report
            .errors
            .iter()
            .any(|e| matches!(e, CheckError::CapsCoverage { .. })),
        "expected CapsCoverage, got {:?}",
        report.errors
    );
}

#[test]
fn fe_f2b_match_guard_capability_call_passes_when_declared() {
    let src = r#"
@caps(fs)
def main() {
    match 1 {
        n if fs::read_file("README.md") != "" => n
        _ => 0
    }
}
"#;
    let module = garnet_parser::parse_source(src).expect("parse failed");
    let report = check_module(&module);
    assert!(
        report.ok(),
        "expected declared @caps(fs) in match guard to pass, got {:?}",
        report.errors
    );
}

#[test]
fn fe_f4_let_bound_try_in_safe_fn_is_fatal_safe_mode_violation() {
    let src = r#"
@safe
@caps()
fn bad_safe(x: Int) -> Int {
    let y = try { x } rescue e { 0 }
    y
}
"#;
    let module = garnet_parser::parse_source(src).expect("parse failed");
    let report = check_module(&module);
    assert!(
        !report.ok(),
        "expected let-bound try in @safe fn to fail check, got {:?}",
        report.errors
    );
    assert!(
        report
            .errors
            .iter()
            .any(|e| matches!(e, CheckError::SafeModeViolation(msg) if msg.contains("try"))),
        "expected SafeModeViolation mentioning try, got {:?}",
        report.errors
    );
}

#[test]
fn fe_f4_return_and_guard_try_in_safe_fn_are_rejected() {
    let src = r#"
@safe
@caps()
fn bad_return(x: Int) -> Int {
    return try { x } rescue e { 0 }
}
"#;
    let module = garnet_parser::parse_source(src).expect("parse failed");
    let report = check_module(&module);
    assert!(
        report
            .errors
            .iter()
            .any(|e| matches!(e, CheckError::SafeModeViolation(msg) if msg.contains("try"))),
        "expected SafeModeViolation for return try, got {:?}",
        report.errors
    );
}

#[test]
fn fe_f4_try_inside_string_interpolation_and_match_pattern_is_rejected() {
    let interp_src = r##"
@safe
@caps()
fn main() -> String {
    "#{try { raise :oops } rescue e { 7 }}"
}
"##;
    let m1 = garnet_parser::parse_source(interp_src).expect("parse interp");
    let r1 = check_module(&m1);
    assert!(
        !r1.ok()
            && r1
                .errors
                .iter()
                .any(|e| matches!(e, CheckError::SafeModeViolation(msg) if msg.contains("try"))),
        "expected SafeModeViolation for try inside string interpolation, got {:?}",
        r1.errors
    );

    let pat_src = r##"
@safe
@caps()
fn pat(s: String) -> Int {
    match s {
        "#{try { raise :oops } rescue e { 7 }}" => 1
        _ => 0
    }
}
"##;
    let m2 = garnet_parser::parse_source(pat_src).expect("parse pat");
    let r2 = check_module(&m2);
    assert!(
        !r2.ok()
            && r2
                .errors
                .iter()
                .any(|e| matches!(e, CheckError::SafeModeViolation(msg) if msg.contains("try"))),
        "expected SafeModeViolation for try inside match literal pattern, got {:?}",
        r2.errors
    );
}

#[test]
fn caps_graph_walks_match_pattern_interpolation_and_closures() {
    let src = r##"
@caps()
def main() {
    match "x" {
        "#{memory::working(1)}" => 1
        _ => 0
    }
}
"##;
    let module = garnet_parser::parse_source(src).expect("parse");
    let report = check_module(&module);
    assert!(
        !report.ok()
            && report
                .errors
                .iter()
                .any(|e| matches!(e, CheckError::CapsCoverage { .. })),
        "expected CapsCoverage for interpolated call in match literal pattern, got {:?}",
        report.errors
    );
}

fn nest_interp(inner: &str, depth: usize) -> String {
    let mut s = inner.to_string();
    for _ in 0..depth {
        s = format!("\"#{{{s}}}\"");
    }
    s
}

#[test]
fn fe_f4_deeply_nested_interpolation_is_still_rejected_without_cutoff() {
    let nested = nest_interp("try { raise :x } rescue e { 7 }", 12);
    let direct_src = format!("@safe\n@caps()\nfn main() -> String {{\n    {nested}\n}}\n");
    let m1 = garnet_parser::parse_source(&direct_src).expect("parse deep direct");
    let r1 = check_module(&m1);
    assert!(
        !r1.ok()
            && r1
                .errors
                .iter()
                .any(|e| matches!(e, CheckError::SafeModeViolation(msg) if msg.contains("try"))),
        "expected SafeModeViolation at 12-level nested interpolation, got {:?}",
        r1.errors
    );

    let pat_src = format!(
        "@safe\n@caps()\nfn pat(s: String) -> Int {{\n    match s {{\n        {nested} => 1\n        _ => 0\n    }}\n}}\n"
    );
    let m2 = garnet_parser::parse_source(&pat_src).expect("parse deep pattern");
    let r2 = check_module(&m2);
    assert!(
        !r2.ok()
            && r2
                .errors
                .iter()
                .any(|e| matches!(e, CheckError::SafeModeViolation(msg) if msg.contains("try"))),
        "expected SafeModeViolation for 12-level nested interpolation in Pattern::Literal, got {:?}",
        r2.errors
    );
}

#[test]
fn caps_graph_deeply_nested_interpolation_and_pipeline_calls_are_rejected() {
    let nested = nest_interp("memory::working(1)", 12);
    let deep_src = format!("@caps()\ndef main() {{\n    {nested}\n}}\n");
    let m1 = garnet_parser::parse_source(&deep_src).expect("parse deep caps");
    let r1 = check_module(&m1);
    assert!(
        !r1.ok()
            && r1
                .errors
                .iter()
                .any(|e| matches!(e, CheckError::CapsCoverage { .. })),
        "expected CapsCoverage at 12-level nested interpolation, got {:?}",
        r1.errors
    );

    let pipe_src = r#"
@caps()
def main() {
    1 |> memory::working
}
"#;
    let m2 = garnet_parser::parse_source(pipe_src).expect("parse pipe");
    let r2 = check_module(&m2);
    assert!(
        !r2.ok()
            && r2
                .errors
                .iter()
                .any(|e| matches!(e, CheckError::CapsCoverage { .. })),
        "expected CapsCoverage for pipeline call `1 |> memory::working`, got {:?}",
        r2.errors
    );

    let pipe_declared = r#"
@caps(mem)
def main() {
    1 |> memory::working
}
"#;
    let m3 = garnet_parser::parse_source(pipe_declared).expect("parse pipe declared");
    let r3 = check_module(&m3);
    assert!(
        r3.ok(),
        "expected declared @caps(mem) pipeline call to pass, got {:?}",
        r3.errors
    );
}
