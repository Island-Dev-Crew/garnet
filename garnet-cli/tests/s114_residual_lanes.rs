//! S114-FIX-2 — close the residual capability fail-open lanes.
//!
//! The merged S114 fix (`4994867`) installed a program-entry `@caps` frame on the
//! `garnet run` / VM / `agent-loop` lanes, but `require_capability` remained
//! fail-open at `active_frames == 0`, so the SAME load/eval-time host-authority
//! bypass survived on the `eval`, `test`, `doctest`, and vendored-dependency
//! preload lanes (Opus final review, 2026-06-25, dynamically confirmed). These
//! are deterministic traps, not assertions of intent: a host primitive reached
//! with no declared capability frame must be REFUSED on every execution lane,
//! exactly as `garnet run` refuses it.

use std::process::Command;

fn garnet() -> Command {
    Command::new(env!("CARGO_BIN_EXE_garnet"))
}

/// The exact substring the runtime raises when fs authority is undeclared.
const FS_TRAP: &str = "requires @caps(fs)";
/// A unique marker written into the "secret" file; it must NEVER reach stdout.
const SECRET_MARKER: &str = "S114-RESIDUAL-SECRET-d3adb33f";

fn secret_file(dir: &std::path::Path) -> String {
    let p = dir.join("secret.txt");
    std::fs::write(&p, SECRET_MARKER).unwrap();
    p.to_string_lossy().replace('\\', "\\\\")
}

/// `garnet eval "read_file(...)"` must NOT exfiltrate file contents: with no
/// declared `@caps`, the eval lane must trap, not print the secret.
#[test]
fn eval_rejects_undeclared_fs_read() {
    let dir = tempfile::TempDir::new().unwrap();
    let secret = secret_file(dir.path());
    let out = garnet()
        .arg("eval")
        .arg(format!("read_file(\"{secret}\")"))
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        !out.status.success(),
        "garnet eval must refuse an undeclared fs read; stdout={stdout} stderr={stderr}"
    );
    assert!(
        !stdout.contains(SECRET_MARKER),
        "SECRET LEAKED to stdout via garnet eval: {stdout}"
    );
    assert!(
        stderr.contains(FS_TRAP),
        "expected caps trap; stderr was: {stderr}"
    );
}

/// A `tests/*.garnet` file whose TOP-LEVEL `let` initializer reads a file must
/// trap at load (the test runner frames test bodies, but the load-time top-level
/// initializer window was fail-open).
#[test]
fn test_lane_rejects_top_level_let_fs_at_load() {
    let dir = tempfile::TempDir::new().unwrap();
    let secret = secret_file(dir.path());
    std::fs::create_dir(dir.path().join("tests")).unwrap();
    std::fs::write(
        dir.path().join("tests/probe.garnet"),
        format!("let leaked = read_file(\"{secret}\")\n@caps()\ndef test_noop() {{ assert_eq(1, 1) }}\n"),
    )
    .unwrap();
    let out = garnet().arg("test").arg(dir.path()).output().unwrap();
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        combined.contains(FS_TRAP),
        "garnet test must trap a top-level fs read at load; output was: {combined}"
    );
}

/// A `doctest` file whose TOP-LEVEL `let` initializer reads a file must trap at
/// load rather than silently reading it.
#[test]
fn doctest_lane_rejects_top_level_let_fs_at_load() {
    let dir = tempfile::TempDir::new().unwrap();
    let secret = secret_file(dir.path());
    let path = dir.path().join("doc.garnet");
    std::fs::write(
        &path,
        format!("let leaked = read_file(\"{secret}\")\n/// ```garnet\n/// assert_eq(1, 1)\n/// ```\n@caps()\ndef main() {{ 0 }}\n"),
    )
    .unwrap();
    let out = garnet().arg("doctest").arg(&path).output().unwrap();
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        combined.contains(FS_TRAP),
        "garnet doctest must trap a top-level fs read at load; output was: {combined}"
    );
}

/// A vendored dependency whose TOP-LEVEL `let` initializer reads a file must be
/// refused during `garnet run`'s dependency preload — the dep is loaded before
/// the user entry frame and must not exercise ambient fs authority.
#[test]
fn vendored_dep_preload_rejects_top_level_fs_at_load() {
    let dir = tempfile::TempDir::new().unwrap();
    let secret = secret_file(dir.path());
    std::fs::create_dir_all(dir.path().join(".garnet/vendor/evil")).unwrap();
    std::fs::write(
        dir.path().join("Garnet.toml"),
        "[dependencies]\nevil = { path = \"evil\", vendor = \".garnet/vendor/evil\" }\n",
    )
    .unwrap();
    std::fs::write(
        dir.path().join(".garnet/vendor/evil/lib.garnet"),
        format!("let leaked = read_file(\"{secret}\")\n"),
    )
    .unwrap();
    let main = dir.path().join("main.garnet");
    std::fs::write(&main, "@caps()\ndef main() { 0 }\n").unwrap();
    let out = garnet()
        .arg("run")
        .arg("--interp")
        .arg(&main)
        .output()
        .unwrap();
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        combined.contains(FS_TRAP),
        "vendored-dep preload must trap a top-level fs read; output was: {combined}"
    );
    assert!(
        !combined.contains(SECRET_MARKER),
        "SECRET LEAKED via vendored-dep preload: {combined}"
    );
}

/// An invalid `@max_depth(9999)` on a function `main` never calls must still be
/// refused by `garnet run` on BOTH backends (range validation must happen at
/// registration, matching `garnet check`), not only when the function is entered.
#[test]
fn run_rejects_uncalled_invalid_max_depth_on_both_backends() {
    let dir = tempfile::TempDir::new().unwrap();
    let path = dir.path().join("prog.garnet");
    std::fs::write(
        &path,
        "@max_depth(9999)\ndef dead(n) { if n <= 0 { 0 } else { dead(n - 1) } }\n@caps()\ndef main() { 0 }\n",
    )
    .unwrap();
    for backend in ["--interp", "--vm"] {
        let out = garnet().args(["run", backend]).arg(&path).output().unwrap();
        let combined = format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        assert!(
            !out.status.success(),
            "garnet run {backend} must reject an out-of-range @max_depth even when uncalled; output: {combined}"
        );
        assert!(
            combined.contains("1..=64"),
            "expected the 1..=64 range diagnostic ({backend}); output: {combined}"
        );
    }
}
