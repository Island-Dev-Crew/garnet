//! T3: source-bound seals and fail-closed capability acceptance.
use serde_json::Value;
use std::{
    fs,
    path::Path,
    process::{Command, Output},
};
fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_garnet"))
        .args(args)
        .output()
        .unwrap()
}
fn path(p: &Path) -> &str {
    p.to_str().unwrap()
}
fn source(dir: &Path, name: &str, src: &str) -> std::path::PathBuf {
    let p = dir.join(name);
    fs::write(&p, src).unwrap();
    p
}
const CLEAN: &str = "@caps()\ndef main() { 1 }\n";
const WIDE: &str = "@caps(fs)\ndef main() { 1 }\n";
fn seal(p: &Path) -> Value {
    let out = run(&["seal", path(p)]);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    serde_json::from_slice(&out.stdout).unwrap()
}
#[test]
fn subject_binds_capability_edits_and_normalized_source() {
    let dir = tempfile::tempdir().unwrap();
    let p = source(dir.path(), "app.garnet", CLEAN);
    let a = seal(&p);
    fs::write(&p, WIDE).unwrap();
    let b = seal(&p);
    assert_ne!(a["subject"][0]["digest"], b["subject"][0]["digest"]);
    assert_eq!(
        a["subject"][0]["digest"]["blake3"],
        a["predicate"]["source_blake3"]
    );
    assert_eq!(
        a["predicateType"],
        "https://garnet-lang.org/attestation/seal/v2"
    );
    fs::write(&p, WIDE.replace('\n', "\r\n")).unwrap();
    assert_eq!(b, seal(&p));
}
#[test]
fn checker_invalid_source_never_emits_or_overwrites_a_seal() {
    let dir = tempfile::tempdir().unwrap();
    let p = source(dir.path(), "app.garnet", "def main() { 1 }\n");
    let dest = dir.path().join("seal.json");
    let out = run(&["seal", path(&p), "--out", path(&dest)]);
    assert!(!out.status.success());
    assert!(out.stdout.is_empty());
    assert!(!dest.exists());
    fs::write(&dest, "retain me").unwrap();
    let out = run(&["seal", path(&p), "--out", path(&dest)]);
    assert!(!out.status.success());
    assert_eq!(fs::read_to_string(&dest).unwrap(), "retain me");
}
#[test]
fn seals_round_trip_and_refuse_tampered_source_or_bindings() {
    let dir = tempfile::tempdir().unwrap();
    let p = source(dir.path(), "app.garnet", CLEAN);
    let dest = dir.path().join("seal.json");
    assert!(run(&["seal", path(&p), "--out", path(&dest)])
        .status
        .success());
    assert!(run(&["verify", path(&p), path(&dest)]).status.success());
    assert!(!run(&["verify", path(&p), path(&dest), "--signature"])
        .status
        .success());
    let original = fs::read_to_string(&dest).unwrap();
    for (from, to) in [
        ("seal/v2", "seal/v1"),
        ("seal/v2", "seal/v99"),
        ("\"aggregate\":[]", "\"aggregate\":[\"fs\"]"),
    ] {
        fs::write(&dest, original.replace(from, to)).unwrap();
        assert!(
            !run(&["verify", path(&p), path(&dest)]).status.success(),
            "{from}"
        );
    }
    fs::write(&dest, &original).unwrap();
    fs::write(&p, WIDE).unwrap();
    assert!(!run(&["verify", path(&p), path(&dest)]).status.success());
}
#[test]
fn baseline_widening_errors_and_invalid_inputs_fail_both_verify_routes() {
    let dir = tempfile::tempdir().unwrap();
    let old = source(dir.path(), "old.garnet", CLEAN);
    let p = source(dir.path(), "app.garnet", WIDE);
    let dest = dir.path().join("seal.json");
    assert!(run(&["seal", path(&p), "--out", path(&dest)])
        .status
        .success());
    let malformed = source(dir.path(), "malformed.garnet", "def (");
    let invalid = source(dir.path(), "invalid.garnet", "def main() { 1 }\n");
    let missing = dir.path().join("missing");
    let empty = dir.path().join("empty");
    fs::create_dir(&empty).unwrap();
    for baseline in [&old, &malformed, &invalid, &missing, &empty] {
        for artifact in [None, Some(&dest)] {
            let mut args = vec!["verify", path(&p)];
            if let Some(a) = artifact {
                args.push(path(a));
            }
            args.extend(["--caps-baseline", path(baseline)]);
            let out = run(&args);
            assert!(
                !out.status.success(),
                "accepted {} with artifact={artifact:?}: {}",
                baseline.display(),
                String::from_utf8_lossy(&out.stdout)
            );
            assert!(!String::from_utf8_lossy(&out.stdout).contains("gate: PASS"));
        }
    }
    assert!(run(&["verify", path(&p), "--caps-baseline", path(&p)])
        .status
        .success());
    assert!(
        run(&["verify", path(&p), path(&dest), "--caps-baseline", path(&p)])
            .status
            .success()
    );
}

