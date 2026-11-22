//! End-to-end regression tests for C1 / BE-F1: enforcement traps (`@caps`
//! refusal and `@max_depth` breach) must be uncatchable by `try`/`rescue` and
//! un-maskable by `ensure` on BOTH `--interp` and `--vm`, and `garnet
//! agent-loop` must reject and refuse to seal a proposal that catches a trap.

use std::process::Command;

fn garnet_bin() -> &'static str {
    env!("CARGO_BIN_EXE_garnet")
}

const BE_F1_RESCUED_CAPS: &str = r#"
@caps(env)
def peek() {
    std::env::get("HOME")
}

@caps()
def main() {
    let f = peek
    try {
        f()
    } rescue e {
        "swallowed"
    }
}
"#;

const BE_F1_ENSURE_RETURN_BYPASS: &str = r#"
@caps(env)
def peek() {
    std::env::get("HOME")
}

@caps()
def main() {
    let f = peek
    try {
        f()
    } rescue e {
        "swallowed"
    } ensure {
        return "bypassed"
    }
}
"#;

const RESCUED_MAX_DEPTH: &str = r#"
@max_depth(3)
@caps()
def deep(n) {
    if n <= 0 { 0 } else { deep(n - 1) }
}

@caps()
def main() {
    try {
        deep(10)
    } rescue e {
        "swallowed"
    }
}
"#;

const BASELINE_PURE: &str = r#"
@caps()
def main() {
    "ok"
}
"#;

#[test]
fn be_f1_rescued_caps_trap_cannot_be_swallowed_on_interp_or_vm() {
    let dir = tempfile::tempdir().expect("tempdir");
    let prog = dir.path().join("rescued_caps.garnet");
    std::fs::write(&prog, BE_F1_RESCUED_CAPS).expect("write");

    for backend in ["--interp", "--vm"] {
        let out = Command::new(garnet_bin())
            .args(["run", backend])
            .arg(&prog)
            .output()
            .expect("spawn garnet run");
        let stdout = String::from_utf8_lossy(&out.stdout);
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(
            !out.status.success(),
            "backend {backend} exited 0 on rescued @caps trap! stdout={stdout:?} stderr={stderr:?}"
        );
        assert!(
            !stdout.contains("swallowed"),
            "backend {backend} swallowed @caps trap! stdout={stdout:?}"
        );
        assert!(
            stderr.contains("capability:") && stderr.contains("@caps(env)"),
            "backend {backend} stderr missing capability trap detail: {stderr:?}"
        );
    }
}

#[test]
fn be_f1_ensure_return_cannot_mask_caps_trap() {
    let dir = tempfile::tempdir().expect("tempdir");
    let prog = dir.path().join("ensure_bypass.garnet");
    std::fs::write(&prog, BE_F1_ENSURE_RETURN_BYPASS).expect("write");

    for backend in ["--interp", "--vm"] {
        let out = Command::new(garnet_bin())
            .args(["run", backend])
            .arg(&prog)
            .output()
            .expect("spawn garnet run");
        let stdout = String::from_utf8_lossy(&out.stdout);
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(
            !out.status.success(),
            "backend {backend} let ensure return mask @caps trap! stdout={stdout:?} stderr={stderr:?}"
        );
        assert!(
            !stdout.contains("bypassed") && !stdout.contains("swallowed"),
            "backend {backend} printed masked value: stdout={stdout:?}"
        );
    }
}

#[test]
fn be_f1_rescued_max_depth_trap_cannot_be_swallowed_on_interp_or_vm() {
    let dir = tempfile::tempdir().expect("tempdir");
    let prog = dir.path().join("rescued_depth.garnet");
    std::fs::write(&prog, RESCUED_MAX_DEPTH).expect("write");

    for backend in ["--interp", "--vm"] {
        let out = Command::new(garnet_bin())
            .args(["run", backend])
            .arg(&prog)
            .output()
            .expect("spawn garnet run");
        let stdout = String::from_utf8_lossy(&out.stdout);
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(
            !out.status.success(),
            "backend {backend} exited 0 on rescued @max_depth trap! stdout={stdout:?} stderr={stderr:?}"
        );
        assert!(
            !stdout.contains("swallowed"),
            "backend {backend} swallowed @max_depth trap! stdout={stdout:?}"
        );
        assert!(
            stderr.contains("@max_depth(3) exceeded"),
            "backend {backend} stderr missing @max_depth detail: {stderr:?}"
        );
    }
}

#[test]
fn be_f1_agent_loop_refuses_to_seal_rescued_caps_trap() {
    let dir = tempfile::tempdir().expect("tempdir");
    let baseline = dir.path().join("baseline.garnet");
    let proposal = dir.path().join("proposal.garnet");
    std::fs::write(&baseline, BASELINE_PURE).expect("write baseline");
    std::fs::write(&proposal, BE_F1_RESCUED_CAPS).expect("write proposal");

    for backend in ["interp", "vm"] {
        let seal = dir.path().join(format!("proposal-{backend}.seal.json"));
        let out = Command::new(garnet_bin())
            .args(["agent-loop", "--baseline"])
            .arg(&baseline)
            .arg("--proposal")
            .arg(&proposal)
            .args(["--backend", backend, "--seal-out"])
            .arg(&seal)
            .output()
            .expect("spawn garnet agent-loop");
        let stdout = String::from_utf8_lossy(&out.stdout);
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(
            !out.status.success(),
            "agent-loop --backend {backend} accepted rescued @caps trap! stdout={stdout:?} stderr={stderr:?}"
        );
        assert!(
            !seal.exists(),
            "agent-loop --backend {backend} wrote seal file despite trap!"
        );
    }
}

#[test]
fn ordinary_user_exceptions_remain_rescuable() {
    let dir = tempfile::tempdir().expect("tempdir");
    let prog = dir.path().join("normal_rescue.garnet");
    std::fs::write(
        &prog,
        r#"
@caps()
def main() {
    try {
        raise "boom"
    } rescue e {
        "caught"
    }
}
"#,
    )
    .expect("write");

    let out = Command::new(garnet_bin())
        .args(["run", "--interp"])
        .arg(&prog)
        .output()
        .expect("spawn garnet run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        out.status.success(),
        "ordinary raise should still be rescued"
    );
    assert!(stdout.contains("=> caught"), "stdout={stdout:?}");
}
