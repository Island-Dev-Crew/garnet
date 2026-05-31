//! S41 — `garnet concurrency` integration test (runs the built binary).

use std::path::{Path, PathBuf};
use std::process::Command;

fn garnet() -> Command {
    Command::new(env!("CARGO_BIN_EXE_garnet"))
}

fn fresh(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("garnet_s41_{tag}"));
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
fn reports_actor_protocols_ask_and_tell() {
    let dir = fresh("actors");
    let p = write(
        dir.as_path(),
        "app.garnet",
        "actor Counter {\n  protocol incr()\n  protocol get() -> Int\n  on incr() { 1 }\n  on get() { 0 }\n}\n",
    );
    let out = garnet().arg("concurrency").arg(&p).output().unwrap();
    assert!(out.status.success());
    let s = String::from_utf8(out.stdout).unwrap();
    assert!(s.contains("actor Counter"), "{s}");
    assert!(s.contains("get/0: ask"), "get() -> Int is an ask: {s}");
    assert!(s.contains("incr/0: tell"), "incr() is a tell: {s}");
    assert!(s.contains("BOUNDED mpsc mailbox"), "model note: {s}");
}

#[test]
fn no_actors_reports_none() {
    let dir = fresh("none");
    let p = write(dir.as_path(), "app.garnet", "@caps()\ndef main() { 1 }\n");
    let out = garnet().arg("concurrency").arg(&p).output().unwrap();
    assert!(out.status.success());
    let s = String::from_utf8(out.stdout).unwrap();
    assert!(s.contains("no actors declared"), "{s}");
}
