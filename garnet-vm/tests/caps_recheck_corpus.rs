//! RB-4b.3 — the caps re-check holds on the real example corpus.
//!
//! Proves the per-pass caps invariant is SATISFIED (not vacuous-because-
//! unreachable) on every workspace example the VM can compile: faithful
//! lowering never widens authority. Together with the planted-laundering
//! trap (unit test in `caps_recheck.rs`), this shows the check is a real,
//! deterministic guard — green on honest programs, red on laundering.

use std::fs;
use std::path::{Path, PathBuf};

fn examples() -> Vec<PathBuf> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace root")
        .join("examples");
    let mut v: Vec<PathBuf> = fs::read_dir(&root)
        .expect("examples dir")
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().and_then(|x| x.to_str()) == Some("garnet"))
        .collect();
    v.sort();
    v
}

#[test]
fn every_compilable_example_passes_the_caps_recheck() {
    let files = examples();
    assert!(!files.is_empty(), "example corpus is non-empty");
    let mut compiled = 0usize;
    for file in files {
        let src = fs::read_to_string(&file).expect("read example");
        // Only re-check what the VM actually compiles (mismatch/expected-fail
        // examples may not). compile_source_rechecked returns Err on EITHER a
        // compile failure OR a laundering — distinguish: a laundering message
        // is the one we must never see on the honest corpus.
        match garnet_vm::compile_source_rechecked(&src) {
            Ok(_) => compiled += 1,
            Err(e) => {
                let msg = e.to_string();
                assert!(
                    !msg.contains("caps laundering"),
                    "{}: lowering laundered authority: {msg}",
                    file.display()
                );
            }
        }
    }
    assert!(
        compiled > 0,
        "at least one example compiles + re-checks clean"
    );
}
