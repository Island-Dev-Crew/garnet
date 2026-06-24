//! Foundation HARDEN follow-up — a self-referential value must not abort a CLI
//! lane on render. `Value::display()` recursing on a cycle is a stack overflow
//! (a non-unwinding `SIGABRT`/exit 134) the `eval`/`repl`/`test`/`doctest`
//! panic firewall CANNOT catch; the fix is a cycle/depth guard in the renderer
//! (garnet-interp `Value::render`). This proves the end-to-end guarantee at the
//! lane that can actually build a cycle (the REPL keeps an interpreter alive
//! across lines, so `let a = [1]  a.push(a)  a` is reachable).

use std::io::Write;
use std::process::{Command, Stdio};

#[test]
fn repl_survives_rendering_a_self_referential_value() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_garnet"))
        .arg("repl")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    // Build a cycle, render it, then prove the session is STILL alive.
    child
        .stdin
        .take()
        .unwrap()
        .write_all(b"let a = [1]\na.push(a)\na\n40 + 2\n:quit\n")
        .unwrap();
    let out = child.wait_with_output().unwrap();
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    // exit 0 (controlled), NOT 134 (SIGABRT from a stack overflow).
    assert_eq!(
        out.status.code(),
        Some(0),
        "the REPL must survive rendering a cycle, not abort; got {:?}\n{combined}",
        out.status
    );
    assert!(
        combined.contains("[1, [...]]"),
        "the cycle should render with a marker; got:\n{combined}"
    );
    assert!(
        combined.contains("42"),
        "the session must keep evaluating after the cycle render; got:\n{combined}"
    );
}
