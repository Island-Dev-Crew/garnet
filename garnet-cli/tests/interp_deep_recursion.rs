//! S85 — interpreter deep-recursion robustness (WIN-S73-001).
//!
//! The tree-walking interpreter now evaluates on a thread with a large explicit
//! stack, so deep-but-finite recursion does not overflow the default OS thread
//! stack (notably ~1 MiB on Windows, where `--interp` aborted while `--vm`
//! succeeded). These run on every OS in the `cargo test --workspace` matrix, so
//! the Windows lane re-proves the original failing fixture here.

use std::path::{Path, PathBuf};
use std::process::Command;

fn garnet() -> Command {
    Command::new(env!("CARGO_BIN_EXE_garnet"))
}

fn examples() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("examples")
}

/// The exact fixture that stack-overflowed on Windows `--interp` (exit
/// 0xC00000FD) while `--vm` printed `=> 7105`. It must now run on the
/// interpreter on every platform.
#[test]
fn audit_fixture_runs_on_interpreter() {
    let prog = examples().join("mvp_function_call_demo.garnet");
    let out = garnet()
        .args(["run", "--interp"])
        .arg(&prog)
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        out.status.success(),
        "interp must not stack-overflow on {}: stdout={stdout} stderr={}",
        prog.display(),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(stdout.contains("=> 7105"), "got {stdout}");
}

/// A synthetic deep-but-finite recursion that needs the large stack: the
/// tree-walking interpreter spends ~tens of KiB of host stack per Garnet frame,
/// so a 5_000-deep recursion overflows even the default ~8 MiB main thread (and
/// the ~1 MiB Windows default) — but runs cleanly on the large `garnet-interp`
/// stack. So this test would *fail without the fix* on this machine, and passes
/// with it.
///
/// Honest scope: this raises the recursion ceiling by ~hundreds×; it is not an
/// unbounded guarantee — recursion past the large stack still overflows, which is
/// the `@bounded` *enforcement* story (S89), not a stack-size question.
#[test]
fn deep_finite_recursion_uses_the_large_stack() {
    let dir = tempfile::TempDir::new().unwrap();
    let prog = dir.path().join("deep.garnet");
    std::fs::write(
        &prog,
        "def countdown(n) {\n  if n <= 0 { 0 } else { 1 + countdown(n - 1) }\n}\n\
         @caps()\ndef main() {\n  countdown(5000)\n}\n",
    )
    .unwrap();
    let out = garnet()
        .args(["run", "--interp"])
        .arg(&prog)
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        out.status.success(),
        "5000-deep recursion must run on the large interpreter stack: stdout={stdout} stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(stdout.contains("=> 5000"), "got {stdout}");
}
