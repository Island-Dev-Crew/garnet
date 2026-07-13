//! S114 acceptance, condition #5 — `garnet test` helper preload fails closed.
//!
//! `garnet test` pre-loads `src/main.garnet` as a helper context for the files
//! under `tests/`. If that helper fails to load, the file's tests ran against a
//! broken/partial helper — setup failure must FAIL the run, not print a warning
//! and still report "N passed; 0 failed". Verification integrity: a green run
//! must mean the tests actually ran against the intended context.

use std::process::Command;

fn garnet() -> Command {
    Command::new(env!("CARGO_BIN_EXE_garnet"))
}

fn project(helper: &str, test_file: &str) -> tempfile::TempDir {
    let dir = tempfile::TempDir::new().unwrap();
    std::fs::create_dir_all(dir.path().join("src")).unwrap();
    std::fs::create_dir_all(dir.path().join("tests")).unwrap();
    std::fs::write(dir.path().join("src/main.garnet"), helper).unwrap();
    std::fs::write(dir.path().join("tests/probe.garnet"), test_file).unwrap();
    dir
}

/// A helper that reaches for undeclared fs authority at load time (top-level
/// `let` initializer) must fail closed: the deny-by-default latch traps the
/// read, and the run must NOT report success.
#[test]
fn authority_trapping_helper_fails_the_run() {
    let dir = tempfile::TempDir::new().unwrap();
    std::fs::create_dir_all(dir.path().join("src")).unwrap();
    std::fs::create_dir_all(dir.path().join("tests")).unwrap();
    let secret = dir.path().join("secret.txt");
    std::fs::write(&secret, "HELPER-SECRET-MARKER").unwrap();
    let secret_lit = secret.to_string_lossy().replace('\\', "\\\\");
    std::fs::write(
        dir.path().join("src/main.garnet"),
        format!("let leaked = read_file(\"{secret_lit}\")\n"),
    )
    .unwrap();
    std::fs::write(
        dir.path().join("tests/probe.garnet"),
        "@caps()\ndef test_ok() -> bool { true }\n",
    )
    .unwrap();
    let out = garnet().arg("test").arg(dir.path()).output().unwrap();
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        !out.status.success(),
        "a helper that traps on undeclared authority must fail the run; output: {combined}"
    );
    assert!(
        !combined.contains("HELPER-SECRET-MARKER"),
        "secret leaked via helper preload: {combined}"
    );
}

/// A helper that fails to PARSE must likewise fail the run rather than let the
/// file's tests report a green pass against a helper that never loaded.
#[test]
fn unparseable_helper_fails_the_run() {
    let dir = project(
        "def broken( {\n", // syntax error
        "@caps()\ndef test_ok() -> bool { true }\n",
    );
    let out = garnet().arg("test").arg(dir.path()).output().unwrap();
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        !out.status.success(),
        "an unparseable helper must fail the run, not report success; output: {combined}"
    );
}

/// Control: a WELL-FORMED helper still lets the test pass — the fail-closed
/// change must not break the normal cross-file helper flow.
#[test]
fn well_formed_helper_still_passes() {
    let dir = project(
        "def helper_value() -> int { 42 }\n",
        "@caps()\ndef test_ok() -> bool { true }\n",
    );
    let out = garnet().arg("test").arg(dir.path()).output().unwrap();
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        out.status.success(),
        "a well-formed helper must not break a passing test; output: {combined}"
    );
}
