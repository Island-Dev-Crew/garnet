//! S102 — `garnet agent-loop`: the real agent-acceptance loop (Stage U).
//!
//! A simulated agent proposes a Garnet change; the loop ACCEPTS it only on
//! ENFORCED evidence — diff-caps (no capability widening, Rule 2) + the enforced
//! kernel (S99 `@max_depth` / S100 `@caps` traps) — then seals it. A widening or a
//! kernel trap is a true gate failure: the proposal is refused and never sealed.
//! The agent is simulated (on-disk fixtures), not a live LLM (S94). Runs on every
//! OS in the matrix.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn garnet() -> Command {
    Command::new(env!("CARGO_BIN_EXE_garnet"))
}

// Baseline: aggregate capability surface {fs}, recursion within `@max_depth(8)`.
const BASELINE: &str = "@caps(fs)\n@max_depth(8)\ndef deep(n) { if n <= 0 { 0 } else { 1 + deep(n - 1) } }\n@caps(fs)\ndef main() { deep(3) }\n";
// Capability-safe proposal: same surface {fs}, still within the ceiling (=> 2).
const ACCEPT: &str = "@caps(fs)\n@max_depth(8)\ndef deep(n) { if n <= 0 { 0 } else { 1 + deep(n - 1) } }\n@caps(fs)\ndef main() { deep(2) }\n";
// Widening proposal: {fs} -> {fs, net} — diff-caps must hard-fail (the headline).
const WIDEN: &str = "@caps(fs, net)\n@max_depth(8)\ndef deep(n) { if n <= 0 { 0 } else { 1 + deep(n - 1) } }\n@caps(fs)\ndef main() { deep(3) }\n";
// Over-ceiling proposal: passes diff-caps (no widening) but the enforced kernel
// traps it (`@max_depth(4)` with `deep(20)`) — acceptance rests on the run, not
// only the static cap gate.
const OVERDEPTH: &str = "@caps(fs)\n@max_depth(4)\ndef deep(n) { if n <= 0 { 0 } else { 1 + deep(n - 1) } }\n@caps(fs)\ndef main() { deep(20) }\n";
// Invalid annotation proposal: `garnet check` rejects this as outside the
// documented 1..=64 ceiling range. The agent-loop must fail closed before run
// or seal can treat the invalid bound as a huge effective ceiling.
const INVALID_MAX_DEPTH: &str = "@max_depth(9999)\ndef deep(n) { if n <= 0 { 0 } else { deep(n - 1) } }\n@caps()\ndef main() { deep(100) }\n";

fn write(dir: &Path, name: &str, src: &str) -> PathBuf {
    let path = dir.join(name);
    std::fs::write(&path, src).unwrap();
    path
}

/// Stage a baseline + proposal in a fresh tempdir and run the loop.
fn run_loop(
    proposal_src: &str,
    backend: &str,
    seal_name: &str,
) -> (Output, PathBuf, tempfile::TempDir) {
    let dir = tempfile::TempDir::new().unwrap();
    let baseline = write(dir.path(), "baseline.garnet", BASELINE);
    let proposal = write(dir.path(), "proposal.garnet", proposal_src);
    let seal_out = dir.path().join(seal_name);
    let out = garnet()
        .args(["agent-loop", "--baseline"])
        .arg(&baseline)
        .arg("--proposal")
        .arg(&proposal)
        .args(["--backend", backend, "--seal-out"])
        .arg(&seal_out)
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
    (out, seal_out, dir)
}

#[test]
fn accept_path_passes_gate_runs_and_seals() {
    let (out, seal, _dir) = run_loop(ACCEPT, "interp", "accept.seal.json");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success(), "accept path must exit 0: {stdout}");
    assert!(stdout.contains("stage diff-caps -> PASS"), "{stdout}");
    assert!(stdout.contains("stage run(--interp) -> PASS"), "{stdout}");
    assert!(
        stdout.contains("ACCEPTED on capability+depth evidence"),
        "{stdout}"
    );
    assert!(seal.is_file(), "an accepted proposal must be sealed");
    // The seal records the autonomous acceptance + the gate it accepted under.
    let predicate = std::fs::read_to_string(&seal).unwrap();
    assert!(
        predicate.contains("capability_manifest"),
        "seal missing the capability manifest"
    );
    assert!(
        predicate.contains("gate_version"),
        "seal must record the gate version (Rule 3)"
    );
    assert!(
        predicate.contains("garnet-agent-loop"),
        "seal must record the accepting tool"
    );
    assert!(
        predicate.contains("\"autonomous\""),
        "seal must record the autonomous acceptance"
    );
}

