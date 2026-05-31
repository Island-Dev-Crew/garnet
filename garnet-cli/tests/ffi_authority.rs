//! S61 — FFI authority model (CI-gated cross-OS proof).
//!
//! FFI is an explicit, declared, diff-gated, sandbox-flagged authority — not an
//! implicit escape hatch. `examples/ffi/native_boundary.garnet` declares
//! `@caps(ffi)` on the function wrapping a native call; `no_native.garnet` is the
//! capability-free baseline. This proves the model end-to-end via the built CLI
//! (so it runs on every OS in the `cargo test --workspace` matrix).

use std::path::{Path, PathBuf};
use std::process::Command;

fn garnet() -> Command {
    Command::new(env!("CARGO_BIN_EXE_garnet"))
}

fn example(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("examples")
        .join("ffi")
        .join(rel)
}

#[test]
fn declared_ffi_checks_clean() {
    // FFI is not a violation when it is *declared* — both examples check clean.
    for rel in ["no_native.garnet", "native_boundary.garnet"] {
        let out = garnet().arg("check").arg(example(rel)).output().unwrap();
        let s = String::from_utf8(out.stdout).unwrap();
        assert!(out.status.success(), "{rel}: {s}");
        assert!(s.contains("0 diagnostics"), "{rel}: {s}");
    }
}

#[test]
fn sandbox_flags_ffi_as_uncontainable() {
    let out = garnet()
        .arg("sandbox")
        .arg(example("native_boundary.garnet"))
        .output()
        .unwrap();
    let s = String::from_utf8(out.stdout).unwrap();
    assert!(
        s.contains("warning:") && s.contains("ffi") && s.contains("does not contain FFI"),
        "{s}"
    );
}

#[test]
fn diff_caps_flags_gaining_ffi() {
    let out = garnet()
        .arg("diff-caps")
        .arg(example("no_native.garnet"))
        .arg(example("native_boundary.garnet"))
        .output()
        .unwrap();
    let s = String::from_utf8(out.stdout).unwrap();
    assert!(!out.status.success(), "gaining ffi must exit non-zero: {s}");
    assert!(s.contains("caps GAINED") && s.contains("ffi"), "{s}");
    assert!(s.contains("AUTHORITY EXPANDED"), "{s}");
}
