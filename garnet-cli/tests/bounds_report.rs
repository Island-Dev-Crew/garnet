//! S39 — `garnet bounds` integration test (runs the built binary).

use std::path::{Path, PathBuf};
use std::process::Command;

fn garnet() -> Command {
    Command::new(env!("CARGO_BIN_EXE_garnet"))
}

fn fresh(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("garnet_s39_{tag}"));
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
fn reports_declared_fuel_budgets_with_honest_deferral() {
    let dir = fresh("report");
    let p = write(
        dir.as_path(),
        "app.garnet",
        "@bounded(1000)\ndef f() { 1 }\n@caps()\ndef main() { 1 }\n",
    );
    let out = garnet().arg("bounds").arg(&p).output().unwrap();
    assert!(out.status.success());
    let s = String::from_utf8(out.stdout).unwrap();
    assert!(s.contains("f: 1000 fuel units"), "{s}");
    assert!(s.contains("Wasmtime-fuel"), "{s}");
    assert!(
        s.contains("not yet runtime fuel-enforced"),
        "the honest deferral note must be present: {s}"
    );
}

#[test]
fn no_bounded_reports_none() {
    let dir = fresh("none");
    let p = write(dir.as_path(), "app.garnet", "@caps()\ndef main() { 1 }\n");
    let out = garnet().arg("bounds").arg(&p).output().unwrap();
    assert!(out.status.success());
    let s = String::from_utf8(out.stdout).unwrap();
    assert!(s.contains("no @bounded"), "{s}");
}
