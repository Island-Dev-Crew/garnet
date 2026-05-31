//! S42 — over-catch advisory integration test (runs the built binary).
//!
//! A catch-all `rescue` (no exception type) is a NON-FATAL advisory in
//! `garnet check`: it appears in human + JSON output but never changes the exit
//! code; a typed rescue is not flagged.

use std::path::{Path, PathBuf};
use std::process::Command;

fn garnet() -> Command {
    Command::new(env!("CARGO_BIN_EXE_garnet"))
}

fn fresh(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("garnet_s42_{tag}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn write(dir: &Path, name: &str, body: &str) -> PathBuf {
    let p = dir.join(name);
    std::fs::write(&p, body).unwrap();
    p
}

#[test]
fn catch_all_rescue_is_a_nonfatal_advisory() {
    let dir = fresh("human");
    let p = write(
        dir.as_path(),
        "app.garnet",
        "@caps()\ndef main() { try { 1 } rescue e { 0 } }\n",
    );
    let out = garnet().arg("check").arg(&p).output().unwrap();
    assert!(out.status.success(), "over-catch is non-fatal -> exit 0");
    let s = String::from_utf8(out.stdout).unwrap();
    assert!(
        s.contains("catch-all rescue") && s.contains("advisory"),
        "{s}"
    );
}

#[test]
fn over_catch_appears_in_json_and_keeps_ok_true() {
    let dir = fresh("json");
    let p = write(
        dir.as_path(),
        "app.garnet",
        "@caps()\ndef main() { try { 1 } rescue e { 0 } }\n",
    );
    let out = garnet()
        .args(["check", "--format", "json"])
        .arg(&p)
        .output()
        .unwrap();
    assert!(out.status.success());
    let s = String::from_utf8(out.stdout).unwrap();
    assert!(s.contains(r#""code":"check.over_catch""#), "{s}");
    assert!(s.contains(r#""ok":true"#), "advisory keeps ok=true: {s}");
}

#[test]
fn typed_rescue_is_not_flagged() {
    let dir = fresh("typed");
    let p = write(
        dir.as_path(),
        "app.garnet",
        "@caps()\ndef main() { try { 1 } rescue e: IoError { 0 } }\n",
    );
    let out = garnet().arg("check").arg(&p).output().unwrap();
    let s = String::from_utf8(out.stdout).unwrap();
    assert!(
        !s.contains("catch-all rescue"),
        "a typed rescue must not be flagged: {s}"
    );
}
