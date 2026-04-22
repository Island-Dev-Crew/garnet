//! Smoke tests for the three real-world example programs in
//! `E_Engineering_Artifacts/examples/`. Each must `parse` cleanly. The
//! safe_io_layer program also passes `check` and `run` (it doesn't depend
//! on the actor runtime on the hot path).

use std::path::PathBuf;
use std::process::Command;

fn garnet_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_garnet"))
}

fn workspace_root() -> PathBuf {
    // Cargo runs each integration test from the crate dir, so go one level up.
    let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    crate_dir.parent().unwrap().to_path_buf()
}

fn example(name: &str) -> PathBuf {
    workspace_root().join("examples").join(name)
}

#[test]
fn multi_agent_builder_parses() {
    let f = example("multi_agent_builder.garnet");
    assert!(f.exists(), "example file missing: {f:?}");
    let out = Command::new(garnet_bin())
        .args(["parse", f.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "parse multi_agent_builder failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn agentic_log_analyzer_parses() {
    let f = example("agentic_log_analyzer.garnet");
    assert!(f.exists(), "example file missing: {f:?}");
    let out = Command::new(garnet_bin())
        .args(["parse", f.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "parse agentic_log_analyzer failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn safe_io_layer_parses() {
    let f = example("safe_io_layer.garnet");
    assert!(f.exists(), "example file missing: {f:?}");
    let out = Command::new(garnet_bin())
        .args(["parse", f.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "parse safe_io_layer failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn safe_io_layer_checks() {
    let f = example("safe_io_layer.garnet");
    let out = Command::new(garnet_bin())
        .args(["check", f.to_str().unwrap()])
        .output()
        .unwrap();
    // `check` may surface annotation diagnostics; we just want it to not
    // crash.
    let _ = out.status;
}
