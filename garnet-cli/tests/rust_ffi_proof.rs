//! S62 — Rust FFI proof (CI-gated cross-OS attestation proof).
//!
//! A `@caps(ffi)` Garnet function wrapping a Rust `extern "C"` symbol is a
//! first-class, attested authority: it checks clean, runs, and `garnet seal`
//! emits an in-toto predicate whose capability manifest attests `ffi`. The
//! binding *runtime* (value ↔ C ABI marshalling) is deferred — this proves the
//! authority/attestation half, cross-OS via the `cargo test --workspace` matrix.

use std::path::{Path, PathBuf};
use std::process::Command;

fn garnet() -> Command {
    Command::new(env!("CARGO_BIN_EXE_garnet"))
}

fn rust_extern() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("examples/ffi/rust_extern.garnet")
}

#[test]
fn rust_wrapper_checks_clean_and_runs() {
    let check = garnet().arg("check").arg(rust_extern()).output().unwrap();
    let cs = String::from_utf8(check.stdout).unwrap();
    assert!(
        check.status.success() && cs.contains("0 diagnostics"),
        "{cs}"
    );

    let run = garnet().arg("run").arg(rust_extern()).output().unwrap();
    let rs = String::from_utf8(run.stdout).unwrap();
    assert!(run.status.success(), "{rs}");
    assert!(rs.contains("payload"), "{rs}");
}

#[test]
fn seal_attests_the_ffi_authority() {
    let out = garnet().arg("seal").arg(rust_extern()).output().unwrap();
    assert!(out.status.success());
    let s = String::from_utf8(out.stdout).unwrap();
    assert!(
        s.contains(r#""predicateType":"https://garnet-lang.org/attestation/seal/v1""#),
        "{s}"
    );
    // The embedded capability manifest must attest `ffi`.
    assert!(s.contains(r#""aggregate":["ffi"]"#), "{s}");
}
