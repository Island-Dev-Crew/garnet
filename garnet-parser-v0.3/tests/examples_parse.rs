//! All-corpus parse-success gate (RB-4a review restoration).
//!
//! The retired legacy round-trip test was, incidentally, the only test that
//! auto-enumerated BOTH example corpora and demanded `parse_source` success
//! on every file — the rowan CST suite deliberately tolerates unparseable
//! inputs (error recovery is a feature there). This test restores the loud
//! all-corpus gate on the AST path: a new example that fails to parse, or a
//! parser regression confined to an example no name-based test covers, fails
//! HERE instead of passing the suite silently.

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

#[test]
fn every_example_in_both_corpora_parses() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace root")
        .to_path_buf();
    let mut files = garnet_files(&root.join("examples"));
    files.extend(garnet_files(&root.join("garnet-parser-v0.3/examples")));
    assert!(!files.is_empty(), "example corpus should be non-empty");

    let mut parsed = 0usize;
    for file in &files {
        let src = fs::read_to_string(file).expect("read example");
        match garnet_parser::parse_source(&src) {
            Ok(_) => parsed += 1,
            Err(e) => panic!("example failed to parse: {} — {e:?}", file.display()),
        }
    }
    assert_eq!(parsed, files.len(), "every enumerated example must parse");
}
