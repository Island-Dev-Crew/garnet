//! S33 — `garnet verify` acceptance-gate integration test.
//!
//! Exercises the gate end-to-end through `gate_tally` (collect targets →
//! edition-aware parse → safe-mode check → aggregate tally) and asserts the
//! accept/reject verdict over real files on disk — the contract's "exits zero on
//! a clean tree, non-zero on a planted regression" reduced to its testable core.

use garnet_cli::cmd::verify_gate::gate_tally;
use std::fs;
use std::path::PathBuf;

/// A fresh, uniquely-named temp dir (per test tag), cleaned if it lingered from
/// a prior run. Avoids a `tempfile` dev-dependency.
fn fresh_dir(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("garnet_s33_verify_{tag}"));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn write(dir: &std::path::Path, name: &str, body: &str) -> PathBuf {
    let path = dir.join(name);
    fs::write(&path, body).unwrap();
    path
}

#[test]
fn clean_tree_passes_with_band_5() {
    let dir = fresh_dir("clean");
    // `@caps()` satisfies the safe-mode "main must declare its capabilities" rule.
    let clean = write(&dir, "clean.garnet", "@caps()\ndef main() { 1 }\n");
    let tally = gate_tally(&clean).expect("a readable .garnet file is a valid target");
    assert!(tally.passes(), "clean tree must pass: {tally:?}");
    assert_eq!(tally.failing, 0);
    assert_eq!(tally.internal_band().get(), 5);
}

#[test]
fn planted_regression_fails_the_gate() {
    let dir = fresh_dir("planted");
    // `main` without `@caps` fails the safe-mode check fatally.
    let bad = write(&dir, "bad.garnet", "def main() { 1 }\n");
    let tally = gate_tally(&bad).expect("a readable .garnet file is a valid target");
    assert!(!tally.passes(), "a planted regression must fail the gate");
    assert_eq!(tally.failing, 1);
    assert_eq!(tally.internal_band().get(), 1);
}

#[test]
fn directory_walk_aggregates_mixed_results() {
    let dir = fresh_dir("mixed");
    write(&dir, "clean.garnet", "@caps()\ndef main() { 1 }\n");
    write(&dir, "bad.garnet", "def main() { 1 }\n");
    let tally = gate_tally(&dir).expect("dir contains .garnet files");
    assert_eq!(tally.targets, 2);
    assert_eq!(tally.failing, 1, "the one bad file must fail the aggregate");
    assert!(!tally.passes());
}

#[test]
fn empty_target_is_a_usage_error_not_a_gate_fail() {
    let dir = fresh_dir("empty");
    let err = gate_tally(&dir).unwrap_err();
    assert!(err.contains("no .garnet files"), "got: {err}");
}