#[test]
fn advisory_diagnostics_do_not_block_check_seal_or_verify() {
    let dir = tempfile::tempdir().unwrap();
    let p = source(
        dir.path(),
        "app.garnet",
        "@caps()\ndef main() { try { 1 } rescue e { 0 } }\n",
    );
    let checked = run(&["check", path(&p)]);
    assert!(checked.status.success());
    assert!(String::from_utf8_lossy(&checked.stdout).contains("advisory"));
    let dest = dir.path().join("seal.json");
    let sealed = run(&["seal", path(&p), "--out", path(&dest)]);
    assert!(sealed.status.success());
    assert!(sealed.stdout.is_empty());
    assert!(String::from_utf8_lossy(&sealed.stderr).contains("advisory"));
    assert!(run(&["verify", path(&p), path(&dest)]).status.success());
}
#[test]
fn verify_rechecks_even_a_structurally_matching_seal() {
    let dir = tempfile::tempdir().unwrap();
    let p = source(dir.path(), "app.garnet", "def main() { 1 }\n");
    let src = fs::read_to_string(&p).unwrap();
    let module = garnet_parser::parse_source(&src).unwrap();
    let build = garnet_cli::manifest::Manifest::build(&src, &module);
    let caps = garnet_cli::cap_manifest::CapabilityManifest::from_surface(
        garnet_check::capability_surface(&module),
    );
    let dest = dir.path().join("seal.json");
    fs::write(
        &dest,
        garnet_cli::seal::statement_json("app", &build, &caps),
    )
    .unwrap();
    assert!(!run(&["verify", path(&p), path(&dest)]).status.success());
}
#[test]
fn binding_tamper_duplicate_unknown_and_noncanonical_fields_fail() {
    let dir = tempfile::tempdir().unwrap();
    let p = source(dir.path(), "app.garnet", CLEAN);
    let dest = dir.path().join("seal.json");
    let sealed = run(&["seal", path(&p)]);
    let original = String::from_utf8(sealed.stdout).unwrap();
    let parsed: Value = serde_json::from_str(&original).unwrap();
    let digest = parsed["subject"][0]["digest"]["blake3"].as_str().unwrap();
    let mutations = [
        original.replacen(digest, &"0".repeat(64), 1),
        original.replace("garnet-source-lf-blake3-v1", "garnet-source-lf-blake3-v2"),
        original.replace("\"subject\":", "\"extra\":true,\"subject\":"),
        original.replace("\"_type\":", "\"_type\":\"bogus\",\"_type\":"),
        original.replace(
            "\"source_hash\":",
            "\"source_hash\":\"bogus\",\"source_hash\":",
        ),
        serde_json::to_string_pretty(&parsed).unwrap(),
    ];
    for mutated in mutations {
        assert_ne!(mutated, original);
        fs::write(&dest, mutated).unwrap();
        assert!(!run(&["verify", path(&p), path(&dest)]).status.success());
    }
}
#[test]
fn provenance_chain_is_recomputed_and_cannot_claim_independent_origin() {
    let dir = tempfile::tempdir().unwrap();
    let p = source(dir.path(), "app.garnet", CLEAN);
    let dest = dir.path().join("seal.json");
    let out = run(&[
        "seal",
        path(&p),
        "--provenance-chain",
        "--attest",
        "agent=test",
        "--attest",
        "model=test",
        "--attest",
        "prompt_sha256=sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
    ]);
    assert!(out.status.success());
    let original = String::from_utf8(out.stdout).unwrap();
    fs::write(&dest, &original).unwrap();
    assert!(run(&["verify", path(&p), path(&dest)]).status.success());
    let parsed: Value = serde_json::from_str(&original).unwrap();
    let chain = parsed["predicate"]["provenance_chain"]["chain_blake3"]
        .as_str()
        .unwrap();
    for mutated in [
        original.replace(chain, &"0".repeat(64)),
        original.replace(
            "\"independent_origin_verified\":false",
            "\"independent_origin_verified\":true",
        ),
        original.replacen("\"agent\":\"test\"", "\"agent\":\"changed\"", 1),
    ] {
        fs::write(&dest, mutated).unwrap();
        assert!(!run(&["verify", path(&p), path(&dest)]).status.success());
    }
}
#[test]
fn legacy_manifests_still_verify_and_baseline_is_enforced() {
    let dir = tempfile::tempdir().unwrap();
    let p = source(dir.path(), "app.garnet", WIDE);
    let old = source(dir.path(), "old.garnet", CLEAN);
    assert!(run(&["build", "--deterministic", path(&p)])
        .status
        .success());
    let manifest = p.with_extension("garnet.manifest.json");
    assert!(run(&["verify", path(&p), path(&manifest)]).status.success());
    assert!(!run(&[
        "verify",
        path(&p),
        path(&manifest),
        "--caps-baseline",
        path(&old)
    ])
    .status
    .success());
    assert!(run(&[
        "verify",
        path(&p),
        path(&manifest),
        "--caps-baseline",
        path(&p)
    ])
    .status
    .success());
}
#[test]
fn omitted_source_roots_and_duplicate_attestation_fail_closed() {
    let dir = tempfile::tempdir().unwrap();
    let old = dir.path().join("old");
    fs::create_dir(&old).unwrap();
    source(&old, "app.garnet", CLEAN);
    fs::create_dir(old.join("target")).unwrap();
    source(&old.join("target"), "hidden.garnet", WIDE);
    let p = source(dir.path(), "app.garnet", CLEAN);
    let out = run(&["verify", path(&p), "--caps-baseline", path(&old)]);
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("incomplete"));
    let out = run(&["seal", path(&p), "--attest", "tool=a", "--attest", "tool=b"]);
    assert!(!out.status.success());
    assert!(out.stdout.is_empty());
}

#[test]
fn gate_diagnostics_distinguish_current_source_and_absent_baseline() {
    let dir = tempfile::tempdir().unwrap();
    let current = dir.path().join("current.garnet");
    let baseline = dir.path().join("baseline.garnet");
    std::fs::write(&current, "@caps()\ndef main() { time::now_ms() }\n").unwrap();
    std::fs::write(&baseline, "@caps()\ndef main() { 0 }\n").unwrap();
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_garnet"))
        .arg("verify")
        .arg(&current)
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(!String::from_utf8_lossy(&output.stdout).contains("widening"));
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_garnet"))
        .arg("verify")
        .arg(&current)
        .arg("--caps-baseline")
        .arg(&baseline)
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("current source failed checker"));
}
