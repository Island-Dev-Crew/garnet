use garnet_wasm::{check_source, check_source_json, diff_caps_json, diff_caps_source};

const PURE: &str = "@caps()\ndef main() { 0 }\n";
const FS: &str = "@caps(fs)\ndef main() { 0 }\n";
const BAD: &str = "@caps()\ndef main() { \"unterminated }\n";
const SCOPE: &str = "declared-surface-only; does not prove absence of undeclared authority; bound annotations are not part of this surface";

#[test]
fn check_accepts_a_valid_pure_program() {
    let result = check_source(PURE);
    assert_eq!("garnet.wasm.check/1", result.schema);
    assert!(result.ok);
    assert!(result.diagnostics.is_empty());
}

#[test]
fn check_returns_one_stable_parse_diagnostic_without_panicking() {
    let result = check_source(BAD);
    assert!(!result.ok);
    assert_eq!(1, result.diagnostics.len());
    assert_eq!("parse.unterminated_string", result.diagnostics[0].code);
    assert_eq!("error", result.diagnostics[0].severity);
    assert_eq!("unterminated string literal", result.diagnostics[0].message);
}

#[test]
fn check_preserves_fatal_and_advisory_checker_semantics() {
    let fatal = check_source("@caps()\ndef main() { fs::read_file(\"secret.txt\") }\n");
    assert!(!fatal.ok);
    let caps = fatal
        .diagnostics
        .iter()
        .find(|d| d.code == "check.caps_coverage")
        .expect("real caps-coverage diagnostic");
    assert_eq!("error", caps.severity);
    assert!(caps.message.contains("does not declare `fs`"), "{caps:?}");
    let advisory = check_source("@caps()\ndef main() { try { 1 } rescue e { 0 } }\n");
    assert!(advisory.ok, "advisories must not become fatal");
    assert!(advisory
        .diagnostics
        .iter()
        .any(|d| d.code == "check.over_catch" && d.severity == "info"));
}

#[test]
fn diff_reports_expansion_surfaces_and_declared_only_scope() {
    let result = diff_caps_source(PURE, FS);
    assert_eq!("garnet.wasm.diff-caps/1", result.schema);
    assert!(result.ok);
    assert_eq!(Some(true), result.authority_expanded);
    assert_eq!(vec!["fs"], result.aggregate_added);
    assert!(result.aggregate_removed.is_empty());
    assert_eq!(Some(false), result.wildcard_introduced);
    assert_eq!(SCOPE, result.scope);
    assert!(result.parse_error.is_none());
    assert!(result.old_surface.as_ref().unwrap().aggregate.is_empty());
    assert_eq!(vec!["fs"], result.new_surface.as_ref().unwrap().aggregate);
}

#[test]
fn identical_reducing_and_per_function_only_diffs_do_not_expand_authority() {
    let identical = diff_caps_source(FS, FS);
    assert_eq!(Some(false), identical.authority_expanded);
    assert!(identical.aggregate_added.is_empty());
    let reducing = diff_caps_source(FS, PURE);
    assert_eq!(Some(false), reducing.authority_expanded);
    assert_eq!(vec!["fs"], reducing.aggregate_removed);
    let old = "@caps(fs)\ndef a() { 0 }\n@caps()\ndef b() { 0 }\n";
    let new = "@caps(fs)\ndef a() { 0 }\n@caps(fs)\ndef b() { 0 }\n";
    let moved = diff_caps_source(old, new);
    assert_eq!(Some(false), moved.authority_expanded);
    assert_eq!("b", moved.functions_caps_expanded[0].name);
    assert_eq!(vec!["fs"], moved.functions_caps_expanded[0].gained);
}

#[test]
fn diff_keeps_wildcards_unknown_caps_and_function_dimensions() {
    let old = r#"
@caps(fs)
def keep() { 0 }
@caps(fs)
def remove_me() { 0 }
@caps()
def expand_me() { 0 }
"#;
    let new = r#"
@caps(fs, custom_cap)
def keep() { 0 }
@caps(*)
def add_me() { 0 }
@caps(fs)
def expand_me() { 0 }
"#;
    let result = diff_caps_source(old, new);
    assert!(result.ok);
    assert_eq!(Some(true), result.authority_expanded);
    assert_eq!(vec!["*", "custom_cap"], result.aggregate_added);
    assert_eq!(Some(true), result.wildcard_introduced);
    assert_eq!(vec!["add_me"], result.functions_added);
    assert_eq!(vec!["remove_me"], result.functions_removed);
    assert_eq!("expand_me", result.functions_caps_expanded[0].name);
    assert_eq!(vec!["fs"], result.functions_caps_expanded[0].gained);
    assert_eq!("keep", result.functions_caps_expanded[1].name);
    assert_eq!(vec!["custom_cap"], result.functions_caps_expanded[1].gained);
}

#[test]
fn diff_parse_failures_name_the_side_and_have_no_verdict() {
    let old_bad = diff_caps_source(BAD, FS);
    assert!(!old_bad.ok);
    assert_eq!(None, old_bad.authority_expanded);
    assert_eq!("old", old_bad.parse_error.as_ref().unwrap().side);
    let new_bad = diff_caps_source(FS, BAD);
    assert!(!new_bad.ok);
    assert_eq!(None, new_bad.authority_expanded);
    assert_eq!("new", new_bad.parse_error.as_ref().unwrap().side);
}

#[test]
fn json_helpers_preserve_schema_and_verdict_fields() {
    let check: serde_json::Value =
        serde_json::from_str(&check_source_json(PURE)).expect("valid check JSON");
    assert_eq!("garnet.wasm.check/1", check["schema"]);
    assert_eq!(true, check["ok"]);
    let diff: serde_json::Value =
        serde_json::from_str(&diff_caps_json(PURE, FS)).expect("valid diff JSON");
    assert_eq!("garnet.wasm.diff-caps/1", diff["schema"]);
    assert_eq!(true, diff["ok"]);
    assert_eq!(true, diff["authority_expanded"]);
    assert_eq!("fs", diff["aggregate_added"][0]);
    assert_eq!(SCOPE, diff["scope"]);
}
