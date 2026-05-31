//! S66 — model/prompt/tool attestation block (CI-gated cross-OS proof).
//!
//! `garnet seal --attest <k>=<v>` (repeatable) records a deterministic
//! `attestation` object in the predicate; omitting it records none (default
//! shape unchanged); it composes with `--authored-by` (S65). Cross-OS via the
//! `cargo test --workspace` matrix.

use std::path::{Path, PathBuf};
use std::process::Command;

fn garnet() -> Command {
    Command::new(env!("CARGO_BIN_EXE_garnet"))
}

fn hello() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("examples/hello.garnet")
}

#[test]
fn attest_records_a_sorted_attestation_block() {
    let out = garnet()
        .arg("seal")
        .arg(hello())
        .args(["--attest", "tool=mcp:filesystem"])
        .args(["--attest", "model=claude-opus-4-8"])
        .output()
        .unwrap();
    let s = String::from_utf8(out.stdout).unwrap();
    assert!(out.status.success());
    // Keys are sorted deterministically: model before tool.
    assert!(
        s.contains(r#""attestation":{"model":"claude-opus-4-8","tool":"mcp:filesystem"}"#),
        "{s}"
    );
}

#[test]
fn attestation_composes_with_authorship() {
    let out = garnet()
        .arg("seal")
        .arg(hello())
        .args(["--authored-by", "ai:claude-opus-4-8"])
        .args(["--attest", "model=claude-opus-4-8"])
        .output()
        .unwrap();
    let s = String::from_utf8(out.stdout).unwrap();
    assert!(s.contains(r#""authorship":"ai:claude-opus-4-8""#), "{s}");
    assert!(
        s.contains(r#""attestation":{"model":"claude-opus-4-8"}"#),
        "{s}"
    );
}

#[test]
fn default_seal_has_no_attestation_block() {
    let out = garnet().arg("seal").arg(hello()).output().unwrap();
    let s = String::from_utf8(out.stdout).unwrap();
    assert!(out.status.success());
    assert!(!s.contains("\"attestation\""), "{s}");
}

#[test]
fn malformed_attest_is_rejected() {
    let out = garnet()
        .arg("seal")
        .arg(hello())
        .args(["--attest", "no_equals_sign"])
        .output()
        .unwrap();
    assert!(!out.status.success(), "missing '=' must be a usage error");
}
