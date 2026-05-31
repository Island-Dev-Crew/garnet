//! S49 — AI-PR-review-collapse wedge demo (CI-gated correctness proof).
//!
//! The `examples/wedge_pr_review/{before,after}.garnet` pair simulates an
//! AI-suggested PR that silently widens authority `@caps(fs)` → `@caps(fs, net)`.
//! This test proves the wedge: `garnet check` is clean on BOTH versions (the
//! escalation is invisible to the checker), yet `garnet diff-caps` flags the
//! gained `net` capability and exits non-zero, and `garnet sandbox` shows the
//! egress posture flip deny-all → allow. Runs on every OS in the `cargo test
//! --workspace` matrix, so the wedge is cross-OS CI-gated.

use std::path::{Path, PathBuf};
use std::process::Command;

fn garnet() -> Command {
    Command::new(env!("CARGO_BIN_EXE_garnet"))
}

fn example(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("examples")
        .join("wedge_pr_review")
        .join(rel)
}

#[test]
fn both_versions_check_clean() {
    // The escalation is an authority change, not a checker error — both clean.
    for rel in ["before.garnet", "after.garnet"] {
        let out = garnet().arg("check").arg(example(rel)).output().unwrap();
        let s = String::from_utf8(out.stdout).unwrap();
        assert!(out.status.success(), "{rel} should check clean: {s}");
        assert!(s.contains("0 diagnostics"), "{rel}: {s}");
    }
}

#[test]
fn diff_caps_catches_the_silent_escalation() {
    let out = garnet()
        .arg("diff-caps")
        .arg(example("before.garnet"))
        .arg(example("after.garnet"))
        .output()
        .unwrap();
    let s = String::from_utf8(out.stdout).unwrap();
    assert!(
        !out.status.success(),
        "diff-caps must exit non-zero on authority expansion: {s}"
    );
    assert!(s.contains("caps GAINED") && s.contains("net"), "{s}");
    assert!(s.contains("AUTHORITY EXPANDED"), "{s}");
}

#[test]
fn sandbox_shows_egress_flip() {
    let before = sandbox_egress("before.garnet");
    let after = sandbox_egress("after.garnet");
    assert_eq!(before, "deny-all", "before should deny egress");
    assert_eq!(after, "allow", "after should allow egress");
}

fn sandbox_egress(rel: &str) -> String {
    let out = garnet()
        .args(["sandbox", "--format", "json"])
        .arg(example(rel))
        .output()
        .unwrap();
    let s = String::from_utf8(out.stdout).unwrap();
    // Hand-parse the deterministic JSON: find "mode":"<x>".
    let needle = "\"mode\":\"";
    let start = s.find(needle).expect("egress mode present") + needle.len();
    let end = s[start..].find('"').unwrap() + start;
    s[start..end].to_string()
}
