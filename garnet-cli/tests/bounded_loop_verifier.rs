//! S93 - static bounded-loop verifier integration tests.

use std::path::{Path, PathBuf};
use std::process::Command;

fn garnet() -> Command {
    Command::new(env!("CARGO_BIN_EXE_garnet"))
}

fn fresh(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("garnet_s93_{tag}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn write(dir: &Path, name: &str, body: &str) -> PathBuf {
    let path = dir.join(name);
    std::fs::write(&path, body).unwrap();
    path
}

#[test]
fn check_accepts_safe_literal_range_loop() {
    let dir = fresh("range");
    let path = write(
        dir.as_path(),
        "main.garnet",
        r#"
        fn sum5() -> Int {
            let mut total = 0
            for i in 0..5 {
                total += i
            }
            total
        }
        "#,
    );
    let out = garnet().arg("check").arg(&path).output().unwrap();
    assert!(
        out.status.success(),
        "statically bounded range loop should pass: stdout={} stderr={}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn check_rejects_safe_uncheckable_while_loop() {
    let dir = fresh("while");
    let path = write(
        dir.as_path(),
        "main.garnet",
        r#"
        fn countdown(n: Int) -> Int {
            let mut current = n
            while current > 0 {
                current -= 1
            }
            current
        }
        "#,
    );
    let out = garnet().arg("check").arg(&path).output().unwrap();
    assert_eq!(
        out.status.code(),
        Some(1),
        "uncheckable safe while loop must fail"
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("static bounded-loop verifier"),
        "expected S93 diagnostic, got: {stdout}"
    );
    assert!(
        stdout.contains("No Wasmtime fuel"),
        "diagnostic must preserve the no-Wasmtime boundary: {stdout}"
    );
}

#[test]
fn json_diagnostic_uses_bounded_loop_code() {
    let dir = fresh("json");
    let path = write(
        dir.as_path(),
        "main.garnet",
        r#"
        @bounded(10)
        def budgeted(n) {
            while n > 0 {
                n
            }
            0
        }
        "#,
    );
    let out = garnet()
        .args(["check", "--format", "json"])
        .arg(&path)
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(
        stdout.contains(r#""code":"check.bounded_loop""#),
        "stdout: {stdout}"
    );
    assert!(stdout.contains(r#""ok":false"#), "stdout: {stdout}");
}
