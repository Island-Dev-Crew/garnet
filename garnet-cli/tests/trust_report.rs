//! S7 — Integration test for `garnet trust-report`.
//!
//! Runs the contract's exact dogfood block against the
//! `examples/agent_orchestrator_3thread.garnet` fixture and asserts the
//! literal `actors: 3 / threads: 3` line appears in stdout. Keeps the
//! S7 surface enforced by every `cargo test --workspace` invocation.

use std::path::PathBuf;
use std::process::Command;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("garnet-cli has a workspace parent")
        .to_path_buf()
}

fn garnet_binary() -> PathBuf {
    let target = std::env::var("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| workspace_root().join("target"));
    let profile = if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    };
    target.join(profile).join("garnet")
}

fn ensure_binary_built() {
    let binary = garnet_binary();
    if binary.exists() {
        return;
    }
    let status = Command::new("cargo")
        .args(["build", "-p", "garnet-cli"])
        .current_dir(workspace_root())
        .status()
        .expect("cargo build invocation");
    assert!(status.success(), "cargo build -p garnet-cli failed");
}

#[test]
fn trust_report_on_three_thread_fixture_reports_three_threads() {
    ensure_binary_built();
    let fixture = workspace_root().join("examples/agent_orchestrator_3thread.garnet");
    assert!(
        fixture.exists(),
        "S7 fixture {} must exist on every PR",
        fixture.display()
    );
    let output = Command::new(garnet_binary())
        .arg("trust-report")
        .arg(&fixture)
        .output()
        .expect("running garnet trust-report");
    assert!(
        output.status.success(),
        "garnet trust-report exited non-zero: stderr={}",
        String::from_utf8_lossy(&output.stderr),
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("actors: 3 / threads: 3"),
        "S7 dogfood literal missing from stdout:\n{stdout}",
    );
}

#[test]
fn trust_report_lists_three_named_actors() {
    ensure_binary_built();
    let fixture = workspace_root().join("examples/agent_orchestrator_3thread.garnet");
    let output = Command::new(garnet_binary())
        .arg("trust-report")
        .arg(&fixture)
        .output()
        .expect("running garnet trust-report");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    for actor in ["Researcher", "Synthesizer", "Reviewer"] {
        assert!(
            stdout.contains(actor),
            "S7 actor {actor} missing from trust-report stdout:\n{stdout}"
        );
    }
}
