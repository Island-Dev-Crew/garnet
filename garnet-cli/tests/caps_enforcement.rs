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

fn run_backend(program: &str, backend: &str) -> std::process::Output {
    let dir = tempfile::TempDir::new().unwrap();
    let path = dir.path().join("prog.garnet");
    std::fs::write(&path, program).unwrap();
    garnet().args(["run", backend]).arg(&path).output().unwrap()
}

fn run_interp(program: &str) -> std::process::Output {
    run_backend(program, "--interp")
}

/// S100: run the same program through the bytecode VM (`garnet run --vm`).
fn run_vm(program: &str) -> std::process::Output {
    run_backend(program, "--vm")
}

/// S100: assert BOTH backends trap identically — same exit code and the same trap
/// substring in each stderr. This is the `@caps` trap-parity assertion that closes
/// the VM authority-laundering seam (before S100 the VM allowed what `--interp`
/// trapped, because the VM established no program-entry caps frame).
fn traps_on_both_with(program: &str, trap_substr: &str) {
    let interp = run_interp(program);
    let vm = run_vm(program);
    assert!(!interp.status.success(), "interp must trap on: {program}");
    assert!(
        !vm.status.success(),
        "VM must trap too (no @caps laundering through --vm) on: {program}"
    );
    assert_eq!(
        interp.status.code(),
        vm.status.code(),
        "interp and VM must agree on the trap exit code"
    );
    for (label, out) in [("interp", &interp), ("vm", &vm)] {
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(
            stderr.contains(trap_substr),
            "{label} stderr missing `{trap_substr}`: {stderr}"
        );
    }
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

fn process_output_program() -> (&'static str, &'static str) {
    if cfg!(windows) {
        ("cmd", r#"["/c", "exit", "0"]"#)
    } else {
        ("true", "[]")
    }
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
fn proc_helper_laundering_traps_when_entry_lacks_proc() {
    let (program, argv) = process_output_program();
    let out = run_interp(&format!(
        r#"
        @caps(proc)
        def helper() {{
          let result = std::process::output("{program}", {argv})
          result.get("code")
        }}

        @caps()
        def main() {{
          helper()
        }}
        "#
    ));
    assert!(
        !out.status.success(),
        "entry without @caps(proc) must not launder subprocess authority through a helper"
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("requires program entry @caps(proc)"),
        "expected program-entry proc trap, got: {stderr}"
    );
}

#[test]
fn proc_helper_runs_when_entry_declares_proc() {
    let (program, argv) = process_output_program();
    let out = run_interp(&format!(
        r#"
        @caps(proc)
        def helper() {{
          let result = std::process::output("{program}", {argv})
          result.get("code")
        }}

        @caps(proc)
        def main() {{
          helper()
        }}
        "#
    ));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        out.status.success(),
        "entry @caps(proc) must allow helper subprocess authority: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(stdout.contains("=> 0"), "got {stdout}");
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

// ---------------------------------------------------------------------------
// S100 — VM / interpreter `@caps` TRAP-parity (close the authority-laundering seam).
//
// Basic `@caps` was already enforced on the VM via the fallback's regular managed
// frame — but the S92 program-entry gate was NOT: the VM established no entry
// frame, so undeclared subprocess authority laundered through a helper that
// declares `@caps(proc)` trapped under `--interp` yet RAN under `--vm` (exit 0).
// S100 installs the program-entry caps frame at the VM's per-run seam, so every
// `@caps` trap — including the entry gate — now fires identically on both backends.
// ---------------------------------------------------------------------------

#[test]
fn vm_undeclared_env_traps_identically() {
    traps_on_both_with(
        "@caps()\ndef main() {\n  std::env::get(\"HOME\")\n}\n",
        "requires @caps(env)",
    );
}

#[test]
fn vm_undeclared_proc_traps_identically() {
    traps_on_both_with(
        "@caps()\ndef main() {\n  std::process::spawn(\"echo\")\n}\n",
        "requires @caps(proc)",
    );
}

#[test]
fn vm_undeclared_fs_traps_identically() {
    traps_on_both_with(
        "@caps()\ndef main() {\n  fs::read_file(\"/etc/hostname\")\n}\n",
        "requires @caps(fs)",
    );
}

#[test]
fn vm_undeclared_net_traps_identically() {
    traps_on_both_with(
        "@caps()\ndef main() {\n  tcp_connect(\"127.0.0.1\", 1)\n}\n",
        "requires @caps(net)",
    );
}

/// The headline: the S92 program-entry gate now fires on the VM too. Before S100
/// this laundered subprocess authority through the `@caps(proc)` helper and RAN
/// under `--vm` (exit 0) while `--interp` trapped — the closed hole.
#[test]
fn vm_entry_caps_not_launderable_through_helper() {
    let (program, argv) = process_output_program();
    traps_on_both_with(
        &format!(
            r#"
            @caps(proc)
            def helper() {{
              let result = std::process::output("{program}", {argv})
              result.get("code")
            }}

            @caps()
            def main() {{
              helper()
            }}
            "#
        ),
        "requires program entry @caps(proc)",
    );
}

#[test]
fn vm_declared_env_runs() {
    // The entry-caps frame must GRANT the declared cap, not blanket-deny.
    let out = run_vm("@caps(env)\ndef main() {\n  std::env::get(\"HOME\")\n}\n");
    assert!(
        out.status.success(),
        "declared @caps(env) must run on the VM: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn vm_pure_computation_is_unaffected() {
    // Installing the entry frame adds no trap to pure native code that never
    // touches a host bridge.
    let out = run_vm("@caps()\ndef main() {\n  1 + 2 * 3\n}\n");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        out.status.success(),
        "pure code must run on the VM: {stdout}"
    );
    assert!(stdout.contains("=> 7"), "got {stdout}");
}
