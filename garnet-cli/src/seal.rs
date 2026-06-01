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
/// Garnet's self-declared provenance-chain block schema.
pub const PROVENANCE_CHAIN_SCHEMA: &str = "garnet-provenance-chain-v1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SealProvenanceChain {
    pub agent: String,
    pub model: String,
    pub prompt_sha256: String,
    pub artifact_blake3: String,
    pub source_blake3: String,
    pub chain_blake3: String,
}

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
    statement_json_with_authorship(program, build, caps, cosign, None)
}

/// As [`statement_json`], but optionally records an **AI-authorship provenance**
/// declaration (S65) in the predicate. `authorship` is a free-form, self-declared
/// provenance string (e.g. `"ai:claude-opus-4-8"`, `"ai-assisted:..."`,
/// `"human:jon"`) — it is a *declared* fact, not AI-detection. Omitted entirely
/// when `None`, so the default predicate shape is unchanged.
pub fn statement_json_with_authorship(
    program: &str,
    build: &Manifest,
    caps: &CapabilityManifest,
    cosign: bool,
    authorship: Option<&str>,
) -> String {
    statement_json_full(program, build, caps, cosign, authorship, &[])
}

/// As [`statement_json_with_authorship`], but also records a structured
/// **attestation** block (S66) — model / prompt / tool declarations — in the
/// predicate. `attestation` is a list of `(key, value)` pairs (sorted, rendered
/// as a JSON object; omitted when empty). Like authorship, these are
/// *self-declared* (e.g. `model=claude-opus-4-8`, `prompt_sha256=…`,
/// `tool=mcp:filesystem`), not verified.
pub fn statement_json_full(
    program: &str,
    build: &Manifest,
    caps: &CapabilityManifest,
    cosign: bool,
    authorship: Option<&str>,
    attestation: &[(String, String)],
) -> String {
    statement_json_with_chain(program, build, caps, cosign, authorship, attestation, None)
}

/// As [`statement_json_full`], but also records an S97 provenance chain. The
/// chain only verifies deterministic binding of self-declared fields to this
/// seal's current subject/source hashes; it does not independently prove origin.
pub fn statement_json_with_chain(
    program: &str,
    build: &Manifest,
    caps: &CapabilityManifest,
    cosign: bool,
    authorship: Option<&str>,
    attestation: &[(String, String)],
    provenance_chain: Option<&SealProvenanceChain>,
) -> String {
    let cosign_note = if cosign {
        "available — sign with: cosign attest --predicate <file> --type custom"
    } else {
        "not installed — predicate emitted UNSIGNED; install cosign to attest"
    };
    let sbom_note = "garnet-capability-manifest (native SBOM-equivalent; CycloneDX/SPDX via syft/cyclonedx when present)";
    let authorship_field = match authorship {
        Some(a) => format!(",\"authorship\":\"{}\"", json_escape(a)),
        None => String::new(),
    };
    let attestation_field = if attestation.is_empty() {
        String::new()
    } else {
        let mut pairs: Vec<(String, String)> = attestation.to_vec();
        pairs.sort_by(|a, b| a.0.cmp(&b.0));
        let inner: Vec<String> = pairs
            .iter()
            .map(|(k, v)| format!("\"{}\":\"{}\"", json_escape(k), json_escape(v)))
            .collect();
        format!(",\"attestation\":{{{}}}", inner.join(","))
    };
    let provenance_chain_field = match provenance_chain {
        Some(chain) => format!(",\"provenance_chain\":{}", chain.to_json()),
        None => String::new(),
    };
    format!(
        "{{\"_type\":\"{stmt}\",\
         \"subject\":[{{\"name\":\"{name}\",\"digest\":{{\"blake3\":\"{ast}\"}}}}],\
         \"predicateType\":\"{ptype}\",\
         \"predicate\":{{\
         \"source_blake3\":\"{src}\",\
         \"build_manifest\":{build_json},\
         \"capability_manifest\":{caps_json},\
         \"tooling\":{{\"cosign\":\"{cosign_note}\",\"sbom\":\"{sbom}\"}}{authorship_field}{attestation_field}{provenance_chain_field}\
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

/// Build and verify the S97 chain from the existing self-declared attestation
/// pairs. Verification here means: required fields exist, the prompt hash is
/// canonical, and the chain binds to the live seal's source/artifact digests.
pub fn build_provenance_chain(
    build: &Manifest,
    authorship: Option<&str>,
    attestation: &[(String, String)],
) -> Result<SealProvenanceChain, String> {
    let agent = required_unique_attestation(attestation, "agent")?;
    let model = required_unique_attestation(attestation, "model")?;
    let prompt_sha256 = required_unique_attestation(attestation, "prompt_sha256")?;
    validate_prompt_sha256(&prompt_sha256)?;

    let mut pairs = attestation.to_vec();
    pairs.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));
    let mut payload = String::new();
    payload.push_str(PROVENANCE_CHAIN_SCHEMA);
    payload.push('\n');
    payload.push_str("agent=");
    payload.push_str(&agent);
    payload.push('\n');
    payload.push_str("model=");
    payload.push_str(&model);
    payload.push('\n');
    payload.push_str("prompt_sha256=");
    payload.push_str(&prompt_sha256);
    payload.push('\n');
    payload.push_str("source_blake3=");
    payload.push_str(&build.source_hash);
    payload.push('\n');
    payload.push_str("artifact_blake3=");
    payload.push_str(&build.ast_hash);
    payload.push('\n');
    payload.push_str("authorship=");
    payload.push_str(authorship.unwrap_or(""));
    payload.push('\n');
    for (key, value) in &pairs {
        payload.push_str("attest.");
        payload.push_str(key);
        payload.push('=');
        payload.push_str(value);
        payload.push('\n');
    }

    Ok(SealProvenanceChain {
        agent,
        model,
        prompt_sha256,
        artifact_blake3: build.ast_hash.clone(),
        source_blake3: build.source_hash.clone(),
        chain_blake3: blake3::hash(payload.as_bytes()).to_hex().to_string(),
    })
}

