//! RB-2 + RFC-0002 — integer-overflow abort conversion: cross-backend parity.
//!
//! `i64::MIN / -1` was a process abort on BOTH backends (interp: panic in
//! the spawned interpreter thread, degraded to a generic "thread panicked"
//! line by the cmd/run.rs firewall; VM: in-process panic). RB-2 converts it
//! to a runtime diagnostic with the SAME message on both backends, in the
//! `vm_and_interp_traps_are_identical` style: same exit code, same
//! diagnostic line — not merely "some error".
//!
//! RFC-0002 ("integer arithmetic is checked by default", accepted 2026-06-12)
//! extends the SAME discipline to `+`, `-`, `*`, and unary `-`. The cases below
//! assert each overflowing operator exits `1` (a controlled diagnostic, not an
//! abort) with a byte-identical `integer overflow: …` line on both backends —
//! the user-facing proof that checked arithmetic is profile-independent (no
//! more "wrap in release, abort in debug").

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

// ── RFC-0002: +, -, *, unary - cross-backend parity ─────────────────────

/// Assert an overflowing `+`/`-`/`*`/unary-`-` program exits `1` with the
/// byte-identical `diag` line on BOTH backends — the RFC-0002 user-facing
/// guarantee (controlled diagnostic, profile-independent, never an abort).
fn assert_overflow_parity(program: &str, diag: &str) {
    let interp = run_backend(program, "--interp");
    let vm = run_backend(program, "--vm");
    assert_eq!(
        interp.status.code(),
        Some(1),
        "interp: overflow must be a diagnostic exit, not an abort: {:?}\nstderr: {}",
        interp.status,
        String::from_utf8_lossy(&interp.stderr)
    );
    assert_eq!(
        vm.status.code(),
        Some(1),
        "vm: overflow must be a diagnostic exit, not an abort: {:?}\nstderr: {}",
        vm.status,
        String::from_utf8_lossy(&vm.stderr)
    );
    assert!(
        String::from_utf8_lossy(&interp.stderr).contains(diag),
        "interp stderr missing `{diag}`: {}",
        String::from_utf8_lossy(&interp.stderr)
    );
    assert!(
        String::from_utf8_lossy(&vm.stderr).contains(diag),
        "VM stderr missing the identical `{diag}`: {}",
        String::from_utf8_lossy(&vm.stderr)
    );
}

#[test]
fn add_overflow_is_a_diagnostic_on_both_backends_with_identical_message() {
    // i64::MAX + 1.
    let program = "@caps()\ndef main() {\n  var a = 9223372036854775807\n  a + 1\n}\n";
    assert_overflow_parity(program, "integer overflow: 9223372036854775807 + 1");
}

#[test]
fn sub_overflow_is_a_diagnostic_on_both_backends_with_identical_message() {
    // i64::MIN - 1.
    let program = "@caps()\ndef main() {\n  var a = 0 - 9223372036854775807 - 1\n  a - 1\n}\n";
    assert_overflow_parity(program, "integer overflow: -9223372036854775808 - 1");
}

#[test]
fn mul_overflow_is_a_diagnostic_on_both_backends_with_identical_message() {
    // 3037000500^2 = 9223372037000250000 > i64::MAX.
    let program = "@caps()\ndef main() {\n  var a = 3037000500\n  a * a\n}\n";
    assert_overflow_parity(program, "integer overflow: 3037000500 * 3037000500");
}

#[test]
fn neg_overflow_is_a_diagnostic_on_both_backends_with_identical_message() {
    // -(i64::MIN). The doubled `-` is `-(−9223372036854775808)` — both backends
    // format the offending operand identically, which is the parity point.
    let program = "@caps()\ndef main() {\n  var a = 0 - 9223372036854775807 - 1\n  -a\n}\n";
    assert_overflow_parity(program, "integer overflow: --9223372036854775808");
}
