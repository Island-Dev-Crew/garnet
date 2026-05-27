//! S24 runtime dispatch proof: `std::log` file sink (`@caps(fs)`).
//!
//! End-to-end and source-level: a Garnet program appends two log lines to a
//! file via `std::log::to_file`, then reads them back with `read_file` and the
//! test asserts both lines are present in order. This proves the file sink runs
//! through the interpreter, not just the Rust stdlib helper.

use garnet_interp::{Interpreter, Value};

fn run(src: &str) -> Value {
    let mut interp = Interpreter::new();
    interp.load_source(src).expect("load source");
    interp.call("main", vec![]).expect("call main")
}

fn expect_string(value: &Value) -> String {
    match value {
        Value::Str(s) => (**s).clone(),
        other => panic!("expected String, got {other:?}"),
    }
}

fn unique_temp_path() -> std::path::PathBuf {
    // A monotonic per-call counter — NOT just nanos — guarantees a distinct path
    // for every call within this process. The two tests below run in parallel, and
    // on hosts with coarse clock granularity (observed on macOS CI) two nanos reads
    // can be equal; a shared path then lets one test's cleanup delete/recreate the
    // file mid-run and the read-back loses a line. The counter makes that impossible.
    static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let n = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "garnet_s24_dispatch_{}_{nanos}_{n}.log",
        std::process::id()
    ))
}

#[test]
fn s24_log_to_file_writes_then_reads_back_from_source() {
    let path = unique_temp_path();
    // Forward slashes are valid path separators on every host and avoid
    // backslash-escape issues inside the Garnet string literal.
    let p = path.to_str().unwrap().replace('\\', "/");

    let src = format!(
        r#"
        @caps(fs)
        def main() {{
          std::log::to_file("{p}", "INFO", "alpha")
          std::log::to_file("{p}", "WARN", "beta")
          read_file("{p}")
        }}
        "#
    );

    let contents = expect_string(&run(&src));
    let alpha = contents
        .find("[INFO] alpha")
        .expect("first log line should be present");
    let beta = contents
        .find("[WARN] beta")
        .expect("second log line should be present");
    assert!(
        alpha < beta,
        "log lines should be appended in order: {contents:?}"
    );

    std::fs::remove_file(&path).ok();
}

#[test]
fn s24_log_to_file_returns_formatted_line_from_source() {
    let path = unique_temp_path();
    let p = path.to_str().unwrap().replace('\\', "/");

    let src = format!(
        r#"
        @caps(fs)
        def main() {{
          std::log::to_file("{p}", "ERROR", "boom")
        }}
        "#
    );

    assert_eq!(expect_string(&run(&src)), "[ERROR] boom");
    std::fs::remove_file(&path).ok();
}
