//! RB-2 — malformed-input smoke: no abort on the corpus.
//!
//! Drives the `garnet` binary over (a) the hand-built malformed corpus in
//! `tests/fixtures/malformed/` (parse errors, runtime diagnostics, the
//! converted `i64::MIN / -1` overflow, caps violations — all terminating by
//! construction) and (b) the parser fuzz seed corpus
//! (`garnet-parser-v0.3/fuzz/corpus/parse_input`, `garnet check` only —
//! seeds are full programs and `run` could be non-terminating).
//!
//! The assertion: every invocation exits with a CONTROLLED diagnostic code
//! (0, 1, or 2 — the S34 contract) — never a panic exit (101) and never a
//! signal/abort (`status.code() == None` on Unix).
//!
//! Scoped claim (per RB-2): "no abort on this corpus" — not "never panics".
//! Unbounded recursion without `@max_depth` is deliberately OUT of this
//! corpus: stack exhaustion there is the documented opt-in-ceiling boundary
//! (S99), not an RB-2 crash-surface item.

use std::path::{Path, PathBuf};
use std::process::Command;

fn garnet() -> Command {
    Command::new(env!("CARGO_BIN_EXE_garnet"))
}

fn assert_controlled_exit(args: &[&str], input: &Path) {
    let out = garnet()
        .args(args)
        .arg(input)
        .output()
        .expect("binary invocation itself must succeed");
    let code = out.status.code();
    assert!(
        matches!(code, Some(0) | Some(1) | Some(2)),
        "{} {} must exit 0/1/2 (controlled diagnostic), got {:?}\nstderr: {}",
        args.join(" "),
        input.display(),
        out.status,
        String::from_utf8_lossy(&out.stderr)
    );
}

fn fixture_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/malformed")
}

fn fuzz_seed_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../garnet-parser-v0.3/fuzz/corpus/parse_input")
}

fn garnet_files(dir: &Path) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = std::fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("corpus dir {} must exist: {e}", dir.display()))
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|p| p.is_file())
        .collect();
    files.sort();
    assert!(
        !files.is_empty(),
        "corpus {} must be non-empty",
        dir.display()
    );
    files
}

#[test]
fn malformed_fixtures_never_abort_check_or_either_backend() {
    for f in garnet_files(&fixture_dir()) {
        assert_controlled_exit(&["check"], &f);
        assert_controlled_exit(&["run", "--interp"], &f);
        assert_controlled_exit(&["run", "--vm"], &f);
    }
}

#[test]
fn parser_fuzz_seeds_never_abort_check() {
    for f in garnet_files(&fuzz_seed_dir()) {
        assert_controlled_exit(&["check"], &f);
    }
}

#[test]
fn non_utf8_input_is_a_controlled_error() {
    let dir = tempfile::TempDir::new().expect("tempdir");
    let p = dir.path().join("non_utf8.garnet");
    std::fs::write(&p, [0xC3, 0x28, 0xA0, 0xFF, 0x00, 0x9F]).expect("write bytes");
    assert_controlled_exit(&["check"], &p);
    assert_controlled_exit(&["run", "--interp"], &p);
    assert_controlled_exit(&["run", "--vm"], &p);
}
