//! S38 — `garnet seal` integration test (runs the built binary).
//!
//! Confirms the in-toto seal predicate is emitted with the expected shape and
//! that the cosign-availability note is present (honest either way).

use std::path::{Path, PathBuf};
use std::process::Command;

fn garnet() -> Command {
    Command::new(env!("CARGO_BIN_EXE_garnet"))
}

fn fresh(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("garnet_s38_{tag}"));
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
fn seal_emits_an_in_toto_statement() {
    let dir = fresh("seal");
    let p = write(dir.as_path(), "app.garnet", "@caps(fs)\ndef main() { 1 }\n");
    let out = garnet().arg("seal").arg(&p).output().unwrap();
    assert!(out.status.success());
    let s = String::from_utf8(out.stdout).unwrap();
    assert!(
        s.contains(r#""_type":"https://in-toto.io/Statement/v1""#),
        "{s}"
    );
    assert!(s.contains(r#""predicateType":"https://garnet-lang.org/attestation/seal/v1""#));
    assert!(s.contains(r#""build_manifest":{"#), "{s}");
    assert!(
        s.contains(r#""capability_manifest":{"schema":"garnet-capability-manifest-v1""#),
        "{s}"
    );
    assert!(s.contains(r#""aggregate":["fs"]"#), "{s}");
}

#[test]
fn seal_reports_cosign_availability_on_stderr() {
    // The seal wrapper always notes cosign's presence/absence (honest either way).
    let dir = fresh("seal_cosign");
    let p = write(dir.as_path(), "app.garnet", "@caps()\ndef main() { 1 }\n");
    let out = garnet().arg("seal").arg(&p).output().unwrap();
    assert!(out.status.success());
    let err = String::from_utf8(out.stderr).unwrap();
    assert!(
        err.contains("cosign"),
        "stderr should mention cosign: {err}"
    );
}
