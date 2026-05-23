//! `cst_to_ast` structural parity vs the AST parser, on the canonical corpus.
//!
//! For every example that the AST parser accepts, `cst_to_ast(parse_cst(src))`
//! must be structurally equal to `garnet_parser::parse_source(src)`. Comparison
//! is **span-normalized** (span values are erased before comparing) since the
//! two parsers compute spans independently; structure, names, and literal
//! values must match.

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

fn corpus() -> Vec<PathBuf> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf();
    let mut files = garnet_files(&root.join("examples"));
    files.extend(garnet_files(&root.join("garnet-parser-v0.3/examples")));
    files
}

/// Replace every `Span { start: N, len: M }` with `Span` so the comparison is
/// insensitive to span values (the two parsers compute spans independently).
fn normalize_spans(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(idx) = rest.find("Span { start: ") {
        out.push_str(&rest[..idx]);
        out.push_str("Span");
        let after = &rest[idx..];
        match after.find('}') {
            Some(close) => rest = &after[close + 1..],
            None => {
                rest = "";
                break;
            }
        }
    }
    out.push_str(rest);
    out
}

#[test]
fn cst_to_ast_matches_parse_source_on_corpus() {
    let mut mismatches = Vec::new();
    let mut compared = 0usize;
    for f in corpus() {
        let src = fs::read_to_string(&f).expect("read example");
        let Ok(ast) = parse_source(&src) else {
            continue; // AST parser rejects it; nothing to compare against
        };
        let via_cst = cst_to_ast(parse_cst(&src).syntax());
        let a = normalize_spans(&format!("{ast:?}"));
        let b = normalize_spans(&format!("{via_cst:?}"));
        compared += 1;
        if a != b {
            mismatches.push(f.display().to_string());
        }
    }
    assert!(compared > 0, "should have compared at least one example");
    assert!(
        mismatches.is_empty(),
        "cst_to_ast structural mismatch on {} / {} compared files: {:#?}",
        mismatches.len(),
        compared,
        mismatches
    );
}
