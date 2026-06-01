//! S89 — `@max_depth(N)` runtime enforcement seed.
//!
//! The interpreter now *enforces* the `@max_depth(N)` recursion ceiling: a
//! function declaring `@max_depth` traps deterministically when its recursion
//! depth exceeds N — real enforcement (the interpreter refuses to recurse
//! further), distinct from the S85 host-stack raise. Honest scope: this is the
//! ONE enforced ceiling; `@bounded` (Wasmtime fuel), memory, time, and mailbox
//! remain declared-not-enforced. Runs on every OS in the matrix.

use std::process::Command;

fn garnet() -> Command {
    Command::new(env!("CARGO_BIN_EXE_garnet"))
}

fn run_interp(program: &str) -> std::process::Output {
    let dir = tempfile::TempDir::new().unwrap();
    let path = dir.path().join("prog.garnet");
    std::fs::write(&path, program).unwrap();
    garnet()
        .args(["run", "--interp"])
        .arg(&path)
        .output()
        .unwrap()
}

const OVER: &str = "@max_depth(4)\ndef deep(n) {\n  if n <= 0 { 0 } else { 1 + deep(n - 1) }\n}\n\
                    @caps()\ndef main() {\n  deep(20)\n}\n";

const WITHIN: &str =
    "@max_depth(8)\ndef deep(n) {\n  if n <= 0 { 0 } else { 1 + deep(n - 1) }\n}\n\
                      @caps()\ndef main() {\n  deep(3)\n}\n";

#[test]
fn over_ceiling_recursion_traps_deterministically() {
    let out = run_interp(OVER);
    assert!(
        !out.status.success(),
        "recursion past @max_depth must trap (non-zero exit)"
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("@max_depth(4) exceeded for `deep`"),
        "expected a deterministic max_depth trap, got: {stderr}"
    );
}

#[test]
fn within_ceiling_recursion_runs() {
    let out = run_interp(WITHIN);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        out.status.success(),
        "within-ceiling recursion must run: {stdout}"
    );
    assert!(stdout.contains("=> 3"), "got {stdout}");
}

#[test]
fn the_trap_is_deterministic_across_runs() {
    // Compare the exit code and the trap *message* — not raw stderr, which also
    // carries run-to-run episodic-cache notes (the documented S73 nondeterminism).
    let a = run_interp(OVER);
    let b = run_interp(OVER);
    assert_eq!(
        a.status.code(),
        b.status.code(),
        "exit code must be deterministic"
    );
    let trap = "@max_depth(4) exceeded for `deep` (recursion depth 5)";
    assert!(
        String::from_utf8_lossy(&a.stderr).contains(trap)
            && String::from_utf8_lossy(&b.stderr).contains(trap),
        "the same trap (depth 5) must fire deterministically"
    );
}

/// A function without `@max_depth` is NOT capped by this seed (it recurses up to
/// the host stack, S85) — the enforcement is opt-in per the declared annotation.
#[test]
fn unannotated_recursion_is_not_capped() {
    let prog = "def deep(n) {\n  if n <= 0 { 0 } else { 1 + deep(n - 1) }\n}\n\
                @caps()\ndef main() {\n  deep(100)\n}\n";
    let out = run_interp(prog);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        out.status.success(),
        "unannotated recursion within stack must run: {stdout}"
    );
    assert!(stdout.contains("=> 100"), "got {stdout}");
}
