//! RB-4b.1 — substrate fidelity: the green-tree path must match the
//! recursive-descent parser EXACTLY, spans included, and must agree with it
//! about what is an error.
//!
//! The pre-existing parity test deliberately erased spans before comparing
//! ("span values are not load-bearing"); RB-4b makes the green tree the
//! substrate the AST is derived from, so spans BECOME load-bearing (doc
//! extraction scans backwards from item span starts; miette carets render
//! from error spans). These tests are the red→green record for closing
//! that gap.

use garnet_cst::{cst_to_ast, parse_cst};
use garnet_parser::parse_source;
use std::fs;
use std::path::{Path, PathBuf};

fn garnet_files(dir: &Path) -> Vec<PathBuf> {
    let mut v = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for e in entries.flatten() {
            let p = e.path();
            if p.extension().and_then(|x| x.to_str()) == Some("garnet") {
                v.push(p);
            }
        }
    }
    v.sort();
    v
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace root")
        .to_path_buf()
}

fn corpus() -> Vec<PathBuf> {
    let root = workspace_root();
    let mut files = garnet_files(&root.join("examples"));
    files.extend(garnet_files(&root.join("garnet-parser-v0.3/examples")));
    files
}

#[test]
fn cst_to_ast_is_span_exact_on_corpus() {
    let mut mismatches = Vec::new();
    let mut compared = 0usize;
    let files = corpus();
    let total = files.len();
    for f in files {
        let src = fs::read_to_string(&f).expect("read example");
        let Ok(ast) = parse_source(&src) else {
            continue;
        };
        compared += 1;
        let via_cst = cst_to_ast(parse_cst(&src).syntax());
        let a = format!("{ast:?}");
        let b = format!("{via_cst:?}");
        if a != b {
            // Report the first divergence point for the log.
            let pos = a
                .bytes()
                .zip(b.bytes())
                .position(|(x, y)| x != y)
                .unwrap_or(a.len().min(b.len()));
            let lo = pos.saturating_sub(60);
            mismatches.push(format!(
                "{}: first divergence at byte {pos}:\n  parser: …{}\n  cst:    …{}",
                f.display(),
                &a[lo..(pos + 60).min(a.len())],
                &b[lo..(pos + 60).min(b.len())],
            ));
        }
    }
    assert_eq!(compared, total, "every corpus file must be compared");
    assert!(
        mismatches.is_empty(),
        "cst_to_ast diverged from parse_source (spans included) on {} of {} files:\n{}",
        mismatches.len(),
        compared,
        mismatches.join("\n")
    );
}

#[test]
fn error_verdict_parity_with_parse_source() {
    // The two paths must agree about WHAT IS AN ERROR: parse_cst records
    // errors (it never fails); parse_source fails fast. For every corpus
    // file and every malformed fixture, parse_source rejecting must imply
    // recorded CST errors, and parse_source accepting must imply zero.
    let root = workspace_root();
    let mut inputs = corpus();
    inputs.extend(garnet_files(
        &root.join("garnet-cli/tests/fixtures/malformed"),
    ));
    assert!(!inputs.is_empty());

    let mut disagreements = Vec::new();
    for f in inputs {
        let src = fs::read_to_string(&f).expect("read input");
        let ast_rejects = parse_source(&src).is_err();
        let cst_errors = !parse_cst(&src).errors.is_empty();
        if ast_rejects != cst_errors {
            disagreements.push(format!(
                "{}: parse_source rejects={ast_rejects}, cst records errors={cst_errors}",
                f.display()
            ));
        }
    }
    assert!(
        disagreements.is_empty(),
        "error-verdict disagreements:\n{}",
        disagreements.join("\n")
    );
}

#[test]
fn budget_and_edition_api_fences_match_the_parser() {
    use garnet_cst::parse_cst_with_budget_and_edition;
    use garnet_parser::{Edition, ParseBudget};

    // Default wrapper: valid input parses clean on both APIs.
    let ok = "@caps()\ndef main() { 1 }\n";
    assert!(garnet_cst::parse_cst(ok).errors.is_empty());
    assert!(
        parse_cst_with_budget_and_edition(ok, ParseBudget::default(), Edition::default())
            .errors
            .is_empty()
    );

    // Over-depth input: the parser rejects it (nesting budget); the CST path
    // must now RECORD an error rather than silently accept (the RB-4b.1
    // error-verdict parity that this plumbing closes).
    let deep = format!(
        "@caps()\ndef main() {{ {}1{} }}\n",
        "(".repeat(400),
        ")".repeat(400)
    );
    assert!(garnet_parser::parse_source(&deep).is_err());
    assert!(
        !garnet_cst::parse_cst(&deep).errors.is_empty(),
        "default-budget CST must record the over-depth error"
    );

    // A generous explicit budget accepts the same deep input on both paths.
    let roomy = ParseBudget {
        max_depth: usize::MAX,
        ..ParseBudget::default()
    };
    assert!(garnet_parser::parse_source_with_budget(&deep, roomy).is_ok());
    assert!(
        parse_cst_with_budget_and_edition(&deep, roomy, Edition::default())
            .errors
            .is_empty(),
        "a roomy budget must accept the deep input on the CST path too"
    );
}
