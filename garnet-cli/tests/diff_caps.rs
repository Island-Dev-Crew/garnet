//! S37 — `garnet diff-caps` integration test (runs the built binary).
//!
//! The contract gate: non-zero exit iff the program GAINED authority between two
//! revisions; zero when caps only shrink or stay the same.

use std::path::{Path, PathBuf};
use std::process::Command;

fn garnet() -> Command {
    Command::new(env!("CARGO_BIN_EXE_garnet"))
}

fn fresh(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("garnet_s37_{tag}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn write(dir: &Path, name: &str, body: &str) -> PathBuf {
    let p = dir.join(name);
    std::fs::write(&p, body).unwrap();
    p
}

#[test]
fn authority_expansion_exits_nonzero() {
    let dir = fresh("expand");
    let old = write(dir.as_path(), "old.garnet", "@caps(fs)\ndef main() { 1 }\n");
    let new = write(
        dir.as_path(),
        "new.garnet",
        "@caps(fs, net)\ndef main() { 1 }\n",
    );
    let out = garnet()
        .arg("diff-caps")
        .arg(&old)
        .arg(&new)
        .output()
        .unwrap();
    assert_eq!(
        out.status.code(),
        Some(1),
        "authority expansion must exit non-zero"
    );
    let s = String::from_utf8(out.stdout).unwrap();
    assert!(s.contains("AUTHORITY EXPANDED"), "{s}");
    assert!(s.contains("caps GAINED") && s.contains("net"), "{s}");
}

#[test]
fn capability_reduction_exits_zero() {
    let dir = fresh("reduce");
    let old = write(
        dir.as_path(),
        "old.garnet",
        "@caps(fs, net)\ndef main() { 1 }\n",
    );
    let new = write(dir.as_path(), "new.garnet", "@caps(fs)\ndef main() { 1 }\n");
    let out = garnet()
        .arg("diff-caps")
        .arg(&old)
        .arg(&new)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "a capability reduction must exit zero"
    );
    let s = String::from_utf8(out.stdout).unwrap();
    assert!(s.contains("no authority expansion"), "{s}");
    assert!(s.contains("caps removed") && s.contains("net"), "{s}");
}

#[test]
fn identical_surface_exits_zero_with_no_changes() {
    let dir = fresh("same");
    let a = write(dir.as_path(), "a.garnet", "@caps(fs)\ndef main() { 1 }\n");
    let b = write(dir.as_path(), "b.garnet", "@caps(fs)\ndef main() { 1 }\n");
    let out = garnet().arg("diff-caps").arg(&a).arg(&b).output().unwrap();
    assert!(out.status.success());
    let s = String::from_utf8(out.stdout).unwrap();
    assert!(s.contains("no capability changes"), "{s}");
}

#[test]
fn verify_caps_baseline_caps_the_fused_band() {
    // Completes the S33 graft: with a baseline, the diff-caps capability signal
    // feeds verify's fused `min`. A current tree that gained `net` vs the
    // baseline caps the fused merge confidence at 2/5.
    let dir = fresh("verify_baseline");
    let old = write(dir.as_path(), "old.garnet", "@caps(fs)\ndef main() { 1 }\n");
    let cur = write(
        dir.as_path(),
        "cur.garnet",
        "@caps(fs, net)\ndef main() { 1 }\n",
    );
    let out = garnet()
        .arg("verify")
        .arg(&cur)
        .arg("--caps-baseline")
        .arg(&old)
        .output()
        .unwrap();
    let s = String::from_utf8(out.stdout).unwrap();
    assert!(
        s.contains("capability signal (diff-caps vs baseline): 2/5"),
        "{s}"
    );
    assert!(s.contains("Merge confidence (fused): 2/5"), "{s}");
}

// ── RB-1 (Directive 15): --machine JSON verdict ─────────────────────────

