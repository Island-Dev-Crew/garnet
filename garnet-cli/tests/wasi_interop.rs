//! S64 — WASI interop (CI-gated cross-OS authority-mapping proof).
//!
//! A Garnet program's `@caps` are exactly the WASI host capabilities it would
//! request. `examples/ffi/wasi_clock.garnet` declares `@caps(time, fs)`; this
//! proves the sandbox WASI policy reflects them (clocks + preopens, not sockets).
//! The WASI runtime is deferred — this is the authority mapping, cross-OS via the
//! `cargo test --workspace` matrix.

use std::path::{Path, PathBuf};
use std::process::Command;

fn garnet() -> Command {
    Command::new(env!("CARGO_BIN_EXE_garnet"))
}

fn wasi_clock() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("examples/ffi/wasi_clock.garnet")
}

#[test]
fn wasi_program_checks_clean() {
    let out = garnet().arg("check").arg(wasi_clock()).output().unwrap();
    let s = String::from_utf8(out.stdout).unwrap();
    assert!(out.status.success() && s.contains("0 diagnostics"), "{s}");
}

#[test]
fn wasi_policy_reflects_declared_caps() {
    let out = garnet()
        .args(["sandbox", "--format", "json"])
        .arg(wasi_clock())
        .output()
        .unwrap();
    let s = String::from_utf8(out.stdout).unwrap();
    // @caps(time, fs) -> clocks + preopens granted; sockets NOT (no net declared).
    assert!(s.contains(r#""clocks":true"#), "{s}");
    assert!(s.contains(r#""preopens":true"#), "{s}");
    assert!(s.contains(r#""sockets":false"#), "{s}");
}
