//! S40 — `garnet ceilings` integration test (runs the built binary).

use std::path::{Path, PathBuf};
use std::process::Command;

fn garnet() -> Command {
    Command::new(env!("CARGO_BIN_EXE_garnet"))
}

fn fresh(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("garnet_s40_{tag}"));
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
fn reports_explosive_ops_with_governance_and_defaults() {
    let dir = fresh("report");
    let p = write(
        dir.as_path(),
        "app.garnet",
        "@bounded(1000)\ndef worker() { loop { break } }\ndef fanout() { spawn worker() }\n",
    );
    let out = garnet().arg("ceilings").arg(&p).output().unwrap();
    assert!(out.status.success());
    let s = String::from_utf8(out.stdout).unwrap();
    // worker's loop is governed by @bounded; fanout's spawn falls back to default.
    assert!(
        s.contains("unconditional loop") && s.contains("governed by @bounded"),
        "{s}"
    );
    assert!(
        s.contains("spawn") && s.contains("DEFAULT fan-out ceiling"),
        "{s}"
    );
    assert!(
        s.contains("no ceiling is faked"),
        "the honest deferral note must be present: {s}"
    );
}

#[test]
fn clean_file_reports_no_explosive_ops() {
    let dir = fresh("clean");
    let p = write(
        dir.as_path(),
        "app.garnet",
        "@caps()\ndef main() { 1 + 2 }\n",
    );
    let out = garnet().arg("ceilings").arg(&p).output().unwrap();
    assert!(out.status.success());
    let s = String::from_utf8(out.stdout).unwrap();
    assert!(s.contains("no explosive operations"), "{s}");
}
