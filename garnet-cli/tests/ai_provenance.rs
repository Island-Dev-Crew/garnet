//! S65 — AI-authorship provenance (CI-gated cross-OS proof).
//!
//! `garnet seal --authored-by <provenance>` records a self-declared authorship
//! fact in the in-toto predicate; omitting it records none (default shape
//! unchanged). Runs on every OS in the `cargo test --workspace` matrix.

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
fn default_seal_has_no_authorship_field() {
    let out = garnet().arg("seal").arg(hello()).output().unwrap();
    let s = String::from_utf8(out.stdout).unwrap();
    assert!(out.status.success());
    // Silence is honest: no `--authored-by` => no authorship claim.
    assert!(!s.contains("\"authorship\""), "{s}");
}

#[test]
fn authored_by_is_recorded_in_the_predicate() {
    let out = garnet()
        .arg("seal")
        .arg(hello())
        .arg("--authored-by")
        .arg("ai:claude-opus-4-8")
        .output()
        .unwrap();
    let s = String::from_utf8(out.stdout).unwrap();
    assert!(out.status.success());
    assert!(s.contains(r#""authorship":"ai:claude-opus-4-8""#), "{s}");
    // It rides inside the same predicate as the capability manifest.
    assert!(s.contains(r#""capability_manifest""#), "{s}");
}
