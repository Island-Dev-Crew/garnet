//! S38 — `garnet seal` integration test (runs the built binary).
//!
//! Confirms the in-toto seal predicate is emitted with the expected shape and
//! that the cosign-availability note is present (explicit either way).

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
    assert!(s.contains(r#""predicateType":"https://garnet-lang.org/attestation/seal/v2""#));
    assert!(s.contains(r#""build_manifest":{"#), "{s}");
    assert!(
        s.contains(r#""capability_manifest":{"schema":"garnet-capability-manifest-v1""#),
        "{s}"
    );
    assert!(s.contains(r#""aggregate":["fs"]"#), "{s}");
}

#[test]
fn seal_out_writes_the_predicate_to_a_file() {
    // S51: --out writes the predicate so it can feed `cosign attest --predicate`.
    let dir = fresh("seal_out");
    let p = write(
        dir.as_path(),
        "app.garnet",
        "@caps(net)\ndef main() { 1 }\n",
    );
    let out_path = dir.join("predicate.json");
    let out = garnet()
        .arg("seal")
        .arg(&p)
        .arg("--out")
        .arg(&out_path)
        .output()
        .unwrap();
    assert!(out.status.success());
    // Nothing on stdout (the predicate went to the file, not the console).
    assert!(String::from_utf8(out.stdout).unwrap().trim().is_empty());
    let written = std::fs::read_to_string(&out_path).expect("predicate file written");
    assert!(
        written.contains(r#""_type":"https://in-toto.io/Statement/v1""#),
        "{written}"
    );
    assert!(written.contains(r#""aggregate":["net"]"#), "{written}");
    // The cosign hint now points at the real path.
    let err = String::from_utf8(out.stderr).unwrap();
    assert!(err.contains("predicate written to"), "{err}");
}

#[test]
fn seal_reports_cosign_availability_on_stderr() {
    // The seal wrapper always notes cosign's presence/absence (explicit either way).
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

// ── T5a (C2-07): the seal never signs, so its bytes never depend on cosign ──

/// A directory holding a stub `cosign` that answers `cosign version`, so
/// `cosign_available()` is true without a real signer installed.
#[cfg(unix)]
fn stub_cosign_dir(tag: &str) -> PathBuf {
    use std::os::unix::fs::PermissionsExt;
    let dir = fresh(tag);
    let stub = dir.join("cosign");
    std::fs::write(&stub, "#!/bin/sh\necho 'cosign stub v0'\nexit 0\n").unwrap();
    std::fs::set_permissions(&stub, std::fs::Permissions::from_mode(0o755)).unwrap();
    dir
}

#[cfg(unix)]
fn path_with(first: &Path) -> std::ffi::OsString {
    let mut dirs = vec![first.to_path_buf()];
    if let Some(existing) = std::env::var_os("PATH") {
        dirs.extend(std::env::split_paths(&existing));
    }
    std::env::join_paths(dirs).unwrap()
}

#[cfg(unix)]
#[test]
fn seal_bytes_are_identical_with_and_without_cosign() {
    let dir = fresh("seal_no_cosign_dep");
    let p = write(dir.as_path(), "app.garnet", "@caps(fs)\ndef main() { 1 }\n");
    let stub = stub_cosign_dir("seal_cosign_stub");
    let without = garnet()
        .arg("seal")
        .arg(&p)
        .env("PATH", "/usr/bin:/bin")
        .output()
        .unwrap();
    let with = garnet()
        .arg("seal")
        .arg(&p)
        .env("PATH", path_with(&stub))
        .output()
        .unwrap();
    assert!(without.status.success() && with.status.success());
    assert_eq!(
        String::from_utf8(without.stdout).unwrap(),
        String::from_utf8(with.stdout).unwrap(),
        "an installed cosign must not change the seal bytes"
    );
    let err = String::from_utf8(with.stderr).unwrap();
    assert!(
        err.contains("UNSIGNED"),
        "stderr must say UNSIGNED even when cosign is installed: {err}"
    );
}

#[test]
fn seal_predicate_always_says_it_is_unsigned() {
    let dir = fresh("seal_signed_false");
    let p = write(dir.as_path(), "app.garnet", "@caps()\ndef main() { 1 }\n");
    let out = garnet().arg("seal").arg(&p).output().unwrap();
    assert!(out.status.success());
    let s = String::from_utf8(out.stdout).unwrap();
    assert!(s.contains(r#""signed":false"#), "{s}");
}

#[test]
fn verify_rejects_the_retired_cosign_available_variant() {
    let dir = fresh("seal_old_variant");
    let p = write(dir.as_path(), "app.garnet", "@caps()\ndef main() { 1 }\n");
    let out = garnet().arg("seal").arg(&p).output().unwrap();
    assert!(out.status.success());
    let sealed = String::from_utf8(out.stdout).unwrap();
    let start = sealed
        .find("\"cosign\":\"")
        .expect("tooling.cosign present")
        + "\"cosign\":\"".len();
    let end = start + sealed[start..].find('"').expect("closing quote");
    let old_variant = format!(
        "{}{}{}",
        &sealed[..start],
        "available — sign with: cosign attest --predicate <file> --type custom",
        &sealed[end..]
    );
    let seal_path = write(dir.as_path(), "app.seal.json", &old_variant);
    let verify = garnet()
        .arg("verify")
        .arg(&p)
        .arg(&seal_path)
        .output()
        .unwrap();
    assert!(
        !verify.status.success(),
        "a seal whose bytes depend on cosign availability must be rejected"
    );
}
