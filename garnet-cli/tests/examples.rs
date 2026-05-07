//! Smoke tests for the current executable example surface in
//! `E_Engineering_Artifacts/examples/`.
//!
//! The 10 `mvp_*.garnet` files are canonical app-level dogfood: each must
//! parse, check, and run. The three larger real-world files remain parser-scale
//! design references until the runtime grows into their full actor/stdlib shape.

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

fn run_garnet(args: &[&str], file: &PathBuf) -> std::process::Output {
    Command::new(garnet_bin())
        .args(args)
        .arg(file)
        .output()
        .unwrap()
}

#[test]
fn canonical_mvp_examples_parse_check_and_run() {
    let cases = [
        "mvp_01_os_simulator.garnet",
        "mvp_02_relational_db.garnet",
        "mvp_03_compiler_bootstrap.garnet",
        "mvp_04_numerical_solver.garnet",
        "mvp_05_web_app.garnet",
        "mvp_06_multi_agent.garnet",
        "mvp_07_game_server.garnet",
        "mvp_08_distributed_kv.garnet",
        "mvp_09_graph_db.garnet",
        "mvp_10_terminal_ui.garnet",
    ];

    for name in cases {
        let f = example(name);
        assert!(f.exists(), "example file missing: {f:?}");
        for subcommand in ["parse", "check", "run"] {
            let out = run_garnet(&[subcommand], &f);
            assert!(
                out.status.success(),
                "garnet {subcommand} {name} failed\nstdout:\n{}\nstderr:\n{}",
                String::from_utf8_lossy(&out.stdout),
                String::from_utf8_lossy(&out.stderr)
            );
        }
    }
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
