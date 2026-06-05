# Garnet v0.8.1 — academic-review-grade evidence package (S118)

The package for a skeptical academic reader (the CMU / MIT / Rice / UC-Berkeley bar):
**every load-bearing claim is sourced** to a slice, a source file, a test, or a sealed
proof artifact, and the **honest concessions are a first-class section, not a
footnote**. The companion gate (`scripts/garnet_academic_evidence_status.py`) fails if
any cited source does not resolve on disk — so this index cannot rot into aspiration.

v0.8.1 is a **research-grade prototype milestone**. There is **no production / 1.0
claim** and **no tag** in this package — the v0.8.1 cut (S120) is a human decision.

## The one-sentence contribution

A language + toolchain in which a project can **autonomously accept agent-authored
code on enforced evidence and refuse a silent authority expansion** — the refusal a
true gate failure, not a warning — with the runtime kernel actually *enforcing* the
capability and recursion ceilings the acceptance rests on. Each pillar is precedented;
**the integration — a sealed capability-diff gate wired into an autonomous accept/reject
loop that wraps the real subcommands — is the contribution.**

## Sourced claim index

### Pillar 1 — the enforced kernel (capability + recursion, both backends)
- **`@caps` host-authority is trapped at runtime on undeclared use.** Source:
  `garnet-interp-v0.3/src/eval.rs` (interpreter trap), `garnet-vm/src/vm.rs` (VM
  entry-caps), tests `garnet-cli/tests/caps_enforcement.rs`.
- **`@max_depth` recursion is trapped deterministically at the declared ceiling.**
  Source: `garnet-vm/src/vm.rs` (depth guard), tests
  `garnet-cli/tests/bounded_enforcement.rs`.
- **Both ceilings fire identically on interpreter AND VM, on Windows + Mac + Linux.**
  Source: `F_Project_Management/GARNET_CROSS_OS_REPRODUCIBILITY.md` and the consolidated
  matrix `proofs/cross-os/matrix/cross-os-trap-parity-20260604-s109/garnet-cross-os-trap-parity-matrix.json`
  (`cross_os_complete=true`).

### Pillar 2 — the capability surface → diff → acceptance gate
- **The declared capability surface is derived canonically** (top-level, impl-method,
  and nested-module functions). Source: `garnet-check-v0.3/src/capability_surface.rs`.
- **`diff-caps` is a hard acceptance gate** — it computes the authority delta and exits
  non-zero on a widening. Source: `garnet-cli/src/cmd/diff_caps.rs`, tests
  `garnet-cli/tests/pr_review_wedge.rs`.

### Pillar 3 — the autonomous accept/reject loop (the ultrapunch)
- **`agent-loop` wraps the real subcommands**: diff-caps gate → enforced run → seal;
  accept yields 4 trust artifacts, reject yields no seal. Source:
  `garnet-cli/src/cmd/agent_loop.rs`, tests `garnet-cli/tests/ultrapunch_demo.rs`,
  record `C_Language_Specification/GARNET_ULTRAPUNCH.md`.
- **The reviewer-facing positioning** (the #1 claim + ranked runners-up, every line
  resolving to a proof). Source: `F_Project_Management/GARNET_ULTRAPUNCH_DOSSIER.md`.
- **Six demonstrator domains as Mac-native proof artifacts** (accept seals; refusals
  do not; the MCP lens is a static report). Source:
  `F_Project_Management/GARNET_DOMAIN_PROOF_ARTIFACTS.md` and
  `proofs/mac/domains/mac-domain-proofs-20260604-064412/garnet-mac-domain-proofs.json`.

### Pillar 4 — provenance, integrity, and the OS boundary
- **Sealed, tamper-evident provenance** (in-toto predicate bound to source+AST + a
  BLAKE3 transparency log). Source: `garnet-cli/src/seal.rs`,
  `C_Language_Specification/GARNET_ATTESTATION.md`, tests
  `garnet-cli/tests/provenance_seal_chain.rs`.
- **Evidence integrity is machine-verified** (every sealed bundle's manifest re-checks).
  Source: `scripts/garnet_evidence_integrity_status.py`.
- **An OS sandbox policy is APPLIED and traps on a real Linux kernel** (seccomp,
  Linux-only). Source: `tools/seccomp-apply/PROOF_utm_debian12_aarch64.txt`.

### Pillar 5 — adversarial validation
- **The kernel was red-teamed**: a real HIGH hole was found *and fixed*; two LOW
  findings recorded. Source: `C_Language_Specification/GARNET_RED_TEAM.md`, gate
  `scripts/garnet_red_team_status.py`.

## What we refuse to claim (first-class)

- **Accepted on capability + depth evidence ONLY.** `@bounded` (Wasmtime fuel), memory,
  time, and `@mailbox` are **declared-not-enforced**. **macOS sandbox-exec and Windows
  AppContainer are named-deferred** — OS-sandbox *application* is proven on **Linux
  only** (seccomp); cross-OS parity is at the language-runtime trap layer.
- **The agent is simulated/scripted**, not a live LLM (`provider_api_called=false`).
- **Provenance is self-declared** (bound to digests, not independently witnessed); the
  transparency log is a **local stub** (not Rekor — no signed tree head / inclusion
  proof); seals are **unsigned unless cosign is present** (cosign/syft/CycloneDX absent
  → no signing / no SBOM).
- **No standard adopted** (RFC-0001 is intent + reference impl; no OWASP/LF body has
  adopted anything).
- **Two LOW red-team findings remain open** (caps-log tail; seal subject-digest),
  recorded within their honest stub/mitigated scope.
- **No production / 1.0 claim; no tag.** 1.0 stays held (~a year out, validation-gated,
  never slice-count-gated). The v0.8.1 cut (S120) is Jon's.

## How to reproduce the verification

- Workspace: `cargo test --workspace --no-fail-fast` (0 failed) ·
  `cargo clippy --workspace --all-targets -- -D warnings` · `cargo fmt --all -- --check`.
- Gates (each `--gate` → rc 0): `garnet_red_team_status.py`,
  `garnet_ultrapunch_dossier_status.py`, `garnet_domain_proof_artifacts_status.py`,
  `garnet_evidence_integrity_status.py`, and this package's
  `garnet_academic_evidence_status.py`.
- Cross-OS: re-run the trap-parity matrix; Linux seccomp via `tools/seccomp-apply/`.

Every claim above resolves to one of these sources. The package's strength is not that
nothing is deferred — it is that **what is claimed is enforced and verified, what is
deferred is named, and the kernel survived an adversarial pass that found and fixed a
real hole.**
