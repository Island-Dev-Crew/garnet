//! S34 — `garnet check --format json` integration test.
//!
//! Runs the built `garnet` binary and asserts the machine-readable JSON shape
//! and the authoritative exit code over real files (the contract's "both a
//! human-readable and a machine-parseable form, an authoritative exit code").

use std::path::PathBuf;
use std::process::Command;

fn garnet() -> Command {
    Command::new(env!("CARGO_BIN_EXE_garnet"))
}

fn write(tag: &str, name: &str, body: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("garnet_s34_{tag}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(name);
    std::fs::write(&path, body).unwrap();
    path
}

#[test]
fn json_clean_file_is_ok_and_exits_0() {
    let p = write("clean", "main.garnet", "@caps()\ndef main() { 1 }\n");
    let out = garnet()
        .args(["check", "--format", "json"])
        .arg(&p)
        .output()
        .unwrap();
    assert!(out.status.success(), "clean file must exit 0");
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains(r#""diagnostics":[]"#), "stdout: {stdout}");
    assert!(stdout.contains(r#""ok":true"#), "stdout: {stdout}");
}

#[test]
fn json_error_file_emits_error_diagnostic_and_exits_1() {
    // `main` without @caps fails the safe-mode check fatally.
    let p = write("err", "main.garnet", "def main() { 1 }\n");
    let out = garnet()
        .args(["check", "--format", "json"])
        .arg(&p)
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1), "an error file must exit 1");
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains(r#""severity":"error""#), "stdout: {stdout}");
    assert!(
        stdout.contains(r#""code":"check.annotation_error""#),
        "stdout: {stdout}"
    );
    assert!(stdout.contains(r#""ok":false"#), "stdout: {stdout}");
}

#[test]
fn json_parse_error_carries_a_span() {
    // An unterminated string literal is a parse error, which carries a span.
    let p = write("parse", "main.garnet", "def main() { \"oops }\n");
    let out = garnet()
        .args(["check", "--format", "json"])
        .arg(&p)
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(
        stdout.contains(r#""code":"parse."#),
        "expected a parse.* code: {stdout}"
    );
    assert!(
        stdout.contains(r#""span":{"start":"#),
        "parse diagnostics must carry a span: {stdout}"
    );
}

#[test]
fn default_format_is_human_not_json() {
    let p = write("human", "main.garnet", "@caps()\ndef main() { 1 }\n");
    let out = garnet().arg("check").arg(&p).output().unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(
        stdout.contains("functions checked"),
        "human summary expected: {stdout}"
    );
    assert!(
        !stdout.contains(r#""diagnostics""#),
        "default output must not be JSON: {stdout}"
    );
}
