//! S63 — C ABI proof (CI-gated cross-OS compound-authority proof).
//!
//! A C binding that touches the filesystem needs BOTH `@caps(ffi)` (it is
//! native) AND `@caps(fs)`. `examples/ffi/c_stat.garnet` declares both; this
//! proves the authority model surfaces, sandboxes, and seals compound native
//! authority — no native call smuggles in an undeclared authority. Runs on every
//! OS in the `cargo test --workspace` matrix.

use std::path::{Path, PathBuf};
use std::process::Command;

fn garnet() -> Command {
    Command::new(env!("CARGO_BIN_EXE_garnet"))
}

fn c_stat() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("examples/ffi/c_stat.garnet")
}

#[test]
fn compound_caps_check_clean() {
    let out = garnet().arg("check").arg(c_stat()).output().unwrap();
    let s = String::from_utf8(out.stdout).unwrap();
    assert!(out.status.success() && s.contains("0 diagnostics"), "{s}");
}

#[test]
fn sandbox_surfaces_both_ffi_and_fs() {
    let out = garnet()
        .args(["sandbox", "--format", "json"])
        .arg(c_stat())
        .output()
        .unwrap();
    let s = String::from_utf8(out.stdout).unwrap();
    // fs grants WASI preopens; ffi raises the escape-hatch warning.
    assert!(s.contains(r#""preopens":true"#), "{s}");
    assert!(s.contains("does not contain FFI"), "{s}");
    assert!(s.contains(r#""ffi""#) && s.contains(r#""fs""#), "{s}");
}

#[test]
fn seal_attests_both_authorities() {
    let out = garnet().arg("seal").arg(c_stat()).output().unwrap();
    let s = String::from_utf8(out.stdout).unwrap();
    assert!(out.status.success());
    assert!(s.contains(r#""aggregate":["ffi","fs"]"#), "{s}");
}
