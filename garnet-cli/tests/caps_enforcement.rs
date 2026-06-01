//! S90 — `@caps` host-authority runtime enforcement seed.
//!
//! The interpreter now *enforces* declared `@caps` at the host-authority
//! boundary: a managed function may only invoke `std::env`/`std::process`/`fs::`/
//! `std::log::to_file` primitives whose required capability some frame in the
//! call chain declared (`garnet run` does not run the static checker, so this is
//! the runtime backstop). Honest scope: host-authority surfaces only; pure
//! computation is unaffected. Runs on every OS in the matrix.

use std::process::Command;

fn garnet() -> Command {
    Command::new(env!("CARGO_BIN_EXE_garnet"))
}

fn run_interp(program: &str) -> std::process::Output {
    let dir = tempfile::TempDir::new().unwrap();
    let path = dir.path().join("prog.garnet");
    std::fs::write(&path, program).unwrap();
    garnet()
        .args(["run", "--interp"])
        .arg(&path)
        .output()
        .unwrap()
}

fn traps_with(program: &str, needs: &str) {
    let out = run_interp(program);
    assert!(!out.status.success(), "must trap (non-zero exit)");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains(&format!("requires @caps({needs})")),
        "expected an undeclared-{needs} capability trap, got: {stderr}"
    );
}

#[test]
fn undeclared_env_traps() {
    traps_with(
        "@caps()\ndef main() {\n  std::env::get(\"HOME\")\n}\n",
        "env",
    );
}

#[test]
fn declared_env_runs() {
    let out = run_interp("@caps(env)\ndef main() {\n  std::env::get(\"HOME\")\n}\n");
    assert!(
        out.status.success(),
        "declared @caps(env) must run: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn undeclared_proc_traps_before_spawning() {
    // The trap is the first statement of the bridge, so no process is spawned.
    traps_with(
        "@caps()\ndef main() {\n  std::process::spawn(\"echo\")\n}\n",
        "proc",
    );
}

#[test]
fn undeclared_fs_traps() {
    traps_with(
        "@caps()\ndef main() {\n  fs::read_file(\"/etc/hostname\")\n}\n",
        "fs",
    );
}

#[test]
fn undeclared_net_traps_before_connect_policy() {
    // S91: the net bridge must reject undeclared authority before the host
    // network policy evaluates the address.
    traps_with(
        "@caps()\ndef main() {\n  tcp_connect(\"127.0.0.1\", 1)\n}\n",
        "net",
    );
}

#[test]
fn program_entry_frame_traps_safe_main_env_without_caps() {
    // S91: safe-mode `fn main` does not push a managed frame, so the program
    // entry frame must provide the runtime caps context.
    traps_with(
        "@caps()\nfn main() -> String {\n  std::env::get(\"HOME\")\n}\n",
        "env",
    );
}

#[test]
fn program_entry_frame_allows_safe_main_declared_env() {
    let out = run_interp("@caps(env)\nfn main() -> String {\n  std::env::get(\"HOME\")\n}\n");
    assert!(
        out.status.success(),
        "program-entry @caps(env) frame must allow safe main env read: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

/// Pure computation declaring no caps is completely unaffected by the enforcement.
#[test]
fn pure_computation_is_unaffected() {
    let out = run_interp("@caps()\ndef main() {\n  1 + 2 * 3\n}\n");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success(), "pure code must run: {stdout}");
    assert!(stdout.contains("=> 7"), "got {stdout}");
}
