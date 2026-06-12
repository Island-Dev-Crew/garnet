//! RB-2 follow-up — compound `%= 0` cross-backend parity.
//!
//! Found during the RB-2 adversarial review: the interpreter's
//! compound-assign path (`garnet-interp-v0.3/src/stmt.rs`, `compound_apply`)
//! had a `Div`-by-zero arm but no `Mod`-by-zero arm, so `a %= 0` fell
//! through to the catch-all "compound assignment on unsupported types"
//! error — while the VM (which lowers `%=` to the `Mod` opcode) reports
//! "division by zero". This test pins the fixed behavior in the
//! `overflow_parity.rs` style: same exit code, same diagnostic line on
//! both backends — not merely "some error".
//!
//! `a /= 0` (the arm that already existed) is pinned alongside as a
//! regression guard for the same match.

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

const MOD_ASSIGN_ZERO: &str = "@caps()\ndef main() {\n  var a = 5\n  a %= 0\n  a\n}\n";
const DIV_ASSIGN_ZERO: &str = "@caps()\ndef main() {\n  var a = 5\n  a /= 0\n  a\n}\n";

fn assert_div_by_zero_parity(program: &str, label: &str) {
    let interp = run_backend(program, "--interp");
    let vm = run_backend(program, "--vm");
    assert_eq!(
        interp.status.code(),
        Some(1),
        "{label}: interp must exit 1 with a diagnostic: {:?}",
        interp.status
    );
    assert_eq!(
        vm.status.code(),
        Some(1),
        "{label}: vm must exit 1 with a diagnostic: {:?}",
        vm.status
    );
    let diag = "division by zero";
    assert!(
        String::from_utf8_lossy(&interp.stderr).contains(diag),
        "{label}: interp stderr missing \"{diag}\": {}",
        String::from_utf8_lossy(&interp.stderr)
    );
    assert!(
        String::from_utf8_lossy(&vm.stderr).contains(diag),
        "{label}: VM stderr missing the identical \"{diag}\" diagnostic: {}",
        String::from_utf8_lossy(&vm.stderr)
    );
}

#[test]
fn compound_mod_by_zero_is_division_by_zero_on_both_backends() {
    assert_div_by_zero_parity(MOD_ASSIGN_ZERO, "a %= 0");
}

#[test]
fn compound_div_by_zero_is_division_by_zero_on_both_backends() {
    assert_div_by_zero_parity(DIV_ASSIGN_ZERO, "a /= 0");
}
