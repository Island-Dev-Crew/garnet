//! S46 — `garnet sandbox` integration test (runs the built binary).
//!
//! Generates the seccomp/WASI/egress policy implied by a file's `@caps` and
//! asserts the human + JSON shape. Generation only — the policy is explicitly
//! marked `enforced: false`.

use std::path::{Path, PathBuf};
use std::process::Command;

fn garnet() -> Command {
    Command::new(env!("CARGO_BIN_EXE_garnet"))
}

fn fresh(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("garnet_s46_{tag}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn write(dir: &Path, body: &str) -> PathBuf {
    let p = dir.join("app.garnet");
    std::fs::write(&p, body).unwrap();
    p
}

#[test]
fn fs_caps_allow_files_deny_egress() {
    let dir = fresh("fs");
    let p = write(
        dir.as_path(),
        "@caps(fs)\ndef read_it() { 0 }\ndef main() { 0 }\n",
    );
    let out = garnet().arg("sandbox").arg(&p).output().unwrap();
    assert!(out.status.success());
    let s = String::from_utf8(out.stdout).unwrap();
    assert!(s.contains("egress: deny-all"), "{s}");
    assert!(s.contains("preopens=true"), "{s}");
    assert!(s.contains("enforced: false"), "{s}");
}

#[test]
fn json_marks_unenforced_and_is_well_formed() {
    let dir = fresh("json");
    let p = write(
        dir.as_path(),
        "@caps(net)\ndef call() { 0 }\ndef main() { 0 }\n",
    );
    let out = garnet()
        .args(["sandbox", "--format", "json"])
        .arg(&p)
        .output()
        .unwrap();
    assert!(out.status.success());
    let s = String::from_utf8(out.stdout).unwrap();
    assert!(s.contains(r#""schema":"garnet.sandbox/v1""#), "{s}");
    assert!(s.contains(r#""enforced":false"#), "{s}");
    assert!(s.contains(r#""mode":"allow""#), "{s}");
    assert!(s.contains(r#""socket""#), "{s}");
}

#[test]
fn no_caps_is_pure_compute_deny_all() {
    let dir = fresh("none");
    let p = write(dir.as_path(), "def main() { 0 }\n");
    let out = garnet().arg("sandbox").arg(&p).output().unwrap();
    assert!(out.status.success());
    let s = String::from_utf8(out.stdout).unwrap();
    assert!(s.contains("egress: deny-all"), "{s}");
    assert!(s.contains("preopens=false"), "{s}");
}

#[test]
fn ffi_cap_is_flagged_as_an_escape() {
    let dir = fresh("ffi");
    let p = write(
        dir.as_path(),
        "@caps(ffi)\ndef native() { 0 }\ndef main() { 0 }\n",
    );
    let out = garnet().arg("sandbox").arg(&p).output().unwrap();
    let s = String::from_utf8(out.stdout).unwrap();
    assert!(s.contains("warning:") && s.contains("ffi"), "{s}");
}