impl SealProvenanceChain {
    fn to_json(&self) -> String {
        format!(
            "{{\"schema\":\"{schema}\",\
             \"agent\":\"{agent}\",\
             \"model\":\"{model}\",\
             \"prompt_sha256\":\"{prompt}\",\
             \"artifact_blake3\":\"{artifact}\",\
             \"source_blake3\":\"{source}\",\
             \"chain_blake3\":\"{chain}\",\
             \"binding_verified\":true,\
             \"independent_origin_verified\":false,\
             \"verification_scope\":\"self-declared provenance fields bound to this seal subject/source\",\
             \"limitations\":[\"does not prove the model executed the prompt\",\"does not prove the declared tool list is complete\"]}}",
            schema = json_escape(PROVENANCE_CHAIN_SCHEMA),
            agent = json_escape(&self.agent),
            model = json_escape(&self.model),
            prompt = json_escape(&self.prompt_sha256),
            artifact = json_escape(&self.artifact_blake3),
            source = json_escape(&self.source_blake3),
            chain = json_escape(&self.chain_blake3),
        )
    }
}

fn required_unique_attestation(
    attestation: &[(String, String)],
    key: &str,
) -> Result<String, String> {
    let values: Vec<&String> = attestation
        .iter()
        .filter_map(|(k, v)| (k == key).then_some(v))
        .collect();
    match values.as_slice() {
        [] => Err(format!("missing required attestation key `{key}`")),
        [value] if value.trim().is_empty() => Err(format!("attestation key `{key}` is empty")),
        [value] => Ok((*value).clone()),
        _ => Err(format!("attestation key `{key}` must appear exactly once")),
    }
}

fn validate_prompt_sha256(value: &str) -> Result<(), String> {
    let Some(hex) = value.strip_prefix("sha256:") else {
        return Err("prompt_sha256 must be sha256:<64 lowercase hex>".to_string());
    };
    if hex.len() != 64 || !hex.chars().all(|c| matches!(c, '0'..='9' | 'a'..='f')) {
        return Err("prompt_sha256 must be sha256:<64 lowercase hex>".to_string());
    }
    Ok(())
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

    #[test]
    fn provenance_chain_validates_and_binds_manifest_hashes() {
        let (build, _) = build_for("@caps()\ndef main() { 1 }\n");
        let chain = build_provenance_chain(
            &build,
            Some("ai-assisted:gpt-5"),
            &[
                ("model".to_string(), "gpt-5".to_string()),
                (
                    "prompt_sha256".to_string(),
                    "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
                        .to_string(),
                ),
                ("agent".to_string(), "win-codex".to_string()),
            ],
        )
        .expect("valid chain");
        assert_eq!(chain.agent, "win-codex");
        assert_eq!(chain.model, "gpt-5");
        assert_eq!(chain.artifact_blake3, build.ast_hash);
        assert_eq!(chain.source_blake3, build.source_hash);
        assert_eq!(chain.chain_blake3.len(), 64);
    }
}