#[test]
fn machine_expansion_emits_json_verdict_and_same_exit_code() {
    let dir = fresh("machine_expand");
    let old = write(dir.as_path(), "old.garnet", "@caps(fs)\ndef main() { 1 }\n");
    let new = write(
        dir.as_path(),
        "new.garnet",
        "@caps(fs, net)\ndef main() { 1 }\n",
    );
    let out = garnet()
        .arg("diff-caps")
        .arg("--machine")
        .arg(&old)
        .arg(&new)
        .output()
        .unwrap();
    assert_eq!(
        out.status.code(),
        Some(1),
        "--machine must keep the gating exit code"
    );
    let s = String::from_utf8(out.stdout).unwrap();
    assert!(
        s.contains("\"schema\":\"garnet.diff-caps.machine/1\""),
        "{s}"
    );
    assert!(s.contains("\"verdict\":\"authority-expanded\""), "{s}");
    assert!(s.contains("\"capability_band\":\"2/5\""), "{s}");
    assert!(s.contains("\"aggregate_gained\":[\"net\"]"), "{s}");
    assert!(
        !s.contains("AUTHORITY EXPANDED"),
        "machine mode must emit only the JSON payload: {s}"
    );
}

#[test]
fn machine_no_expansion_emits_json_verdict_and_exit_zero() {
    let dir = fresh("machine_same");
    let a = write(dir.as_path(), "a.garnet", "@caps(fs)\ndef main() { 1 }\n");
    let b = write(dir.as_path(), "b.garnet", "@caps(fs)\ndef main() { 1 }\n");
    let out = garnet()
        .arg("diff-caps")
        .arg("--machine")
        .arg(&a)
        .arg(&b)
        .output()
        .unwrap();
    assert!(out.status.success());
    let s = String::from_utf8(out.stdout).unwrap();
    assert!(s.contains("\"verdict\":\"no-authority-expansion\""), "{s}");
    assert!(s.contains("\"capability_band\":\"5/5\""), "{s}");
    assert!(s.contains("\"exit_code\":0"), "{s}");
}

#[test]
fn machine_output_is_deterministic_and_single_line() {
    let dir = fresh("machine_det");
    let old = write(dir.as_path(), "old.garnet", "@caps(fs)\ndef main() { 1 }\n");
    let new = write(
        dir.as_path(),
        "new.garnet",
        "@caps(fs, net, time)\ndef main() { 1 }\n",
    );
    let run = || {
        let out = garnet()
            .arg("diff-caps")
            .arg("--machine")
            .arg(&old)
            .arg(&new)
            .output()
            .unwrap();
        String::from_utf8(out.stdout).unwrap()
    };
    let first = run();
    let second = run();
    assert_eq!(first, second, "machine output must be deterministic");
    assert_eq!(
        first.trim().lines().count(),
        1,
        "machine output is a single JSON line: {first}"
    );
}

#[test]
fn human_output_is_unchanged_without_machine_flag() {
    // The CI gate scripts parse the human text; --machine must be purely
    // additive. Golden assertion: everything below the path-bearing header
    // line is pinned byte-for-byte.
    let dir = fresh("human_stable");
    let old = write(dir.as_path(), "old.garnet", "@caps(fs)\ndef main() { 1 }\n");
    let new = write(
        dir.as_path(),
        "new.garnet",
        "@caps(fs, net)\ndef main() { 1 }\n",
    );
    let out = garnet()
        .arg("diff-caps")
        .arg(&old)
        .arg(&new)
        .output()
        .unwrap();
    let s = String::from_utf8(out.stdout).unwrap();
    let (header, rest) = s.split_once('\n').expect("multi-line output");
    assert!(header.starts_with("garnet diff-caps: "), "{s}");
    assert_eq!(
        rest,
        "  + caps GAINED:  net\n\
         \x20 ~ main gained: net\n\
         \n\
         diff-caps: AUTHORITY EXPANDED — review required (capability band 2/5)\n",
        "human output below the header must be byte-stable"
    );
}

#[test]
fn unknown_diff_caps_flag_is_rejected() {
    // A typo'd flag must not be silently treated as a path (mirrors the
    // adjacent `verify` arm's explicit unknown-flag rejection).
    let out = garnet()
        .arg("diff-caps")
        .arg("--machin")
        .arg("a.garnet")
        .arg("b.garnet")
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    let err = String::from_utf8(out.stderr).unwrap();
    assert!(err.contains("unknown diff-caps flag: --machin"), "{err}");
}
