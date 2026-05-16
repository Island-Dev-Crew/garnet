//! Semantic dogfood checks for the canonical MVP corpus.
//!
//! `examples.rs` proves parse/check/run across the corpus. This file pins the
//! expected observable results so examples cannot quietly devolve into
//! placeholder programs while still exiting successfully.

use std::path::{Path, PathBuf};
use std::process::Command;

fn garnet_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_garnet"))
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace root")
        .to_path_buf()
}

#[test]
fn canonical_mvp_examples_emit_stable_results() {
    let cases = [
        ("mvp_01_os_simulator.garnet", "=> 9"),
        ("mvp_02_relational_db.garnet", "=> 46"),
        ("mvp_03_compiler_bootstrap.garnet", "=> 40"),
        ("mvp_04_numerical_solver.garnet", "=> 0"),
        ("mvp_05_web_app.garnet", "=> 61"),
        ("mvp_06_multi_agent.garnet", "=> 16"),
        ("mvp_07_game_server.garnet", "=> 2"),
        ("mvp_08_distributed_kv.garnet", "=> 57"),
        ("mvp_09_graph_db.garnet", "=> 405"),
        ("mvp_10_terminal_ui.garnet", "=> 560"),
    ];

    for (name, expected) in cases {
        let path = repo_root().join("examples").join(name);
        let out = Command::new(garnet_bin())
            .args(["run", path.to_str().unwrap()])
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "garnet run {name} failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );

        let stdout = String::from_utf8_lossy(&out.stdout);
        assert!(
            stdout.contains(expected),
            "{name} did not emit {expected}\nstdout:\n{stdout}"
        );
    }
}

#[test]
fn advertised_agentic_examples_emit_stable_results() {
    let cases = [
        ("multi_agent_builder.garnet", "=> 46"),
        ("safe_io_layer.garnet", "=> 402"),
        ("agentic_log_analyzer.garnet", "=> 43"),
    ];

    for (name, expected) in cases {
        let path = repo_root().join("examples").join(name);
        let out = Command::new(garnet_bin())
            .args(["run", path.to_str().unwrap()])
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "garnet run {name} failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );

        let stdout = String::from_utf8_lossy(&out.stdout);
        assert!(
            stdout.contains(expected),
            "{name} did not emit {expected}\nstdout:\n{stdout}"
        );
    }
}

#[test]
fn agent_toolbelt_examples_emit_stable_results() {
    let cases = [
        ("agent_toolbelt_01_triage_router.garnet", "=> 91"),
        ("agent_toolbelt_02_capability_budget.garnet", "=> 61"),
        ("agent_toolbelt_03_memory_recall.garnet", "=> 81"),
        ("agent_toolbelt_04_release_gate.garnet", "=> 80"),
        ("agent_toolbelt_05_repair_planner.garnet", "=> 116"),
    ];

    for (name, expected) in cases {
        let path = repo_root().join("examples").join(name);
        let out = Command::new(garnet_bin())
            .args(["run", path.to_str().unwrap()])
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "garnet run {name} failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );

        let stdout = String::from_utf8_lossy(&out.stdout);
        assert!(
            stdout.contains(expected),
            "{name} did not emit {expected}\nstdout:\n{stdout}"
        );
    }
}
