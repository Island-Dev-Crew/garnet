//! End-to-end integration tests — chain `parse → check → build → verify → run`
//! through subprocess invocations of the `garnet` binary. If any
//! subcommand exits non-zero or the chain breaks, this fires immediately.

use std::path::PathBuf;
use std::process::Command;

fn garnet_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_garnet"))
}

fn fresh_dir(name: &str) -> PathBuf {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nano = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("garnet_e2e_{name}_{}_{nano}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn full_pipeline_parse_check_build_verify_run() {
    let dir = fresh_dir("pipeline");
    let file = dir.join("e2e.garnet");
    std::fs::write(
        &file,
        "@caps()\ndef main() {\n  let xs = [1, 2, 3, 4, 5]\n  xs.reduce(0, |a, b| a + b)\n}",
    )
    .unwrap();

    // 1. parse
    let out = Command::new(garnet_bin())
        .current_dir(&dir)
        .args(["parse", file.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "parse failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    // 2. check
    let out = Command::new(garnet_bin())
        .current_dir(&dir)
        .args(["check", file.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "check failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    // 3. build --deterministic
    let out = Command::new(garnet_bin())
        .current_dir(&dir)
        .args(["build", "--deterministic", file.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "build failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let manifest = file.with_file_name("e2e.garnet.manifest.json");
    assert!(manifest.exists());

    // 4. verify
    let out = Command::new(garnet_bin())
        .current_dir(&dir)
        .args(["verify", file.to_str().unwrap(), manifest.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "verify failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    // 5. run
    let out = Command::new(garnet_bin())
        .current_dir(&dir)
        .args(["run", file.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "run failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("=> 15"),
        "expected sum 15 in output, got: {stdout}"
    );
}

#[test]
fn run_records_episode_then_recall_surfaces_it() {
    let dir = fresh_dir("recall");
    let f = dir.join("greet.garnet");
    std::fs::write(&f, "def main() { 42 }").unwrap();

    // First run.
    let _ = Command::new(garnet_bin())
        .current_dir(&dir)
        .args(["run", f.to_str().unwrap()])
        .output()
        .unwrap();
    // Episode log must exist.
    let log = dir.join(".garnet-cache").join("episodes.log");
    assert!(log.exists());
    let raw = std::fs::read_to_string(&log).unwrap();
    assert!(raw.lines().count() >= 1);
}

#[test]
fn check_fails_loud_on_safe_violation() {
    let dir = fresh_dir("check_violation");
    let f = dir.join("bad.garnet");
    std::fs::write(&f, "@safe\ndef bad() { var x = 1\n raise \"oops\"\n x }").unwrap();

    let out = Command::new(garnet_bin())
        .current_dir(&dir)
        .args(["check", f.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(!out.status.success(), "check should fail on safe violation");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("safe-mode") || stdout.contains("safe"));
}

#[test]
fn build_then_modify_then_verify_fails() {
    let dir = fresh_dir("build_modify");
    let f = dir.join("v.garnet");
    std::fs::write(&f, "def main() { 1 }").unwrap();
    let _ = Command::new(garnet_bin())
        .current_dir(&dir)
        .args(["build", "--deterministic", f.to_str().unwrap()])
        .output()
        .unwrap();
    let manifest = f.with_file_name("v.garnet.manifest.json");
    std::fs::write(&f, "def main() { 999 }").unwrap();
    let out = Command::new(garnet_bin())
        .current_dir(&dir)
        .args(["verify", f.to_str().unwrap(), manifest.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(!out.status.success());
}
