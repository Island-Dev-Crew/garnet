# S110 Windows/WSL Ultrapunch Reproduction Plan

> **For agentic workers:** keep this slice focused on reproduction evidence. Do not change the S102-S104 acceptance gates, the dogfood skill, diff-caps thresholds, or CI merge policy.

**Goal:** Independently reproduce the S104 ultrapunch on this Windows machine and in WSL, commit the resulting proof bundles, and teach the readiness reporter to recognize those committed bundles without claiming WSL/Linux enforcement.

**Source of truth:** `F_Project_Management/GARNET_v0_8_1_PLAN.md` Stage R, S110. S111 domain reproduction is already merged on `origin/main`; this slice backfills S110 before S112 can consolidate.

**Honest scope:** Windows proves local S104 accept/reject reproduction. WSL proves execution/portability reproduction only. This does not prove Linux seccomp, OS-sandbox enforcement, Wasmtime fuel, Linux desktop/Tauri GUI launch, production readiness, or v1.0 readiness.

---

### Task 1: Add a Committed Ultrapunch Repro Recorder

**Files:**
- Add: `scripts/smoke_garnet_ultrapunch_repro.py`
- Add: `scripts/test_smoke_garnet_ultrapunch_repro.py`

- [x] **Step 1: Write focused tests**

Use a fake Garnet command to prove the recorder preserves accept artifacts, refuses widening and over-depth proposals, verifies the transparency log, writes a JSON/Markdown summary, and emits a manifest.

- [x] **Step 2: Implement recorder**

Drive the existing `garnet agent-loop` over `baseline.garnet`, `accept_proposal.garnet`, `reject_widen.garnet`, and `reject_overdepth.garnet`; capture stdout/stderr and retain the four accept trust artifacts.

- [x] **Step 3: Run focused tests**

Run: `python scripts/test_smoke_garnet_ultrapunch_repro.py`

Expected: PASS.

### Task 2: Add Readiness Detection for Committed Windows + WSL Bundles

**Files:**
- Modify: `scripts/garnet_mit_readiness_status.py`
- Modify: `scripts/test_garnet_mit_readiness_status.py`

- [x] **Step 1: Write reporter test**

Create temporary Windows and WSL repro bundles and assert the new lane is `verified` only when both pass manifest-backed summary verification.

- [x] **Step 2: Implement reporter lane**

Search `proofs/windows/ultrapunch/**/garnet-ultrapunch-repro.json` and `proofs/linux/repro/**/garnet-ultrapunch-repro.json`; require both; label WSL as portability-repro only.

- [x] **Step 3: Run reporter tests**

Run: `python scripts/test_garnet_mit_readiness_status.py -k ultrapunch`

Expected: PASS.

### Task 3: Record Windows + WSL Evidence

**Files:**
- Add: `proofs/windows/ultrapunch/<bundle>/...`
- Add: `proofs/linux/repro/<bundle>/...`

- [x] **Step 1: Build Garnet CLI**

Run: `cargo build -p garnet-cli --bin garnet`

- [x] **Step 2: Record Windows reproduction**

Run the new recorder with `--platform windows` and `--garnet target/debug/garnet.exe`.

- [x] **Step 3: Record WSL portability reproduction**

Run the new recorder inside WSL with the WSL-built `target/wsl/debug/garnet`, and write under `proofs/linux/repro/`.

- [x] **Step 4: Verify manifests**

Verify both `MANIFEST.sha256` files and inspect summaries for overclaim wording.

### Task 4: Docs, Goal Ledger, and Verification

**Files:**
- Modify: `.dogfood/goal.json`
- Modify: `CHANGELOG.md`
- Modify: `F_Project_Management/GARNET_v0_8_1_PLAN.md`
- Modify: `F_Project_Management/AGENT_COORDINATION_LEDGER.md`

- [x] **Step 1: Catch up previous merge state**

Record S111 as merged from PR #334 and append the S110 STARTED entry.

- [x] **Step 2: Add S110 notes**

State exactly what is reproduced and what remains pending.

- [x] **Step 3: Run local gates**

Run focused tests, readiness no-regression, `cargo test --workspace --no-fail-fast`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all -- --check`, and `git diff --check`.

Fresh results on 2026-06-03:
- `python scripts/test_smoke_garnet_ultrapunch_repro.py` PASS.
- `python scripts/test_garnet_mit_readiness_status.py` PASS.
- `python3 scripts/garnet_mit_readiness_status.py --check-no-regression --format json` PASS, readiness 90.8%.
- `cargo test --workspace --no-fail-fast` PASS.
- `cargo clippy --workspace --all-targets -- -D warnings` PASS.
- `cargo fmt --all -- --check` PASS.
- `git diff --check` PASS.
- Overclaim grep returned no matching added lines.

- [ ] **Step 4: PR, dogfood, and merge**

Open `S110: windows wsl ultrapunch repro` from `Navigata1:agent-win-codex/s110-windows-wsl-ultrapunch-repro`, run the dogfood-readiness Grep Loop to 5/5, wait for CI green including PR dogfood evidence, and merge through the Chrome Work profile.
