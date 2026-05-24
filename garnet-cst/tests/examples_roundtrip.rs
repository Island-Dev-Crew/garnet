//! Byte-identical round-trip over the canonical example corpus.
//!
//! For every `.garnet` file under `examples/` and `garnet-parser-v0.3/examples/`,
//! `cst_to_source(parse_cst(src)) == src`. This is the headline S15 guarantee:
//! the CST preserves all trivia and reconstructs the source exactly.

use garnet_cst::{cst_to_source, parse_cst};
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

#[test]
fn all_examples_roundtrip_byte_identical() {
    let files = corpus();
    assert!(!files.is_empty(), "example corpus should be non-empty");
    let mut failures = Vec::new();
    for f in &files {
        let src = fs::read_to_string(f).expect("read example");
        let got = cst_to_source(parse_cst(&src).syntax());
        if got != src {
            failures.push(f.display().to_string());
        }
    }
    assert!(
        failures.is_empty(),
        "round-trip mismatch for {} / {} files: {:#?}",
        failures.len(),
        files.len(),
        failures
    );
}
