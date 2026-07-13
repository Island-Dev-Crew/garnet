//! S114 acceptance, condition #4 (scope-table coverage) — the checker-only
//! capability class does NOT trap at run time.
//!
//! `time::*` and `std::uuid::new_v4` / `new_v7` require `@caps(time)` at CHECK
//! time (the CapCaps propagator), but carry `Guard::Declared` — no runtime gate
//! (see `garnet-stdlib/src/registry.rs` and the in-crate parity test
//! `guard_column_matches_runtime_backstop_behavior`). This CLI-level test pins
//! that `garnet run` never raises a capability trap for that class, so the
//! public capability enforcement scope table stays honest:
//! `C_Language_Specification/GARNET_CAPABILITY_ENFORCEMENT_SCOPE.md`.

use std::process::Command;

fn garnet() -> Command {
    Command::new(env!("CARGO_BIN_EXE_garnet"))
}

fn run_program(program: &str) -> std::process::Output {
    let dir = tempfile::TempDir::new().unwrap();
    let path = dir.path().join("prog.garnet");
    std::fs::write(&path, program).unwrap();
    garnet().arg("run").arg(&path).output().unwrap()
}

fn assert_no_caps_trap(out: &std::process::Output, label: &str) {
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        out.status.success(),
        "{label}: checker-only authority must run, not trap; output: {combined}"
    );
    assert!(
        !combined.contains("requires @caps"),
        "{label}: checker-only authority must NOT raise a runtime capability trap; \
         output: {combined}"
    );
}

/// `time::now_ms` runs under `garnet run` with `@caps(time)` declared — no trap.
#[test]
fn time_declared_runs_without_trap() {
    let out = run_program("@caps(time)\ndef main() -> int { let t = time::now_ms()  0 }\n");
    assert_no_caps_trap(&out, "time::now_ms declared");
}

/// Even UNDECLARED, `time::now_ms` does not runtime-trap under `garnet run`:
/// `time` is checker-only, so the deny-by-default runtime latch does not gate
/// it (the check-time rejection is `garnet check`'s job, not `garnet run`'s).
#[test]
fn time_undeclared_does_not_runtime_trap() {
    let out = run_program("@caps()\ndef main() -> int { let t = time::now_ms()  0 }\n");
    assert_no_caps_trap(&out, "time::now_ms undeclared");
}

/// `std::uuid::new_v4` is likewise checker-only (requires `@caps(time)` at check
/// time, no runtime gate) — it runs without a trap.
#[test]
fn uuid_v4_runs_without_trap() {
    let out = run_program("@caps(time)\ndef main() -> int { let u = std::uuid::new_v4()  0 }\n");
    assert_no_caps_trap(&out, "std::uuid::new_v4");
}
