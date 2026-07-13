//! S23 runtime dispatch proof: structured process argv + output capture.
//!
//! Like the S22 suite, these are deliberately source-level — they prove a
//! Garnet program can run a host command with an explicit argv array and
//! consume its captured stdout / exit code, not just that the Rust stdlib
//! helper works.

use garnet_interp::{Interpreter, Value};

fn run(src: &str) -> Value {
    // Trusted internal dispatch harness via the embedded `call` path (no entry
    // frame): permissive constructor, the opt-out from strict-by-default.
    let mut interp = Interpreter::new_permissive();
    interp.load_source(src).expect("load source");
    interp.call("main", vec![]).expect("call main")
}

fn array_items(value: Value) -> Vec<Value> {
    match value {
        Value::Array(items) => items.borrow().clone(),
        other => panic!("expected Array, got {other:?}"),
    }
}

fn expect_int(value: &Value) -> i64 {
    match value {
        Value::Int(i) => *i,
        other => panic!("expected Int, got {other:?}"),
    }
}

fn expect_bool(value: &Value) -> bool {
    match value {
        Value::Bool(b) => *b,
        other => panic!("expected Bool, got {other:?}"),
    }
}

/// `(program, argv-literal)` that echoes `marker` and exits 0, per host.
fn echo_source(marker: &str) -> (&'static str, String) {
    if cfg!(windows) {
        ("cmd", format!(r#"["/c", "echo", "{marker}"]"#))
    } else {
        ("echo", format!(r#"["{marker}"]"#))
    }
}

#[test]
fn s23_process_output_captures_stdout_and_exit_code_from_source() {
    let (prog, argv) = echo_source("garnet-s23");
    let src = format!(
        r#"
        @caps(proc)
        def main() {{
          let result = std::process::output("{prog}", {argv})
          let out = result.get("stdout")
          [contains(out, "garnet-s23"), result.get("code")]
        }}
        "#
    );

    let items = array_items(run(&src));
    assert!(
        expect_bool(&items[0]),
        "captured stdout should contain the marker"
    );
    assert_eq!(expect_int(&items[1]), 0);
}

#[test]
fn s23_spawn_args_runs_explicit_argv_and_waits_from_source() {
    let (prog, argv) = if cfg!(windows) {
        ("cmd", r#"["/c", "exit", "0"]"#)
    } else {
        ("true", "[]")
    };
    let src = format!(
        r#"
        @caps(proc)
        def main() {{
          let proc = std::process::spawn_args("{prog}", {argv})
          let status = std::process::wait(proc)
          std::process::exit_code(status)
        }}
        "#
    );

    assert_eq!(expect_int(&run(&src)), 0);
}

#[test]
fn s23_output_reports_nonzero_exit_from_source() {
    let (prog, argv) = if cfg!(windows) {
        ("cmd", r#"["/c", "exit", "7"]"#)
    } else {
        ("sh", r#"["-c", "exit 7"]"#)
    };
    let src = format!(
        r#"
        @caps(proc)
        def main() {{
          let result = std::process::output("{prog}", {argv})
          result.get("code")
        }}
        "#
    );

    assert_eq!(expect_int(&run(&src)), 7);
}
