//! S103 — the ultrapunch demo: capability-bounded acceptance of agent-authored
//! code, end-to-end, INCLUDING the rejection case (the negative proof).
//!
//! Drives `garnet agent-loop --record-dir` over the committed demo scenario
//! (`tests/fixtures/ultrapunch/`): an ACCEPT proposal yields the **4 trust
//! artifacts** + an honest `decision.md`; a capability-WIDENING proposal is refused
//! at diff-caps and **never sealed** (the punch); an over-ceiling proposal passes
//! diff-caps but the enforced kernel traps it and it is **never sealed**.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn garnet() -> Command {
    Command::new(env!("CARGO_BIN_EXE_garnet"))
}

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/ultrapunch")
        .join(name)
}

/// Run the demo loop for `proposal` into a fresh record dir; return the output and
/// the dir (kept alive by the returned `TempDir`).
fn run_demo(proposal: &str) -> (Output, tempfile::TempDir) {
    let rec = tempfile::TempDir::new().unwrap();
    let seal = rec.path().join("seal-out.json");
    let out = garnet()
        .args(["agent-loop", "--baseline"])
        .arg(fixture("baseline.garnet"))
        .arg("--proposal")
        .arg(fixture(proposal))
        .arg("--seal-out")
        .arg(&seal)
        .arg("--record-dir")
        .arg(rec.path())
        .args([
            "--attest",
            "agent=scripted-agent-v1",
            "--attest",
            "model=simulated",
            "--gate-version",
            "dogfood-gate-v1",
        ])
        .output()
        .unwrap();
    (out, rec)
}

fn read(dir: &Path, name: &str) -> String {
    std::fs::read_to_string(dir.join(name)).unwrap_or_default()
}

#[test]
fn accept_records_the_four_trust_artifacts() {
    let (out, rec) = run_demo("accept_proposal.garnet");
    assert!(
        out.status.success(),
        "the safe proposal must be ACCEPTED: {}",
        String::from_utf8_lossy(&out.stdout)
    );
    // The 4 trust artifacts + the honest decision record.
    for f in [
        "capability_manifest.json",
        "diff_caps.txt",
        "seal.json",
        "transparency_log.jsonl",
        "decision.md",
    ] {
        assert!(rec.path().join(f).is_file(), "missing trust artifact: {f}");
    }
    // Artifact 1: the declared capability surface is {fs}.
    let manifest = read(rec.path(), "capability_manifest.json");
    assert!(
        manifest.contains("garnet-capability-manifest-v1"),
        "{manifest}"
    );
    assert!(manifest.contains("\"fs\""), "{manifest}");
    // Artifact 4: the transparency-log chain verifies.
    let verify = garnet()
        .arg("caps-log")
        .arg("--verify")
        .arg(rec.path().join("transparency_log.jsonl"))
        .status()
        .unwrap();
    assert!(verify.success(), "the transparency-log chain must verify");
    // The decision is honest about scope.
    let decision = read(rec.path(), "decision.md");
    assert!(decision.contains("ACCEPTED"), "{decision}");
    assert!(decision.contains("capability+depth evidence"), "{decision}");
    assert!(decision.contains("declared-not-enforced"), "{decision}");
}

#[test]
fn widening_proposal_is_refused_and_never_sealed() {
    // The punch: a silent authority expansion is a true gate failure.
    let (out, rec) = run_demo("reject_widen.garnet");
    assert_eq!(
        out.status.code(),
        Some(1),
        "a capability widening must be refused (exit 1)"
    );
    assert!(
        !rec.path().join("seal.json").is_file(),
        "a widening proposal must NEVER be sealed — the negative proof"
    );
    assert!(read(rec.path(), "diff_caps.txt").contains("AUTHORITY EXPANDED"));
    let decision = read(rec.path(), "decision.md");
    assert!(decision.contains("REJECTED"), "{decision}");
    assert!(decision.contains("widening"), "{decision}");
}

#[test]
fn overdepth_proposal_traps_at_run_and_is_never_sealed() {
    // Passes diff-caps (no widening) but the enforced kernel traps it.
    let (out, rec) = run_demo("reject_overdepth.garnet");
    assert_eq!(
        out.status.code(),
        Some(1),
        "an over-ceiling proposal must be refused at run"
    );
    assert!(
        !rec.path().join("seal.json").is_file(),
        "a trapped proposal must NOT be sealed"
    );
    assert!(read(rec.path(), "diff_caps.txt").contains("no authority expansion"));
    let trap = read(rec.path(), "run_trap.txt");
    assert!(
        trap.contains("@max_depth(4) exceeded for `digest`"),
        "trap: {trap}"
    );
    let decision = read(rec.path(), "decision.md");
    assert!(decision.contains("REJECTED"), "{decision}");
    assert!(decision.contains("trap"), "{decision}");
}
