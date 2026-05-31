//! S36 — `garnet caps` integration test (runs the built binary).
//!
//! Confirms the capability manifest is emitted as well-formed JSON for a file
//! (per-program) and a directory (per-package aggregate), with the authoritative
//! exit code.

use std::path::{Path, PathBuf};
use std::process::Command;

fn garnet() -> Command {
    Command::new(env!("CARGO_BIN_EXE_garnet"))
}

fn fresh(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("garnet_s36_{tag}"));
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
fn caps_file_emits_manifest_json() {
    let dir = fresh("file");
    let p = write(&dir, "m.garnet", "@caps(fs)\ndef main() { 1 }\n");
    let out = garnet().arg("caps").arg(&p).output().unwrap();
    assert!(out.status.success());
    let s = String::from_utf8(out.stdout).unwrap();
    assert!(
        s.contains(r#""schema":"garnet-capability-manifest-v1""#),
        "{s}"
    );
    assert!(s.contains(r#""aggregate":["fs"]"#), "{s}");
    assert!(s.contains(r#"{"name":"main","caps":["fs"]}"#), "{s}");
    assert!(s.contains(r#""wildcard":false"#), "{s}");
}

#[test]
fn caps_dir_aggregates_package() {
    let dir = fresh("dir");
    write(&dir, "a.garnet", "@caps(fs)\ndef a() { 1 }\n");
    write(&dir, "b.garnet", "@caps(net)\ndef b() { 1 }\n");
    let out = garnet().arg("caps").arg(&dir).output().unwrap();
    assert!(out.status.success());
    let s = String::from_utf8(out.stdout).unwrap();
    // Union of both files' caps, sorted.
    assert!(s.contains(r#""aggregate":["fs","net"]"#), "{s}");
}

#[test]
fn caps_parse_error_exits_1() {
    let dir = fresh("err");
    let p = write(&dir, "bad.garnet", "def main(\n"); // incomplete — parse error
    let out = garnet().arg("caps").arg(&p).output().unwrap();
    assert_eq!(out.status.code(), Some(1));
}
