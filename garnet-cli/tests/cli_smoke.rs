//! CLI smoke tests — invoke the `garnet` binary as a subprocess and verify
//! the user-visible contract for each subcommand.

use std::path::PathBuf;
use std::process::Command;

fn garnet_bin() -> PathBuf {
    // CARGO_BIN_EXE_garnet is set by cargo when running integration tests
    // for the binary crate.
    PathBuf::from(env!("CARGO_BIN_EXE_garnet"))
}

#[test]
fn no_args_prints_help_and_exits_success() {
    let out = Command::new(garnet_bin()).output().unwrap();
    assert!(out.status.success(), "garnet (no args) should exit 0");
    assert!(String::from_utf8_lossy(&out.stdout).contains("USAGE"));
}

#[test]
fn help_subcommand_prints_subcommand_table() {
    let out = Command::new(garnet_bin()).arg("help").output().unwrap();
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("parse"));
    assert!(s.contains("check"));
    assert!(s.contains("run"));
    assert!(s.contains("eval"));
    assert!(s.contains("repl"));
    assert!(s.contains("version"));
}

#[test]
fn version_prints_version_line() {
    let out = Command::new(garnet_bin()).arg("version").output().unwrap();
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains(&format!("garnet {}", env!("CARGO_PKG_VERSION"))));
    assert!(s.contains("parser"));
    assert!(s.contains("interp"));
    assert!(s.contains("check"));
    assert!(s.contains("memory"));
}

#[test]
fn unknown_subcommand_exits_nonzero() {
    let out = Command::new(garnet_bin())
        .arg("frobnicate")
        .output()
        .unwrap();
    assert!(!out.status.success());
}

#[test]
fn parse_without_file_argument_exits_nonzero() {
    let out = Command::new(garnet_bin()).arg("parse").output().unwrap();
    assert!(!out.status.success());
}

#[test]
fn check_without_file_argument_exits_nonzero() {
    let out = Command::new(garnet_bin()).arg("check").output().unwrap();
    assert!(!out.status.success());
}

#[test]
fn run_without_file_argument_exits_nonzero() {
    let out = Command::new(garnet_bin()).arg("run").output().unwrap();
    assert!(!out.status.success());
}

#[test]
fn eval_without_expression_argument_exits_nonzero() {
    let out = Command::new(garnet_bin()).arg("eval").output().unwrap();
    assert!(!out.status.success());
}

#[test]
fn eval_simple_arithmetic() {
    let out = Command::new(garnet_bin())
        .args(["eval", "1 + 2 * 3"])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert!(String::from_utf8_lossy(&out.stdout).trim().contains("7"));
}

#[test]
fn eval_pipeline_expression() {
    let out = Command::new(garnet_bin())
        .args(["eval", "[1, 2, 3].reduce(0, |a, b| a + b)"])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert!(String::from_utf8_lossy(&out.stdout).trim().contains("6"));
}

#[test]
fn eval_nonsense_exits_nonzero() {
    let out = Command::new(garnet_bin())
        .args(["eval", "totally not garnet code @!@#"])
        .output()
        .unwrap();
    assert!(!out.status.success());
}

#[test]
fn parse_nonexistent_file_exits_nonzero() {
    let out = Command::new(garnet_bin())
        .args(["parse", "this_file_does_not_exist.garnet"])
        .output()
        .unwrap();
    assert!(!out.status.success());
}
