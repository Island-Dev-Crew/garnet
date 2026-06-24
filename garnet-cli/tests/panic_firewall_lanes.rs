//! Foundation HARDEN (J8) — process-abort firewall on the `eval`/`test` lanes.
//!
//! Before this slice the `run` lane caught interpreter panics (spawn-a-thread +
//! `join`) but `eval`/`repl`/`test`/`doctest` invoked the interpreter on the
//! main thread, so a panic ABORTED the process (`eval`/`test`: exit `101`) or
//! killed the session (`repl`). These are deterministic traps, not assertions
//! of intent: the trigger is `i64::MIN.abs()`, which overflows and panics —
//! reachable from ordinary Garnet source.
//!
//! Both the EXECUTE path (`eval_expr_src`/`call_entry`) and the LOAD path
//! (`load_source`) are covered: Garnet evaluates top-level `let`/`const`
//! initializers *during* load, so a `const X = i64::MIN.abs()` panics on the
//! load path and must degrade just like a panicking test body. The `test`
//! summary's pass tally is also exercised against a parse-error file, which used
//! to underflow (`usize`) and abort the summary with exit 101.
//!
//! In-crate unit tests (`cmd::repl::tests`, `cmd::doctest::tests`,
//! `panic_firewall::tests`) cover the dispatch/fence/primitive paths; this file
//! proves the lanes whose contract is a process exit code, via subprocess.

use std::process::Command;

fn garnet() -> Command {
    Command::new(env!("CARGO_BIN_EXE_garnet"))
}

/// `i64::MIN.abs()` spelled without the unparseable bare `i64::MIN` literal.
const PANIC_EXPR: &str = "(0 - 9223372036854775807 - 1).abs()";

#[test]
fn eval_lane_firewalls_a_panic_into_a_controlled_exit() {
    let out = garnet().arg("eval").arg(PANIC_EXPR).output().unwrap();
    // The decisive bit: a *controlled* exit code, not a panic abort (`101`) and
    // not a signal kill (`code() == None`).
    assert_eq!(
        out.status.code(),
        Some(1),
        "eval must firewall the panic into exit 1, not abort; status: {:?}\nstderr: {}",
        out.status,
        String::from_utf8_lossy(&out.stderr)
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("runtime error") && stderr.contains("overflow"),
        "eval stderr should carry the firewalled diagnostic, got: {stderr}"
    );
}

#[test]
fn test_lane_marks_a_panicking_test_failed_and_keeps_running() {
    let dir = tempfile::TempDir::new().unwrap();
    std::fs::create_dir(dir.path().join("tests")).unwrap();
    // One test panics, a later test passes — proving the run is NOT aborted by
    // the panic and the summary still reports both.
    std::fs::write(
        dir.path().join("tests/probe.garnet"),
        "@caps()\n\
         def test_aaa_panics() -> int {\n\
         \x20 let m = (0 - 9223372036854775807 - 1)\n\
         \x20 m.abs()\n\
         }\n\
         @caps()\n\
         def test_zzz_passes() -> int { 1 + 1 }\n",
    )
    .unwrap();

    let out = garnet().arg("test").arg(dir.path()).output().unwrap();
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );

    assert_eq!(
        out.status.code(),
        Some(1),
        "a panicking test makes the run FAIL (exit 1), not abort; got {:?}\n{combined}",
        out.status
    );
    assert!(
        combined.contains("test_aaa_panics") && combined.contains("FAILED"),
        "the panicking test must be reported FAILED; got:\n{combined}"
    );
    assert!(
        combined.contains("test_zzz_passes") && combined.contains("ok"),
        "the later test must STILL run (the panic did not abort the suite); got:\n{combined}"
    );
}

#[test]
fn test_lane_survives_a_load_time_panic_in_a_top_level_initializer() {
    // Garnet evaluates top-level `const`/`let` initializers DURING load_source,
    // so this overflow panics on the LOAD path (not the test body). The firewall
    // must keep the load from aborting the whole run.
    let dir = tempfile::TempDir::new().unwrap();
    std::fs::create_dir(dir.path().join("tests")).unwrap();
    std::fs::write(
        dir.path().join("tests/probe.garnet"),
        "const BOOM = (0 - 9223372036854775807 - 1).abs()\n@caps()\ndef test_ok() -> int { 1 }\n",
    )
    .unwrap();
    let out = garnet().arg("test").arg(dir.path()).output().unwrap();
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(
        out.status.code(),
        Some(1),
        "a load-time panic must be a controlled failure, not an abort; got {:?}\n{combined}",
        out.status
    );
    assert!(
        combined.contains("test result"),
        "the summary must still print (run not aborted); got:\n{combined}"
    );
}

#[test]
fn test_lane_summary_does_not_underflow_on_a_parse_error_file() {
    // A file that fails to parse increments the failure tally without a matching
    // run — the old `passed = total_run - total_failed` underflowed (usize) and
    // panicked the summary with exit 101. The direct pass-tally must not.
    let dir = tempfile::TempDir::new().unwrap();
    std::fs::create_dir(dir.path().join("tests")).unwrap();
    std::fs::write(dir.path().join("tests/bad.garnet"), "def test_x() {  )\n").unwrap();
    let out = garnet().arg("test").arg(dir.path()).output().unwrap();
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(
        out.status.code(),
        Some(1),
        "a parse-error file must exit 1, not underflow-panic to 101; got {:?}\n{combined}",
        out.status
    );
    assert!(
        !combined.contains("subtract with overflow"),
        "the summary must not underflow-panic; got:\n{combined}"
    );
    assert!(
        combined.contains("test result"),
        "the summary must still print; got:\n{combined}"
    );
}

#[test]
fn doctest_lane_survives_a_load_time_panic() {
    let dir = tempfile::TempDir::new().unwrap();
    let path = dir.path().join("doc.garnet");
    std::fs::write(
        &path,
        "/// ```garnet\n/// 1 + 1\n/// ```\nconst BOOM = (0 - 9223372036854775807 - 1).abs()\ndef f() -> int { 1 }\n",
    )
    .unwrap();
    let out = garnet().arg("doctest").arg(&path).output().unwrap();
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(
        out.status.code(),
        Some(1),
        "a doctest load-time panic must be a controlled failure, not an abort; got {:?}\n{combined}",
        out.status
    );
    assert!(
        combined.contains("failed to load"),
        "every fence should report the load failure; got:\n{combined}"
    );
}

#[test]
fn repl_preload_survives_a_load_time_panic() {
    let dir = tempfile::TempDir::new().unwrap();
    let path = dir.path().join("preload.garnet");
    std::fs::write(&path, "const BOOM = (0 - 9223372036854775807 - 1).abs()\n").unwrap();
    let mut child = garnet()
        .arg("repl")
        .arg(&path)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    use std::io::Write;
    child.stdin.take().unwrap().write_all(b":quit\n").unwrap();
    let out = child.wait_with_output().unwrap();
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(
        out.status.code(),
        Some(1),
        "a repl preload panic must exit 1, not abort; got {:?}\n{combined}",
        out.status
    );
    assert!(
        combined.contains("preload error"),
        "the preload panic should be reported as a preload error; got:\n{combined}"
    );
}
