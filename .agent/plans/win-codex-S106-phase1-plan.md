# S106 Windows Cross-OS Enforcement Proof Phase 1 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Record a Windows proof and WSL portability replay that the Stage V `@max_depth` and `@caps` VM/interpreter traps still fire from the current v0.8.1 runway.

**Architecture:** Add one proof recorder/status script that runs the existing S101 gate plus focused Stage V Rust tests, writes evidence under `proofs/windows/enforcement/` and `proofs/linux/execution/`, and validates those committed records. The WSL record is explicitly execution/portability evidence, not Linux seccomp, Wasmtime, or OS-sandbox enforcement.

**Tech Stack:** Python 3 status/recorder script, existing Rust integration tests in `garnet-cli`, existing readiness reporter, existing dogfood PR body gate.

---

### Task 1: Test The Proof Contract First

**Files:**
- Create: `scripts/test_garnet_windows_cross_os_enforcement_proof.py`

- [ ] **Step 1: Write tests for committed proof parsing and honesty labels**

Add tests that create temporary `proofs/windows/enforcement/windows-enforcement-proof.json` and `proofs/linux/execution/wsl-execution-portability-proof.json` files, then assert:

- Windows proof is classified as `enforcement-proof`.
- WSL proof is classified as `execution-portability`.
- WSL markdown includes `execution/portability, not enforcement`.
- The gate fails if the Windows proof is missing or if the WSL row claims enforcement.

- [ ] **Step 2: Run the new test and verify RED**

Run: `python scripts/test_garnet_windows_cross_os_enforcement_proof.py`

Expected: FAIL because `scripts/garnet_windows_cross_os_enforcement_proof.py` does not exist yet.

### Task 2: Implement The Proof Recorder/Status Gate

**Files:**
- Create: `scripts/garnet_windows_cross_os_enforcement_proof.py`

- [ ] **Step 1: Implement status data classes and committed proof validation**

The script reads committed proof JSON records from:

- `proofs/windows/enforcement/windows-enforcement-proof.json`
- `proofs/linux/execution/wsl-execution-portability-proof.json`

It checks schema, platform, tier, command exit codes, required trap labels, and WSL honesty text.

- [ ] **Step 2: Implement `--record-proof` for Windows**

Run these commands and store logs + JSON:

- `python scripts/garnet_vm_interp_enforcement_parity_status.py --gate --format json`
- `cargo test -p garnet-cli --test bounded_enforcement -- --nocapture`
- `cargo test -p garnet-cli --test caps_enforcement -- --nocapture`

Record trap labels: `@max_depth`, `@caps(env)`, `@caps(proc)`, `@caps(fs)`, `@caps(net)`, `S92 program-entry @caps(proc)`.

- [ ] **Step 3: Implement `--record-proof` for WSL**

Run the same command set through `wsl.exe`, write to `proofs/linux/execution/`, and force the tier/scope text to `execution/portability, not enforcement`.

- [ ] **Step 4: Add a manifest**

Write `MANIFEST.sha256` for each proof directory, excluding the manifest file itself.

### Task 3: Record The Phase 1 Evidence

**Files:**
- Create: `proofs/windows/enforcement/windows-enforcement-proof.json`
- Create: `proofs/windows/enforcement/windows-enforcement-proof.md`
- Create: `proofs/windows/enforcement/*.log`
- Create: `proofs/windows/enforcement/MANIFEST.sha256`
- Create: `proofs/linux/execution/wsl-execution-portability-proof.json`
- Create: `proofs/linux/execution/wsl-execution-portability-proof.md`
- Create: `proofs/linux/execution/*.log`
- Create: `proofs/linux/execution/MANIFEST.sha256`

- [ ] **Step 1: Run Windows proof recorder**

Run: `python scripts/garnet_windows_cross_os_enforcement_proof.py --record-proof --platform windows --out proofs/windows/enforcement`

Expected: exit 0 and Windows proof JSON says all required commands passed.

- [ ] **Step 2: Run WSL portability recorder**

Run: `python scripts/garnet_windows_cross_os_enforcement_proof.py --record-proof --platform wsl --out proofs/linux/execution`

Expected: exit 0 and WSL proof JSON labels the result `execution/portability, not enforcement`.

- [ ] **Step 3: Run the S106 gate**

Run: `python scripts/garnet_windows_cross_os_enforcement_proof.py --gate --format json`

Expected: exit 0.

### Task 4: Wire Readiness And Project Records

**Files:**
- Modify: `scripts/garnet_mit_readiness_status.py`
- Modify: `scripts/test_garnet_mit_readiness_status.py`
- Modify: `F_Project_Management/GARNET_v0_8_1_PLAN.md`
- Modify: `F_Project_Management/AGENT_COORDINATION_LEDGER.md`
- Modify: `CHANGELOG.md`

- [ ] **Step 1: Add a committed-truth readiness lane**

Add `windows_cross_os_enforcement_proof` at 100% only when the S106 gate passes. Its evidence must name Windows enforcement proof and WSL portability replay separately.

- [ ] **Step 2: Add readiness tests**

Assert the new lane exists, is verified, and includes the phrase `WSL execution/portability, not enforcement`.

- [ ] **Step 3: Update plan, ledger, and changelog**

Record that S106 Phase 1 re-proves Stage V traps on Windows and WSL portability only. Do not claim Linux OS enforcement.

### Task 5: Verify, Package, PR, And Merge

- [ ] **Step 1: Focused verification**

Run:

- `python scripts/test_garnet_windows_cross_os_enforcement_proof.py`
- `python scripts/garnet_windows_cross_os_enforcement_proof.py --gate --format json`
- `python scripts/test_garnet_mit_readiness_status.py`
- `python scripts/garnet_mit_readiness_status.py --check-no-regression --format json`

- [ ] **Step 2: Full verification**

Run serially:

- `cargo test --workspace --no-fail-fast`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo fmt --all --check`
- `git diff --check`

- [ ] **Step 3: Dogfood readiness**

Run the dogfood grep loop against the changed-file inventory, overclaim scan, committed proof directories, status gates, and PR body checker until 5/5.

- [ ] **Step 4: Open PR and merge**

Open `S106: windows cross-OS enforcement proof`, wait for green CI including the dogfood check, merge through authenticated Chrome, update the lane ledger only after GitHub records the merge, then stop and report. Do not start Phase 2.
