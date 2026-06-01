# S97 Provenance Seal Chain Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans or inline TDD. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Extend `garnet seal` with a deterministic provenance-chain block that binds self-declared agent/model/prompt metadata to the current sealed artifact and validates that binding without claiming independent origin proof.

**Architecture:** Reuse the existing S38/S65/S66 seal path in `garnet-cli`. A new `--provenance-chain` flag will require conventional attestation keys (`agent`, `model`, `prompt_sha256`) and emit a `provenance_chain` object whose artifact/source digests come from the live `Manifest`. The chain verification is deliberately narrow: it verifies field presence, prompt-hash shape, deterministic canonicalization, and binding to the current seal subject/source; it does not prove the model ran the prompt or enumerate every tool invoked.

**Tech Stack:** Rust (`garnet-cli`), deterministic hand-rolled JSON, existing BLAKE3 manifest hashes, Python status gate, existing MIT readiness reporter.

---

### Task 1: Failing Seal Tests

**Files:**
- Create: `garnet-cli/tests/provenance_seal_chain.rs`

- [ ] **Step 1: Add CLI tests before implementation**

Create tests that prove:
- `garnet seal examples/hello.garnet --provenance-chain --attest agent=win-codex --attest model=gpt-5 --attest prompt_sha256=sha256:<64 hex>` emits `predicate.provenance_chain`.
- The block contains `schema`, `agent`, `model`, `prompt_sha256`, `artifact_blake3`, `source_blake3`, `chain_blake3`, `binding_verified:true`, and `independent_origin_verified:false`.
- Omitting `agent` fails with a usage error.
- A malformed `prompt_sha256` fails with a usage error.
- Reordering the `--attest` flags produces byte-identical stdout.

- [ ] **Step 2: Verify RED**

Run:

```powershell
cargo test -p garnet-cli --test provenance_seal_chain --no-fail-fast
```

Expected before implementation: compile or assertion failure because `--provenance-chain` is unknown.

### Task 2: Seal Chain Implementation

**Files:**
- Modify: `garnet-cli/src/seal.rs`
- Modify: `garnet-cli/src/cmd/seal.rs`
- Modify: `C_Language_Specification/GARNET_ATTESTATION.md`

- [ ] **Step 1: Add a seal provenance-chain type**

Add a small `SealProvenanceChain` struct plus `build_provenance_chain(...) -> Result<SealProvenanceChain, String>` in `seal.rs`. It should:
- collect `agent`, `model`, and `prompt_sha256` from the existing attestation pairs,
- accept `prompt_sha256` only as `sha256:` plus 64 ASCII hex characters,
- bind `artifact_blake3` to `build.ast_hash` and `source_blake3` to `build.source_hash`,
- compute `chain_blake3` from a deterministic ASCII payload over agent, model, prompt hash, source hash, artifact hash, authorship, and sorted attestation pairs,
- render a deterministic JSON object with honest fields:
  `binding_verified:true`, `independent_origin_verified:false`, and `verification_scope:"self-declared provenance fields bound to this seal subject/source"`.

- [ ] **Step 2: Wire the CLI flag**

In `cmd/seal.rs`, parse `--provenance-chain`; when present, build the chain after the manifest/capability manifest exist. On validation failure, print `garnet seal: provenance-chain: <reason>` and return usage exit `2`.

- [ ] **Step 3: Include the chain in the predicate**

Extend `statement_json_full(...)` with an optional chain argument, while keeping the old `statement_json(...)` and `statement_json_with_authorship(...)` wrappers stable.

- [ ] **Step 4: Document the honest scope**

Add a S97 section to `GARNET_ATTESTATION.md` explaining that chain verification means deterministic binding of declared fields to the sealed artifact, not independent model-run or tool-history verification.

- [ ] **Step 5: Verify GREEN**

Run:

```powershell
cargo test -p garnet-cli --test provenance_seal_chain --no-fail-fast
cargo test -p garnet-cli --test seal_attestation --no-fail-fast
cargo test -p garnet-cli --test seal_attestation_block --no-fail-fast
```

Expected: all pass.

### Task 3: Status Gate and Readiness Lane

**Files:**
- Create: `scripts/garnet_provenance_seal_chain_status.py`
- Create: `scripts/test_garnet_provenance_seal_chain_status.py`
- Modify: `scripts/garnet_mit_readiness_status.py`

- [ ] **Step 1: Add status tests first**

Tests should assert the status JSON reports:
- `schema == "garnet.provenance_seal_chain/v1"`,
- the Rust test file exists,
- the CLI flag is wired,
- the readiness lane import is wired,
- `ok` is true when all source markers are present.

- [ ] **Step 2: Verify RED**

Run:

```powershell
python scripts/test_garnet_provenance_seal_chain_status.py
```

Expected before implementation: failure because the script does not exist.

- [ ] **Step 3: Implement the status script**

The script should read source markers by default and run the focused Rust test when `--gate` is present.

- [ ] **Step 4: Add committed-truth MIT readiness lane**

Add `provenance_seal_chain` after S96. Score 100% only when the status script reports `ok`. Evidence must say self-declared provenance is bound and validated; deferred scope must name missing independent model-run proof and incomplete tool-history proof.

- [ ] **Step 5: Verify GREEN**

Run:

```powershell
python scripts/test_garnet_provenance_seal_chain_status.py
python scripts/garnet_provenance_seal_chain_status.py --gate --format json
python scripts/garnet_mit_readiness_status.py --check-no-regression --format json
```

Expected: all pass; readiness increases from 89.8%.

### Task 4: Lane Docs, Ledger, Evidence

**Files:**
- Modify: `CHANGELOG.md`
- Modify: `F_Project_Management/GARNET_v0_8_1_PLAN.md`
- Modify: `.dogfood/goal.json`
- Modify: `F_Project_Management/AGENT_COORDINATION_LEDGER.md`

- [ ] **Step 1: Update docs with calibrated honesty**

Add a S97 changelog entry and mark S96 merged / S97 active in the plan and goal ledger. Do not claim cryptographic origin proof or complete tool-history capture.

- [ ] **Step 2: Run full local gates**

Run:

```powershell
cargo fmt --all --check
cargo test -p garnet-cli --test provenance_seal_chain --no-fail-fast
python scripts/test_garnet_provenance_seal_chain_status.py
python scripts/garnet_provenance_seal_chain_status.py --gate --format json
python scripts/garnet_mit_readiness_status.py --check-no-regression --format json
cargo test --workspace --no-fail-fast
cargo clippy --workspace --all-targets -- -D warnings
git diff --check
```

- [ ] **Step 3: Build Desktop dogfood bundle**

Copy command outputs into `C:\Users\IslandDevCrew\Desktop\dogfood\garnet-s97-provenance-seal-chain-<timestamp>` and write `MANIFEST.sha256`.

- [ ] **Step 4: Open PR and review**

Open PR title `S97: add provenance seal chain`, run the dogfood PR body checker, complete the grep loop to 5/5 confidence, wait for green CI, merge through Chrome, then start S98 from fresh `origin/main`.
