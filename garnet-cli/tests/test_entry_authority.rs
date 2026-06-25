//! PR-2 — test-runner entry-authority parity (Foundation Tier-0).
//!
//! `garnet test` routes each test function through the program-entry capability
//! frame (`Interpreter::call_entry`), so a `@caps()` test that exercises
//! undeclared host authority FAILS with exactly the same trap `garnet run`
//! raises for a `@caps()` `main`. Before PR-2 the test runner used the embedded
//! `call` path (no entry frame), so a test could exercise authority `garnet run`
//! would reject — a trust-kernel hole. These are deterministic traps, not
//! assertions of intent.

use std::process::Command;

fn garnet() -> Command {
    Command::new(env!("CARGO_BIN_EXE_garnet"))
}

/// The exact substring the runtime raises when fs authority is undeclared in the
/// calling chain (shared by `garnet run` and, post-PR-2, `garnet test`).
const FS_TRAP: &str = "requires @caps(fs)";

/// A `@caps()` test that exercises fs via a helper, laid out as a project so the
/// `test` command discovers it under `tests/`.
fn project_with_test(body: &str) -> tempfile::TempDir {
    let dir = tempfile::TempDir::new().unwrap();
    std::fs::create_dir(dir.path().join("tests")).unwrap();
    std::fs::write(dir.path().join("tests/probe.garnet"), body).unwrap();
    dir
}

/// Baseline: `garnet run` rejects a `@caps()` entry that exercises fs via a
/// helper — the reference behavior the test runner must match.
#[test]
fn run_rejects_undeclared_fs_at_entry() {
    let dir = tempfile::TempDir::new().unwrap();
    let path = dir.path().join("prog.garnet");
    std::fs::write(
        &path,
        "def reads(p) { read_file(p) }\n@caps()\ndef main() -> int { reads(\"x.txt\")  0 }\n",
    )
    .unwrap();
    let out = garnet().arg("run").arg(&path).output().unwrap();
    assert!(
        !out.status.success(),
        "garnet run must reject a @caps() entry that uses undeclared fs"
    );
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains(FS_TRAP), "stderr was: {err}");
}

fn escaped_path(path: &std::path::Path) -> String {
    path.to_string_lossy().replace('\\', "\\\\")
}

fn run_file(program: &str, backend: &str) -> std::process::Output {
    let dir = tempfile::TempDir::new().unwrap();
    let path = dir.path().join("prog.garnet");
    std::fs::write(&path, program).unwrap();
    garnet().args(["run", backend]).arg(&path).output().unwrap()
}

fn load_time_secret_program(prefix: &str, main_caps: &str, dir: &tempfile::TempDir) -> String {
    let secret = dir.path().join("secret.txt");
    std::fs::write(&secret, "codex-s114-secret").unwrap();
    let secret = escaped_path(&secret);
    format!("{prefix} leaked = read_file(\"{secret}\")\n@caps({main_caps})\ndef main() {{ 0 }}\n")
}

#[test]
fn run_rejects_top_level_let_initializer_fs_before_main_on_both_backends() {
    let secret_dir = tempfile::TempDir::new().unwrap();
    let src = load_time_secret_program("let", "", &secret_dir);
    for backend in ["--interp", "--vm"] {
        let out = run_file(&src, backend);
        assert!(
            !out.status.success(),
            "{backend} must reject load-time fs authority under @caps()"
        );
        let err = String::from_utf8_lossy(&out.stderr);
        assert!(err.contains(FS_TRAP), "{backend} stderr was: {err}");
    }
}

#[test]
fn run_rejects_top_level_const_initializer_fs_before_main_on_both_backends() {
    let secret_dir = tempfile::TempDir::new().unwrap();
    let src = load_time_secret_program("const", "", &secret_dir);
    for backend in ["--interp", "--vm"] {
        let out = run_file(&src, backend);
        assert!(
            !out.status.success(),
            "{backend} must reject load-time fs authority under @caps()"
        );
        let err = String::from_utf8_lossy(&out.stderr);
        assert!(err.contains(FS_TRAP), "{backend} stderr was: {err}");
    }
}

#[test]
fn run_allows_declared_entry_caps_for_load_time_initializer() {
    let secret_dir = tempfile::TempDir::new().unwrap();
    let src = load_time_secret_program("let", "fs", &secret_dir);
    for backend in ["--interp", "--vm"] {
        let out = run_file(&src, backend);
        assert!(
            out.status.success(),
            "{backend} should allow load-time fs authority when main declares fs: {}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
    }
}

/// PR-2 trap: `garnet test` must FAIL a `@caps()` test that exercises undeclared
/// fs, with the SAME capability trap — not silently pass it.
#[test]
fn test_runner_enforces_entry_authority_like_run() {
    let dir = project_with_test(
        "def reads(p) { read_file(p) }\n\n@caps()\ndef test_fs_leak() -> int { reads(\"x.txt\")  0 }\n",
    );
    let out = garnet().arg("test").arg(dir.path()).output().unwrap();
    assert!(
        !out.status.success(),
        "garnet test must FAIL a @caps() test that exercises undeclared fs"
    );
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        combined.contains("test_fs_leak") && combined.contains(FS_TRAP),
        "expected the fs trap on test_fs_leak; got:\n{combined}"
    );
}

/// Positive control: declaring `@caps(fs)` on the test is allowed — the fix
/// rejects only *undeclared* authority, it does not over-reject.
#[test]
fn test_runner_allows_declared_caps() {
    let dir = project_with_test("@caps(fs)\ndef test_declared() -> int { 0 }\n");
    let out = garnet().arg("test").arg(dir.path()).output().unwrap();
    assert!(
        out.status.success(),
        "a @caps(fs) test must pass; got:\n{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
}

/// A purely-computational `@caps()` test still passes (no behavior change for the
/// common case).
#[test]
fn test_runner_passes_pure_computational_test() {
    let dir = project_with_test("@caps()\ndef test_pure() -> int { 1 + 1 }\n");
    let out = garnet().arg("test").arg(dir.path()).output().unwrap();
    assert!(
        out.status.success(),
        "a pure @caps() test must pass; got:\n{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
}
