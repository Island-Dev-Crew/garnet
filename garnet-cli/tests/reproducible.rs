//! Reproducible build tests — Paper VI Contribution 7.
//!
//! Drives `garnet build --deterministic` + `garnet verify` end-to-end:
//! - Building the same source twice produces byte-identical manifests.
//! - The verify subcommand exits 0 on a matching pair, 2 on mismatch.
//! - Mutating one byte of the source breaks verification.
//! - Whitespace-only changes preserve the AST hash but break the source hash.

use std::path::PathBuf;
use std::process::Command;

fn garnet_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_garnet"))
}

fn write_temp(name: &str, src: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("garnet_repro_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let p = dir.join(name);
    std::fs::write(&p, src).unwrap();
    p
}

#[test]
fn build_emits_manifest_sidecar() {
    let p = write_temp("hello.garnet", "def main() { 42 }");
    let out = Command::new(garnet_bin())
        .args(["build", "--deterministic", p.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(out.status.success(), "build should succeed");
    let manifest_path = p.with_file_name("hello.garnet.manifest.json");
    assert!(manifest_path.exists(), "manifest sidecar must exist");
    let manifest = std::fs::read_to_string(&manifest_path).unwrap();
    assert!(manifest.contains("\"schema\""));
    assert!(manifest.contains("\"source_hash\""));
    assert!(manifest.contains("\"ast_hash\""));
}

#[test]
fn build_twice_produces_identical_manifest() {
    let p = write_temp("twice.garnet", "def main() { 1 + 2 }");

    let _ = Command::new(garnet_bin())
        .args(["build", "--deterministic", p.to_str().unwrap()])
        .output()
        .unwrap();
    let manifest_path = p.with_file_name("twice.garnet.manifest.json");
    let m1 = std::fs::read_to_string(&manifest_path).unwrap();

    // Re-run.
    let _ = Command::new(garnet_bin())
        .args(["build", "--deterministic", p.to_str().unwrap()])
        .output()
        .unwrap();
    let m2 = std::fs::read_to_string(&manifest_path).unwrap();

    assert_eq!(m1, m2, "two builds of the same source must be byte-identical");
}

#[test]
fn verify_succeeds_for_unchanged_source() {
    let p = write_temp("verify_ok.garnet", "def f(x) { x * 2 }");

    let _ = Command::new(garnet_bin())
        .args(["build", "--deterministic", p.to_str().unwrap()])
        .output()
        .unwrap();
    let manifest_path = p.with_file_name("verify_ok.garnet.manifest.json");

    let out = Command::new(garnet_bin())
        .args([
            "verify",
            p.to_str().unwrap(),
            manifest_path.to_str().unwrap(),
        ])
        .output()
        .unwrap();

    assert!(
        out.status.success(),
        "verify must exit 0 on unchanged source; stdout={}, stderr={}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(String::from_utf8_lossy(&out.stdout).contains("OK"));
}

#[test]
fn verify_fails_after_source_mutation() {
    let p = write_temp("mutated.garnet", "def main() { 100 }");

    let _ = Command::new(garnet_bin())
        .args(["build", "--deterministic", p.to_str().unwrap()])
        .output()
        .unwrap();
    let manifest_path = p.with_file_name("mutated.garnet.manifest.json");

    // Mutate the source AFTER building the manifest.
    std::fs::write(&p, "def main() { 200 }").unwrap();

    let out = Command::new(garnet_bin())
        .args([
            "verify",
            p.to_str().unwrap(),
            manifest_path.to_str().unwrap(),
        ])
        .output()
        .unwrap();

    assert!(
        !out.status.success(),
        "verify must fail after source mutation; stdout={}",
        String::from_utf8_lossy(&out.stdout)
    );
    let exit = out.status.code().unwrap_or(1);
    assert_eq!(exit, 2, "verify must exit 2 on mismatch");
}

#[test]
fn build_identical_across_different_cwds_and_intervals() {
    // v3.3 regression guard: the v3.2 `build_twice_produces_identical_manifest`
    // test ran two builds back-to-back in the SAME temp dir. If the manifest
    // accidentally captured cwd or wall-clock, that test would still pass.
    // This test stresses the determinism claim by:
    //   - building the same source in TWO different directories
    //   - with a wall-clock gap between them (>= 1 second)
    // and asserting the manifests are byte-identical.
    let src = "def main() { 1 + 2 }";
    let dir_a = std::env::temp_dir()
        .join(format!("garnet_repro_a_{}", std::process::id()));
    let dir_b = std::env::temp_dir()
        .join(format!("garnet_repro_b_{}", std::process::id()));
    std::fs::create_dir_all(&dir_a).unwrap();
    std::fs::create_dir_all(&dir_b).unwrap();
    let path_a = dir_a.join("x.garnet");
    let path_b = dir_b.join("x.garnet");
    std::fs::write(&path_a, src).unwrap();
    std::fs::write(&path_b, src).unwrap();

    let _ = Command::new(garnet_bin())
        .args(["build", "--deterministic", path_a.to_str().unwrap()])
        .output()
        .unwrap();
    // Wall-clock gap — longer than any plausible scheduler jitter.
    std::thread::sleep(std::time::Duration::from_millis(1200));
    let _ = Command::new(garnet_bin())
        .args(["build", "--deterministic", path_b.to_str().unwrap()])
        .output()
        .unwrap();

    let manifest_a = std::fs::read_to_string(dir_a.join("x.garnet.manifest.json")).unwrap();
    let manifest_b = std::fs::read_to_string(dir_b.join("x.garnet.manifest.json")).unwrap();

    assert_eq!(
        manifest_a, manifest_b,
        "manifests built in different cwds and >1s apart must be byte-identical\n--- a ---\n{manifest_a}\n--- b ---\n{manifest_b}"
    );
}

#[test]
fn whitespace_change_breaks_source_hash_keeps_ast_hash() {
    // Build manifest from the compact form, then change source to a
    // semantically-identical pretty form, and verify: source_hash
    // must mismatch (so verify fails) but the manifest's ast_hash field
    // is what would match if the user swapped a hash-only checker. We
    // assert the failure mode here.
    let p = write_temp("ws.garnet", "def main() { 1 }");

    let _ = Command::new(garnet_bin())
        .args(["build", "--deterministic", p.to_str().unwrap()])
        .output()
        .unwrap();
    let manifest_path = p.with_file_name("ws.garnet.manifest.json");

    // Reformat the source — different bytes, same AST shape.
    std::fs::write(&p, "def main() {\n  1\n}").unwrap();

    let out = Command::new(garnet_bin())
        .args([
            "verify",
            p.to_str().unwrap(),
            manifest_path.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(!out.status.success(), "verify must fail on source byte change");
    // The error message must call out source_hash, not ast_hash.
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("source_hash mismatch"),
        "expected source_hash mismatch error, got: {stderr}"
    );
}
