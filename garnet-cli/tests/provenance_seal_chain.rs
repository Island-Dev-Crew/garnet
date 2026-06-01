//! S97 - provenance seal chain.
//!
//! `garnet seal --provenance-chain` binds self-declared agent/model/prompt
//! metadata to the current seal subject/source digests. The binding is verified;
//! the origin claim is still self-declared and not independently proven.

use std::path::{Path, PathBuf};
use std::process::Command;

fn garnet() -> Command {
    Command::new(env!("CARGO_BIN_EXE_garnet"))
}

fn hello() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("examples/hello.garnet")
}

fn prompt_hash() -> &'static str {
    "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
}

fn chain_args() -> Vec<String> {
    vec![
        "--provenance-chain".to_string(),
        "--attest".to_string(),
        "agent=win-codex".to_string(),
        "--attest".to_string(),
        "model=gpt-5".to_string(),
        "--attest".to_string(),
        format!("prompt_sha256={}", prompt_hash()),
    ]
}

#[test]
fn provenance_chain_binds_declared_agent_model_prompt_to_current_seal() {
    let out = garnet()
        .arg("seal")
        .arg(hello())
        .args(chain_args())
        .output()
        .unwrap();
    let s = String::from_utf8(out.stdout).unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        s.contains(r#""provenance_chain":{"schema":"garnet-provenance-chain-v1""#),
        "{s}"
    );
    assert!(s.contains(r#""agent":"win-codex""#), "{s}");
    assert!(s.contains(r#""model":"gpt-5""#), "{s}");
    assert!(
        s.contains(&format!(r#""prompt_sha256":"{}""#, prompt_hash())),
        "{s}"
    );
    assert!(s.contains(r#""artifact_blake3":""#), "{s}");
    assert!(s.contains(r#""source_blake3":""#), "{s}");
    assert!(s.contains(r#""chain_blake3":""#), "{s}");
    assert!(s.contains(r#""binding_verified":true"#), "{s}");
    assert!(s.contains(r#""independent_origin_verified":false"#), "{s}");
    assert!(s.contains("self-declared provenance fields bound"), "{s}");
}

#[test]
fn provenance_chain_requires_agent_attestation() {
    let out = garnet()
        .arg("seal")
        .arg(hello())
        .arg("--provenance-chain")
        .args(["--attest", "model=gpt-5"])
        .args(["--attest", &format!("prompt_sha256={}", prompt_hash())])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    let err = String::from_utf8(out.stderr).unwrap();
    assert!(err.contains("provenance-chain"), "{err}");
    assert!(
        err.contains("missing required attestation key `agent`"),
        "{err}"
    );
}

#[test]
fn provenance_chain_rejects_malformed_prompt_hash() {
    let out = garnet()
        .arg("seal")
        .arg(hello())
        .arg("--provenance-chain")
        .args(["--attest", "agent=win-codex"])
        .args(["--attest", "model=gpt-5"])
        .args(["--attest", "prompt_sha256=not-a-sha"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    let err = String::from_utf8(out.stderr).unwrap();
    assert!(err.contains("prompt_sha256"), "{err}");
    assert!(err.contains("sha256:<64 lowercase hex>"), "{err}");
}

#[test]
fn provenance_chain_output_is_deterministic_across_attestation_order() {
    let ordered = garnet()
        .arg("seal")
        .arg(hello())
        .args(chain_args())
        .output()
        .unwrap();
    let reordered = garnet()
        .arg("seal")
        .arg(hello())
        .arg("--provenance-chain")
        .args(["--attest", &format!("prompt_sha256={}", prompt_hash())])
        .args(["--attest", "model=gpt-5"])
        .args(["--attest", "agent=win-codex"])
        .output()
        .unwrap();
    assert!(ordered.status.success());
    assert!(reordered.status.success());
    assert_eq!(ordered.stdout, reordered.stdout);
}
