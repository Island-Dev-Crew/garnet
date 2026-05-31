//! S38 — `seal`: a reproducible build attestation.
//!
//! **Wrap, don't rebuild** (contract graft): `garnet seal` emits an **in-toto
//! Statement** (predicate) over the deterministic build manifest (`manifest.rs`)
//! and the capability manifest (S36) — the artifact a supply-chain tool
//! (`cosign` / Sigstore) signs and verifies. Garnet does **not** implement its
//! own supply-chain signing; it produces the predicate, and `cosign attest`
//! signs it. The capability manifest is Garnet's native SBOM-equivalent
//! extension.
//!
//! `cosign` / `syft` / `cyclonedx` may be absent; `seal` detects-and-honestly-
//! skips the optional signing/SBOM-tool step. The predicate is produced either
//! way.

use crate::cap_manifest::CapabilityManifest;
use crate::diagnostics::json_escape;
use crate::manifest::Manifest;
use std::process::Command;

/// in-toto Statement type (v1).
pub const STATEMENT_TYPE: &str = "https://in-toto.io/Statement/v1";
/// Garnet's seal predicate type.
pub const PREDICATE_TYPE: &str = "https://garnet-lang.org/attestation/seal/v1";

/// Whether `cosign` is available on `PATH` — the supply-chain signer this seal
/// predicate is meant to be attested with. Detected, never required.
pub fn cosign_available() -> bool {
    Command::new("cosign")
        .arg("version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Build the deterministic in-toto Statement JSON for a sealed program. The
/// build + capability manifests are embedded as nested JSON in the predicate.
pub fn statement_json(
    program: &str,
    build: &Manifest,
    caps: &CapabilityManifest,
    cosign: bool,
) -> String {
    let cosign_note = if cosign {
        "available — sign with: cosign attest --predicate <file> --type custom"
    } else {
        "not installed — predicate emitted UNSIGNED; install cosign to attest"
    };
    let sbom_note = "garnet-capability-manifest (native SBOM-equivalent; CycloneDX/SPDX via syft/cyclonedx when present)";
    format!(
        "{{\"_type\":\"{stmt}\",\
         \"subject\":[{{\"name\":\"{name}\",\"digest\":{{\"blake3\":\"{ast}\"}}}}],\
         \"predicateType\":\"{ptype}\",\
         \"predicate\":{{\
         \"source_blake3\":\"{src}\",\
         \"build_manifest\":{build_json},\
         \"capability_manifest\":{caps_json},\
         \"tooling\":{{\"cosign\":\"{cosign_note}\",\"sbom\":\"{sbom}\"}}\
         }}}}",
        stmt = json_escape(STATEMENT_TYPE),
        name = json_escape(program),
        ast = json_escape(&build.ast_hash),
        ptype = json_escape(PREDICATE_TYPE),
        src = json_escape(&build.source_hash),
        build_json = build.to_canonical_json(),
        caps_json = caps.to_json(),
        cosign_note = json_escape(cosign_note),
        sbom = json_escape(sbom_note),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use garnet_check::capability_surface;

    fn build_for(src: &str) -> (Manifest, CapabilityManifest) {
        let module = garnet_parser::parse_source(src).expect("parses");
        (
            Manifest::build(src, &module),
            CapabilityManifest::from_surface(capability_surface(&module)),
        )
    }

    #[test]
    fn statement_is_in_toto_shaped() {
        let (build, caps) = build_for("@caps(fs)\ndef main() { 1 }\n");
        let json = statement_json("demo", &build, &caps, false);
        assert!(
            json.contains(r#""_type":"https://in-toto.io/Statement/v1""#),
            "{json}"
        );
        assert!(json.contains(r#""predicateType":"https://garnet-lang.org/attestation/seal/v1""#));
        assert!(json.contains(r#""name":"demo""#));
        assert!(json.contains(r#""digest":{"blake3":""#), "{json}");
        // The two embedded manifests are present as nested JSON.
        assert!(json.contains(r#""build_manifest":{"#), "{json}");
        assert!(
            json.contains(r#""capability_manifest":{"schema":"garnet-capability-manifest-v1""#),
            "{json}"
        );
        assert!(json.contains(r#""aggregate":["fs"]"#), "{json}");
    }

    #[test]
    fn cosign_flag_is_reflected_in_tooling() {
        let (build, caps) = build_for("@caps()\ndef main() { 1 }\n");
        assert!(statement_json("d", &build, &caps, false).contains("not installed"));
        assert!(statement_json("d", &build, &caps, true).contains("cosign attest"));
    }

    #[test]
    fn statement_is_deterministic() {
        let (build, caps) = build_for("@caps(net)\ndef main() { 1 }\n");
        assert_eq!(
            statement_json("d", &build, &caps, false),
            statement_json("d", &build, &caps, false)
        );
    }
}
