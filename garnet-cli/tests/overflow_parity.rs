//! RB-2 — integer-overflow abort conversion: cross-backend parity.
//!
//! `i64::MIN / -1` was a process abort on BOTH backends (interp: panic in
//! the spawned interpreter thread, degraded to a generic "thread panicked"
//! line by the cmd/run.rs firewall; VM: in-process panic). RB-2 converts it
//! to a runtime diagnostic with the SAME message on both backends, in the
//! `vm_and_interp_traps_are_identical` style: same exit code, same
//! diagnostic line — not merely "some error".
//!
//! Scoped claim: this covers checked division/remainder overflow only.
//! Add/sub/mul overflow policy (wrap in release, abort in debug) is a
//! language-level decision recorded as deferred in the RB-2 slice.

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

/// `i64::MIN` spelled without the unparseable bare literal.
const DIV_OVERFLOW: &str =
    "@caps()\ndef main() {\n  var a = 0 - 9223372036854775807 - 1\n  a / (0 - 1)\n}\n";
const REM_OVERFLOW: &str =
    "@caps()\ndef main() {\n  var a = 0 - 9223372036854775807 - 1\n  a % (0 - 1)\n}\n";

#[test]
fn div_overflow_is_a_diagnostic_on_both_backends_with_identical_message() {
    let interp = run_backend(DIV_OVERFLOW, "--interp");
    let vm = run_backend(DIV_OVERFLOW, "--vm");
    assert_eq!(
        interp.status.code(),
        Some(1),
        "interp: overflow must be a diagnostic exit, not an abort: {:?}",
        interp.status
    );
    assert_eq!(
        vm.status.code(),
        Some(1),
        "vm: overflow must be a diagnostic exit, not an abort: {:?}",
        vm.status
    );
    let diag = "integer overflow: -9223372036854775808 / -1";
    assert!(
        String::from_utf8_lossy(&interp.stderr).contains(diag),
        "interp stderr missing the overflow diagnostic: {}",
        String::from_utf8_lossy(&interp.stderr)
    );
    assert!(
        String::from_utf8_lossy(&vm.stderr).contains(diag),
        "VM stderr missing the identical overflow diagnostic: {}",
        String::from_utf8_lossy(&vm.stderr)
    );
}

#[test]
fn rem_overflow_is_a_diagnostic_on_both_backends_with_identical_message() {
    let interp = run_backend(REM_OVERFLOW, "--interp");
    let vm = run_backend(REM_OVERFLOW, "--vm");
    assert_eq!(interp.status.code(), Some(1));
    assert_eq!(vm.status.code(), Some(1));
    let diag = "integer overflow: -9223372036854775808 % -1";
    assert!(
        String::from_utf8_lossy(&interp.stderr).contains(diag),
        "interp stderr: {}",
        String::from_utf8_lossy(&interp.stderr)
    );
    assert!(
        String::from_utf8_lossy(&vm.stderr).contains(diag),
        "VM stderr: {}",
        String::from_utf8_lossy(&vm.stderr)
    );
}
