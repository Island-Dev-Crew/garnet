# S38 Plan — `seal` (signed reproducible bundle)

Contract: `F_Project_Management/GARNET_v0_8_SLICE_DOGFOOD.md` → S38.
Branch: `codex/s38-seal`.

## Goal ([GRAFT] wrap-don't-rebuild)
A signed, reproducible build attestation. Express it as an **in-toto predicate**,
verify via **cosign**, emit a **CycloneDX/SPDX SBOM**; the capability manifest
(S36) is the **no-SBOM-equivalent extension**. Garnet does NOT implement its own
supply-chain signing — it produces the predicate; `cosign` signs/verifies it.

## Environment reality (honest)
`cosign`, `syft`, `cyclonedx` are **absent** in this environment (verified). So
S38 ships the **real cores** — the in-toto Statement over the deterministic build
manifest (`manifest.rs`) + the capability manifest (S36), with the capability
manifest as the native SBOM-equivalent — and **detects-and-honestly-skips** the
optional cosign signing (the wrapper, not the predicate). The predicate is fully
produced regardless; signing is the user's `cosign attest` step.

## Design
- **`garnet-cli/src/seal.rs`** (new):
  - `cosign_available() -> bool` (detects `cosign version`; never required).
  - `statement_json(program, &Manifest, &CapabilityManifest, cosign: bool)` — a
    deterministic in-toto **Statement v1**: `{_type, subject:[{name,
    digest:{blake3:<ast_hash>}}], predicateType:"…/attestation/seal/v1",
    predicate:{source_blake3, build_manifest:{…}, capability_manifest:{…},
    tooling:{cosign, sbom}}}`. Embeds the build manifest's
    `to_canonical_json()` and the capability manifest's `to_json()` as nested
    JSON (no double-escaping).
- **`garnet seal <file.garnet>`** (`cmd/seal.rs`): edition-aware parse (S32) →
  `Manifest::build` + `capability_surface` (S35/S36) → emit the Statement on
  stdout; a stderr note states whether cosign is present and the exact
  `cosign attest` command to sign the predicate.
- **Readiness lane `seal_attestation`** (committed-truth) + baseline regen.

## Crates touched
- `garnet-cli`: new `seal.rs` + `cmd/seal.rs`, dispatcher + help, new tests.
- Reuses `manifest::Manifest`, `cap_manifest`/`capability_surface`,
  `diagnostics::json_escape`.

## Load-bearing dogfood
- `garnet seal <file>` → valid in-toto Statement JSON (`_type`, `subject` blake3
  digest, `predicateType`, embedded `build_manifest` + `capability_manifest`),
  byte-deterministic for the same source.
- The `tooling.cosign` field + stderr note honestly reflect cosign
  presence/absence; the predicate is emitted either way.

## End-state / gates
- Lane `seal_attestation` added + baseline regen.
- fmt / clippy -D / test --workspace / doc -D / deny / --check-no-regression /
  conformance / python — all green. CHANGELOG + contract S38 state. Dogfood
  bundle → PR → CLI-merge → `s38` advance rides with the S39 PR.

## Honest scope / out of scope (carry the contract's anchor verbatim)
- "seal wraps in-toto/Sigstore/cosign; Garnet does not implement its own signing."
- cosign/syft/cyclonedx are absent in THIS environment → the predicate is emitted
  unsigned and the SBOM is the native capability manifest; the cosign-sign and
  external-SBOM steps run when those tools are present (the wrapper shells out /
  prints the command — it does not auto-sign).
- Per-file seal (per `garnet build`); per-package seal is a follow-up.
