//! S68 — capability transparency log stub (CI-gated cross-OS proof).
//!
//! `garnet caps-log <file> --log <path>` appends a BLAKE3-chained entry;
//! `--verify` confirms the chain (exit 0) and detects tampering (exit 1). Runs on
//! every OS in the `cargo test --workspace` matrix.

use std::path::{Path, PathBuf};
use std::process::Command;

fn garnet() -> Command {
    Command::new(env!("CARGO_BIN_EXE_garnet"))
}

fn example(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("examples")
        .join(rel)
}

fn fresh_log(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("garnet_s68_{tag}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir.join("caps.log")
}

#[test]
fn append_then_verify_intact_chain() {
    let log = fresh_log("intact");
    for f in ["hello.garnet", "ffi/c_stat.garnet"] {
        let out = garnet()
            .arg("caps-log")
            .arg(example(f))
            .arg("--log")
            .arg(&log)
            .output()
            .unwrap();
        assert!(out.status.success(), "append {f} failed");
    }
    // Two chained entries; entry 1's prev_blake3 links to entry 0.
    let content = std::fs::read_to_string(&log).unwrap();
    assert_eq!(content.lines().filter(|l| !l.is_empty()).count(), 2);
    assert!(content.contains("\"prev_blake3\":\"genesis\""), "{content}");

    let verify = garnet()
        .arg("caps-log")
        .arg("--verify")
        .arg(&log)
        .output()
        .unwrap();
    let s = String::from_utf8(verify.stdout).unwrap();
    assert!(verify.status.success(), "intact chain must verify: {s}");
    assert!(s.contains("chain intact"), "{s}");
}

#[test]
fn tampering_breaks_the_chain() {
    let log = fresh_log("tamper");
    for f in ["hello.garnet", "ffi/c_stat.garnet"] {
        garnet()
            .arg("caps-log")
            .arg(example(f))
            .arg("--log")
            .arg(&log)
            .output()
            .unwrap();
    }
    // Flip a byte in entry 0 -> entry 1's prev_blake3 no longer matches.
    let content = std::fs::read_to_string(&log).unwrap();
    let mut lines: Vec<String> = content.lines().map(String::from).collect();
    lines[0] = lines[0].replace("hello", "hellX");
    std::fs::write(&log, lines.join("\n") + "\n").unwrap();

    let verify = garnet()
        .arg("caps-log")
        .arg("--verify")
        .arg(&log)
        .output()
        .unwrap();
    assert!(
        !verify.status.success(),
        "tampered chain must fail verification"
    );
    let err = String::from_utf8(verify.stderr).unwrap();
    assert!(err.contains("CHAIN BROKEN"), "{err}");
}
