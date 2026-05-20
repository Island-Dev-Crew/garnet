//! S4 — Formatter idempotency integration test.
//!
//! Runs `garnet fmt --stdout` on every canonical example, then runs it
//! again on the output and asserts the second pass is byte-identical to
//! the first. This is the contract from
//! `F_Project_Management/GARNET_v0_5_SLICE_DOGFOOD.md` § S4 expressed as a
//! Rust integration test so the workspace `cargo test` covers it.
//!
//! What this DOES prove:
//! - Two runs of the formatter produce identical bytes (the literal
//!   idempotency claim).
//! - The formatter accepts the post-pass source (round-trips through its
//!   internal parse-then-normalize loop).
//!
//! What this does NOT prove:
//! - AST-driven semantic formatting (alignment, spacing rules, import
//!   sorting). The current formatter is whitespace+newline normalization
//!   only; full AST-driven formatting gates on a CST-preserving parser
//!   (see `garnet-cli/src/cmd/fmt.rs` doc header).
//! - That every Garnet source in the wild is idempotent — this corpus is
//!   the 13 canonical examples, not arbitrary input. S5's parser fuzz
//!   harness is the unbounded-input gate.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("garnet-cli has a workspace parent")
        .to_path_buf()
}

fn garnet_binary() -> PathBuf {
    let target = std::env::var("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| workspace_root().join("target"));
    let profile = if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    };
    target.join(profile).join("garnet")
}

fn ensure_binary_built() {
    let binary = garnet_binary();
    if binary.exists() {
        return;
    }
    // Fall back to `cargo build -p garnet-cli` so the test is robust to
    // first-time runs that don't yet have the binary materialized.
    let status = Command::new("cargo")
        .args(["build", "-p", "garnet-cli"])
        .current_dir(workspace_root())
        .status()
        .expect("cargo build invocation");
    assert!(status.success(), "cargo build -p garnet-cli failed");
}

fn fmt_stdout(file: &Path) -> Vec<u8> {
    let binary = garnet_binary();
    let output = Command::new(&binary)
        .args(["fmt", "--stdout"])
        .arg(file)
        .output()
        .unwrap_or_else(|e| panic!("running {} on {file:?}: {e}", binary.display()));
    assert!(
        output.status.success(),
        "garnet fmt --stdout {file:?} failed: stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    output.stdout
}

fn canonical_garnet_examples() -> Vec<PathBuf> {
    let dir = workspace_root().join("examples");
    let mut paths: Vec<PathBuf> = fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("read examples dir {dir:?}: {e}"))
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|p| {
            p.extension().and_then(|s| s.to_str()) == Some("garnet")
                && p.file_name()
                    .and_then(|s| s.to_str())
                    .map(|name| name.starts_with("mvp_") || name.starts_with("det_"))
                    .unwrap_or(false)
        })
        .collect();
    paths.sort();
    paths
}

#[test]
fn canonical_examples_are_idempotent_under_fmt() {
    ensure_binary_built();
    let examples = canonical_garnet_examples();
    assert!(
        !examples.is_empty(),
        "expected canonical examples to exist; corpus empty"
    );

    let mut failures: Vec<String> = Vec::new();
    for path in &examples {
        let once = fmt_stdout(path);
        // Round-trip the formatted output through a temp file and run fmt
        // again. We use a real file (not stdin) so the test exercises the
        // production code path on the same `--stdout` flag.
        let tmpdir = std::env::temp_dir();
        let tmp_once = tmpdir.join(format!(
            "garnet-fmt-once-{}.garnet",
            path.file_stem().unwrap().to_string_lossy()
        ));
        fs::write(&tmp_once, &once).unwrap_or_else(|e| panic!("write {tmp_once:?}: {e}"));
        let twice = fmt_stdout(&tmp_once);
        let _ = fs::remove_file(&tmp_once);
        if once != twice {
            failures.push(format!(
                "{} is NOT idempotent: once={} bytes, twice={} bytes",
                path.display(),
                once.len(),
                twice.len()
            ));
        }
    }

    assert!(
        failures.is_empty(),
        "S4 idempotency violations:\n{}",
        failures.join("\n")
    );
}

#[test]
fn formatter_is_deterministic_within_run() {
    // Three runs on the same input must produce identical bytes. This is
    // a weaker claim than idempotency (twice through the formatter), and
    // it catches non-determinism from e.g. unstable HashMap iteration.
    ensure_binary_built();
    let examples = canonical_garnet_examples();
    let first = examples.first().expect("at least one example");
    let a = fmt_stdout(first);
    let b = fmt_stdout(first);
    let c = fmt_stdout(first);
    assert_eq!(a, b, "fmt output differed between two runs on {first:?}");
    assert_eq!(a, c, "fmt output differed between three runs on {first:?}");
}
