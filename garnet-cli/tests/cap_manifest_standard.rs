//! S98 — capability-manifest standard profile.
//!
//! The default S36 `garnet caps` output remains Garnet's internal manifest.
//! `--standard-profile` emits the language-neutral draft profile seeded for
//! RFC-0001. It is a reference implementation seed, not evidence of standards
//! adoption.

use std::path::{Path, PathBuf};
use std::process::Command;

fn garnet() -> Command {
    Command::new(env!("CARGO_BIN_EXE_garnet"))
}

fn fresh(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("garnet_s98_{tag}"));
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
fn caps_standard_profile_emits_language_neutral_schema() {
    let dir = fresh("profile");
    write(&dir, "load.garnet", "@caps(fs)\ndef load() { 1 }\n");
    write(
        &dir,
        "transmit.garnet",
        "@caps(net)\ndef transmit() { 1 }\n",
    );

    let out = garnet()
        .arg("caps")
        .arg("--standard-profile")
        .arg(&dir)
        .output()
        .unwrap();
    assert!(out.status.success());
    let s = String::from_utf8(out.stdout).unwrap();

    assert!(s.contains(r#""schema":"capability-manifest/v1""#), "{s}");
    assert!(s.contains(r#""status":"draft-reference-seed""#), "{s}");
    assert!(
        s.contains(
            r#""producer":{"name":"garnet","manifest_schema":"garnet-capability-manifest-v1"}"#
        ),
        "{s}"
    );
    assert!(s.contains(r#""aggregate":["fs","net"]"#), "{s}");
    assert!(
        s.contains(r#"{"kind":"function","name":"load","capabilities":["fs"],"source_span":null}"#),
        "{s}"
    );
    assert!(
        s.contains(
            r#"{"kind":"function","name":"transmit","capabilities":["net"],"source_span":null}"#
        ),
        "{s}"
    );
    assert!(
        s.contains(r#""declared-surface only; does not prove absence of undeclared authority""#),
        "{s}"
    );
}

#[test]
fn caps_standard_profile_is_deterministic() {
    let dir = fresh("deterministic");
    let p = write(
        &dir,
        "ordered.garnet",
        "@caps(net)\ndef b() { 1 }\n\n@caps(fs)\ndef a() { 1 }\n",
    );

    let first = garnet()
        .arg("caps")
        .arg("--standard-profile")
        .arg(&p)
        .output()
        .unwrap();
    let second = garnet()
        .arg("caps")
        .arg("--standard-profile")
        .arg(&p)
        .output()
        .unwrap();

    assert!(first.status.success());
    assert_eq!(first.stdout, second.stdout);
    let s = String::from_utf8(first.stdout).unwrap();
    let a = s.find(r#""name":"a""#).unwrap();
    let b = s.find(r#""name":"b""#).unwrap();
    assert!(a < b, "{s}");
}

#[test]
fn caps_standard_profile_does_not_change_default_manifest() {
    let dir = fresh("default");
    let p = write(&dir, "m.garnet", "@caps(fs)\ndef main() { 1 }\n");

    let out = garnet().arg("caps").arg(&p).output().unwrap();
    assert!(out.status.success());
    let s = String::from_utf8(out.stdout).unwrap();

    assert!(
        s.contains(r#""schema":"garnet-capability-manifest-v1""#),
        "{s}"
    );
    assert!(!s.contains(r#""schema":"capability-manifest/v1""#), "{s}");
}
