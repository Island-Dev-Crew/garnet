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

fn run_backend(program: &str, backend: &str) -> std::process::Output {
    let dir = tempfile::TempDir::new().unwrap();
    let path = dir.path().join("prog.garnet");
    std::fs::write(&path, program).unwrap();
    garnet().args(["run", backend]).arg(&path).output().unwrap()
}

fn run_interp(program: &str) -> std::process::Output {
    run_backend(program, "--interp")
}

/// S99: run the same program through the bytecode VM (`garnet run --vm`).
fn run_vm(program: &str) -> std::process::Output {
    run_backend(program, "--vm")
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

// ---------------------------------------------------------------------------
// S99 — VM / interpreter `@max_depth` TRAP-parity.
//
// The interpreter has enforced `@max_depth` since S89; the VM's native path did
// not, so an over-ceiling program diverged (the VM ran it to completion, exit 0).
// S99 closes that seam: the VM now traps at the same recursion depth with the
// identical message. These tests run BOTH backends and assert the trap is
// identical — extending the S73/S85 result-parity campaign to trap-parity.
// ---------------------------------------------------------------------------

#[test]
fn vm_over_ceiling_recursion_traps_deterministically() {
    let out = run_vm(OVER);
    assert!(
        !out.status.success(),
        "the VM must trap past @max_depth (non-zero exit), not run to completion"
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("@max_depth(4) exceeded for `deep`"),
        "expected a deterministic VM max_depth trap, got: {stderr}"
    );
}

#[test]
fn vm_within_ceiling_recursion_runs() {
    let out = run_vm(WITHIN);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        out.status.success(),
        "within-ceiling recursion must run on the VM: {stdout}"
    );
    assert!(stdout.contains("=> 3"), "got {stdout}");
}

#[test]
fn vm_unannotated_recursion_is_not_capped() {
    // No `@max_depth` → the VM does not cap (it recurses on its heap frame stack).
    let prog = "def deep(n) {\n  if n <= 0 { 0 } else { 1 + deep(n - 1) }\n}\n\
                @caps()\ndef main() {\n  deep(100)\n}\n";
    let out = run_vm(prog);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        out.status.success(),
        "unannotated VM recursion must run: {stdout}"
    );
    assert!(stdout.contains("=> 100"), "got {stdout}");
}

#[test]
fn vm_and_interp_traps_are_identical() {
    // The headline trap-parity assertion: both backends trap on OVER with the
    // SAME exit code and the SAME deterministic depth message. The trap fires at
    // recursion depth 5 (ceiling 4 + 1) on each backend; the depth number proves
    // the VM counts exactly as the interpreter does — not merely "some error".
    let interp = run_interp(OVER);
    let vm = run_vm(OVER);
    assert_eq!(
        interp.status.code(),
        vm.status.code(),
        "interp and VM must agree on the trap exit code"
    );
    let trap = "@max_depth(4) exceeded for `deep` (recursion depth 5)";
    assert!(
        String::from_utf8_lossy(&interp.stderr).contains(trap),
        "interp stderr missing the trap: {}",
        String::from_utf8_lossy(&interp.stderr)
    );
    assert!(
        String::from_utf8_lossy(&vm.stderr).contains(trap),
        "VM stderr missing the identical trap: {}",
        String::from_utf8_lossy(&vm.stderr)
    );
}

#[test]
fn vm_trap_is_deterministic_across_runs() {
    // Compare exit code + the trap message, not raw stderr (which carries the
    // documented episodic-cache notes).
    let a = run_vm(OVER);
    let b = run_vm(OVER);
    assert_eq!(
        a.status.code(),
        b.status.code(),
        "VM trap exit code must be deterministic"
    );
    let trap = "@max_depth(4) exceeded for `deep` (recursion depth 5)";
    assert!(
        String::from_utf8_lossy(&a.stderr).contains(trap)
            && String::from_utf8_lossy(&b.stderr).contains(trap),
        "the same VM trap (depth 5) must fire deterministically"
    );
}