#[test]
fn reject_path_widening_hardfails_and_is_refused() {
    let (out, seal, _dir) = run_loop(WIDEN, "interp", "widen.seal.json");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(
        out.status.code(),
        Some(1),
        "a capability widening must be refused (exit 1): {stdout}"
    );
    assert!(stdout.contains("AUTHORITY EXPANDED"), "{stdout}");
    assert!(stdout.contains("REJECTED at stage diff-caps"), "{stdout}");
    assert!(
        !seal.is_file(),
        "a widening proposal must NEVER be sealed — it never reaches attestation"
    );
}

#[test]
fn enforced_kernel_traps_overceiling_proposal() {
    // Passes diff-caps (no widening) but the enforced kernel (S99) traps it.
    let (out, seal, _dir) = run_loop(OVERDEPTH, "interp", "over.seal.json");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(
        out.status.code(),
        Some(1),
        "an over-ceiling proposal must be refused at run: {stdout}"
    );
    assert!(
        stdout.contains("stage diff-caps -> PASS"),
        "diff-caps should pass (no widening): {stdout}"
    );
    assert!(stdout.contains("REJECTED at stage run"), "{stdout}");
    // The enforced-kernel trap is surfaced (proves acceptance rests on the run).
    assert!(
        stdout.contains("@max_depth(4) exceeded for `deep`")
            || stderr.contains("@max_depth(4) exceeded for `deep`"),
        "the enforced @max_depth trap must surface: out={stdout} err={stderr}"
    );
    assert!(!seal.is_file(), "a trapped proposal must NOT be sealed");
}

#[test]
fn invalid_max_depth_is_refused_before_run_or_seal() {
    for backend in ["interp", "vm"] {
        let (out, seal, _dir) = run_loop(INVALID_MAX_DEPTH, backend, "invalid.seal.json");
        let stdout = String::from_utf8_lossy(&out.stdout);
        assert_eq!(
            out.status.code(),
            Some(1),
            "invalid @max_depth must be refused (backend {backend}): {stdout}"
        );
        assert!(
            stdout.contains("stage check -> REJECT"),
            "agent-loop must fail closed at check stage, got: {stdout}"
        );
        assert!(
            stdout.contains("must be in 1..=64"),
            "check diagnostic must be surfaced, got: {stdout}"
        );
        assert!(
            !seal.is_file(),
            "an invalid-bound proposal must never be sealed"
        );
    }
}

#[test]
fn accept_is_deterministic_across_runs() {
    // The simulated-agent loop is reproducible: identical inputs -> byte-identical
    // seal predicate (no timestamp in the predicate). Compares the seal JSON, not
    // raw stderr (which carries the documented episodic-cache notes).
    let (a, seal_a, _da) = run_loop(ACCEPT, "interp", "a.seal.json");
    let (b, seal_b, _db) = run_loop(ACCEPT, "interp", "b.seal.json");
    assert!(a.status.success() && b.status.success());
    let pa = std::fs::read(&seal_a).unwrap();
    let pb = std::fs::read(&seal_b).unwrap();
    assert_eq!(
        pa, pb,
        "the seal predicate must be byte-identical across runs"
    );
}

#[test]
fn vm_backend_parity_on_accept_and_trap() {
    // The same accept/reject verdicts hold on the VM backend (extends the S99/S100
    // trap-parity through the acceptance loop).
    let (accept, seal, _da) = run_loop(ACCEPT, "vm", "accept.vm.seal.json");
    assert!(accept.status.success(), "accept must pass on --vm too");
    assert!(seal.is_file());

    let (trap, seal_t, _dt) = run_loop(OVERDEPTH, "vm", "over.vm.seal.json");
    assert_eq!(
        trap.status.code(),
        Some(1),
        "the VM must trap the over-ceiling proposal too"
    );
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&trap.stdout),
        String::from_utf8_lossy(&trap.stderr)
    );
    assert!(
        combined.contains("@max_depth(4) exceeded for `deep`"),
        "{combined}"
    );
    assert!(
        !seal_t.is_file(),
        "a VM-trapped proposal must NOT be sealed"
    );
}
