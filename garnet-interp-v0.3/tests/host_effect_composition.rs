//! S25 capstone: the host-effect runtime composes END-TO-END from Garnet source.
//!
//! A single `@caps(proc, fs)` program threads every surface completed across
//! S22-S24 together:
//!   * `std::process::output` (S23) runs a host command and captures its stdout,
//!   * `std::log::to_file` (S24) appends a leveled line to a real file,
//!   * `memory::episodic` (S22) keeps a live Mnemos trace of what was logged,
//!   * `read_file` reads the sink back, and `crypto::blake3` binds provenance.
//!
//! Determinism: a cfg-selected echo command + a unique temp log path make the
//! asserted values byte-stable on every host. Each test cleans up its file.

use garnet_interp::{Interpreter, Value};

fn run(src: &str) -> Value {
    // Trusted internal harness driving host effects through the embedded `call`
    // path (no program-entry frame): use the permissive constructor, the
    // documented opt-out from strict-by-default (Interpreter::new()).
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

fn expect_str(value: &Value) -> String {
    match value {
        Value::Str(s) => (**s).clone(),
        other => panic!("expected String, got {other:?}"),
    }
}

fn expect_int(value: &Value) -> i64 {
    match value {
        Value::Int(i) => *i,
        other => panic!("expected Int, got {other:?}"),
    }
}

fn unique_temp_path(tag: &str) -> std::path::PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "garnet_s25_{tag}_{}_{nanos}.log",
        std::process::id()
    ))
}

/// `(program, argv-literal)` that echoes `marker` and exits 0, per host.
fn echo(marker: &str) -> (&'static str, String) {
    if cfg!(windows) {
        ("cmd", format!(r#"["/c", "echo", "{marker}"]"#))
    } else {
        ("echo", format!(r#"["{marker}"]"#))
    }
}

#[test]
fn host_effect_pipeline_composes_process_log_memory_provenance() {
    let (prog, argv) = echo("garnet-s25");
    let logpath = unique_temp_path("pipeline");
    let lp = logpath.to_str().unwrap().replace('\\', "/");

    let src = format!(
        r#"
        @caps(proc, fs)
        def main() {{
          let out = std::process::output("{prog}", {argv})
          let token = trim(out.get("stdout"))
          let line = std::log::to_file("{lp}", "INFO", token)
          let trace = memory::episodic("s25")
          trace.append(line)
          let recent = trace.recent(1)
          let contents = read_file("{lp}")
          [token, recent.len(), contents, out.get("code"), crypto::blake3(token)]
        }}
        "#
    );

    let items = array_items(run(&src));
    assert_eq!(
        expect_str(&items[0]),
        "garnet-s25",
        "S23 captured + normalized stdout"
    );
    assert_eq!(expect_int(&items[1]), 1, "S22 episodic recall");
    assert!(
        expect_str(&items[2]).contains("[INFO] garnet-s25"),
        "S24 file sink written then read back: {:?}",
        expect_str(&items[2])
    );
    assert_eq!(expect_int(&items[3]), 0, "process exit code");
    let fp = expect_str(&items[4]);
    assert_eq!(fp.len(), 64, "blake3 provenance is 64 hex chars: {fp}");
    assert!(
        fp.chars().all(|c| c.is_ascii_hexdigit()),
        "provenance is hex: {fp}"
    );

    std::fs::remove_file(&logpath).ok();
}

#[test]
fn host_effect_pipeline_logs_each_stage_and_recalls_all() {
    let (prog, alpha) = echo("alpha");
    let (_p, beta) = echo("beta");
    let logpath = unique_temp_path("multi");
    let lp = logpath.to_str().unwrap().replace('\\', "/");

    let src = format!(
        r#"
        @caps(proc, fs)
        def main() {{
          let a_out = std::process::output("{prog}", {alpha})
          let a = trim(a_out.get("stdout"))
          let b_out = std::process::output("{prog}", {beta})
          let b = trim(b_out.get("stdout"))
          std::log::to_file("{lp}", "INFO", a)
          std::log::to_file("{lp}", "WARN", b)
          let trace = memory::episodic("s25-multi")
          trace.append(a)
          trace.append(b)
          [trace.recent(2).len(), read_file("{lp}")]
        }}
        "#
    );

    let items = array_items(run(&src));
    assert_eq!(expect_int(&items[0]), 2, "both stages recalled from memory");
    let contents = expect_str(&items[1]);
    assert!(
        contents.contains("[INFO] alpha"),
        "first stage logged: {contents:?}"
    );
    assert!(
        contents.contains("[WARN] beta"),
        "second stage logged: {contents:?}"
    );
    let first = contents.find("[INFO] alpha").unwrap();
    let second = contents.find("[WARN] beta").unwrap();
    assert!(
        first < second,
        "log lines appended in pipeline order: {contents:?}"
    );

    std::fs::remove_file(&logpath).ok();
}
