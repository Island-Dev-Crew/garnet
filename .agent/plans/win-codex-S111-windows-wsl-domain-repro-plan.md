# S111 Windows/WSL Domain Reproduction Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Record a repo-committed Windows + WSL domain proof-matrix reproduction bundle for the Garnet Studio lane, while keeping WSL labeled as execution/portability and not Linux enforcement.

**Architecture:** Reuse the existing `scripts/smoke_garnet_studio_domain_matrix.py` runner and `scripts/garnet_mit_readiness_status.py` verifier. Add a committed-proof lookup path before the existing Desktop/local fallback so readiness can cite evidence from a clean clone. Commit Windows evidence under `proofs/windows/domains/` and WSL evidence under `proofs/linux/execution/domains/`.

**Tech Stack:** Python status/reporting scripts, existing Garnet CLI, WSL Ubuntu portability runner, committed proof manifests, `.dogfood/goal.json`.

---

### Task 1: Committed Domain-Matrix Lookup

**Files:**
- Modify: `scripts/garnet_mit_readiness_status.py`
- Modify: `scripts/test_garnet_mit_readiness_status.py`

- [x] **Step 1: Write the failing test**

Add a test that creates both `proofs/windows/domains/<bundle>/garnet-studio-domain-matrix.json` and `proofs/linux/execution/domains/<bundle>/garnet-studio-domain-matrix.json` in a temporary repo layout and expects `windows_linux_domain_proof_matrix` to be `verified` without setting `GARNET_STUDIO_DOMAIN_MATRIX_ROOT`.

- [x] **Step 2: Run the focused test and verify it fails**

Run: `python scripts/test_garnet_mit_readiness_status.py -k committed_domain`

Expected: FAIL because the reporter only searches the configured Desktop/local evidence root.

- [x] **Step 3: Implement committed lookup**

Add a committed-bundle search that checks `proofs/windows/domains` and `proofs/linux/execution/domains` before the local evidence root. Require both bundles to pass the existing manifest and summary verifier. Preserve the old local fallback and its local-evidence wording.

- [x] **Step 4: Run the focused test and existing readiness tests**

Run: `python scripts/test_garnet_mit_readiness_status.py`

Expected: PASS.

### Task 2: Record Windows + WSL Evidence

**Files:**
- Create: `proofs/windows/domains/<bundle>/...`
- Create: `proofs/linux/execution/domains/<bundle>/...`

- [x] **Step 1: Build the current Garnet CLI**

Run: `cargo build -p garnet-cli`

Expected: PASS and `target/debug/garnet.exe` exists.

- [x] **Step 2: Run the Windows domain matrix**

Run: `python scripts/smoke_garnet_studio_domain_matrix.py --suite all --garnet target/debug/garnet.exe --output-dir proofs/windows/domains/windows-domain-matrix-<stamp> --format json`

Expected: PASS, 20/20 cases and 60/60 commands, with source omitted and provider APIs false.

- [x] **Step 3: Run the WSL portability domain matrix**

Run from WSL against the same checkout, using the Linux-built `target/debug/garnet` if present or building it inside WSL first.

Expected: PASS, 20/20 cases and 60/60 commands. The proof path must be `proofs/linux/execution/domains/wsl-domain-matrix-<stamp>` and the markdown must clearly remain portability evidence, not enforcement.

- [x] **Step 4: Verify manifests**

Run hash verification against both `MANIFEST.sha256` files.

Expected: PASS.

### Task 3: Goal And Docs Catch-Up

**Files:**
- Modify: `.dogfood/goal.json`
- Modify: `F_Project_Management/GARNET_v0_8_1_PLAN.md`
- Modify: `CHANGELOG.md`
- Modify: `F_Project_Management/AGENT_COORDINATION_LEDGER.md`

- [x] **Step 1: Mark S106 merged**

Set S106 to `merged`, `merge_confidence: 5`, and `merged_at: 2026-06-03T13:48:33+00:00` from PR #333.

- [x] **Step 2: Add S111 scoped notes**

Document that this PR records Windows + WSL domain reproduction evidence, not Linux desktop GUI/Tauri launch completion and not seccomp/OS enforcement.

### Task 4: Verification And PR

**Files:**
- Verify all touched files and proof bundles.

- [x] **Step 1: Run focused proof gates**

Run: `python scripts/test_garnet_mit_readiness_status.py`, `python scripts/garnet_mit_readiness_status.py --check-no-regression --format json`, and the two domain-matrix proof commands.

- [x] **Step 2: Run required broad gates**

Run: `python3 scripts/garnet_mit_readiness_status.py`, `cargo test --workspace --no-fail-fast`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all -- --check`, and `git diff --check`.

- [ ] **Step 3: Open PR and dogfood gate**

Open `S111: windows wsl domain proof repro` from `Navigata1:agent-win-codex/s111-windows-wsl-domain-repro` to `Island-Dev-Crew:main`, run the dogfood-readiness grep loop to 5/5, wait for CI green including PR dogfood evidence, then merge through the Chrome Work profile.
