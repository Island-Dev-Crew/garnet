# S117 Linux WSL DEB Install Proof Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a narrow S117 package-pipeline proof that the WSL-built Garnet Studio `.deb` can be installed/extracted in a WSL Linux environment and the installed/extracted binary can run the non-GUI `--studio-smoke` command.

**Architecture:** Follow the existing S117 proof pattern: a script records command logs, a committed proof bundle carries JSON/Markdown plus `MANIFEST.sha256`, focused tests reject overclaims, and readiness/status reporters expose only the evidence that is proven. This remains WSL execution/portability evidence, not Linux desktop GUI launch, clean Linux install, seccomp/OS sandbox, signing/SBOM, winget, ARM64, production, or v1.0 proof.

**Tech Stack:** Python proof recorder/verifier, PowerShell/WSL command execution, Tauri Linux `.deb` artifact, existing Garnet readiness/status reporters.

---

### Task 1: Add the Install/Extract Proof Recorder

**Files:**
- Create: `scripts/smoke_garnet_studio_linux_wsl_deb_install.py`
- Test: `scripts/test_smoke_garnet_studio_linux_wsl_deb_install.py`

- [ ] **Step 1: Write failing tests**

Add tests that build a synthetic committed bundle and assert:
- a valid summary with required command logs verifies;
- any Linux desktop GUI or enforcement overclaim rejects;
- missing extracted binary smoke output rejects;
- manifest paths use POSIX separators for Linux verification.

Run: `python scripts\test_smoke_garnet_studio_linux_wsl_deb_install.py`
Expected before implementation: import/file failure.

- [ ] **Step 2: Implement the recorder**

Create a script with:
- `--record --output-dir <dir>` for WSL recording;
- `--gate --format json` for committed proof verification;
- schema `garnet.studio.linux_wsl_deb_install.v1`;
- required honest-scope lines including "not Linux desktop GUI launch proof", "not clean Linux install proof", and "not Linux seccomp or OS-sandbox enforcement";
- command logs for `dpkg-deb --extract`, extracted binary `--studio-smoke`, and package metadata/contents.

- [ ] **Step 3: Verify focused tests**

Run: `python scripts\test_smoke_garnet_studio_linux_wsl_deb_install.py`
Expected: all tests pass.

### Task 2: Record Committed WSL Evidence

**Files:**
- Create: `proofs/linux/execution/studio-package-install/<bundle>/...`

- [ ] **Step 1: Run the recorder**

Run: `python scripts\smoke_garnet_studio_linux_wsl_deb_install.py --record --output-dir proofs/linux/execution/studio-package-install`
Expected: a new bundle with JSON, Markdown, command logs, extracted package metadata, and `MANIFEST.sha256`.

- [ ] **Step 2: Gate the committed bundle**

Run: `python scripts\smoke_garnet_studio_linux_wsl_deb_install.py --gate --format json`
Expected: `verified: true`, WSL labeled as portability, desktop GUI and clean install unclaimed.

### Task 3: Wire Status and Readiness

**Files:**
- Modify: `scripts/garnet_mit_readiness_status.py`
- Modify: `scripts/garnet_windows_linux_studio_status.py`
- Test: `scripts/test_garnet_mit_readiness_status.py`
- Test: `scripts/test_garnet_windows_linux_studio_status.py`

- [ ] **Step 1: Write/update reporter tests**

Extend tests so the install proof adds a separate committed lane and updates the Windows/Linux distribution detail without changing the meaning of the existing `.deb` package-build lane.

- [ ] **Step 2: Implement reporter wiring**

Import the new verifier, add lane `linux_wsl_studio_deb_install`, and keep `windows_linux_distribution` `active-partial` unless a real Linux desktop GUI proof appears.

- [ ] **Step 3: Verify reporter tests**

Run:
- `python scripts\test_garnet_mit_readiness_status.py`
- `python scripts\test_garnet_windows_linux_studio_status.py`

Expected: both pass.

### Task 4: Update Honest Documentation

**Files:**
- Modify: `CHANGELOG.md`
- Modify: `CURRENT_STATE.md`
- Modify: `F_Project_Management/GARNET_v0_8_1_PLAN.md`
- Modify: `F_Project_Management/AGENT_COORDINATION_LEDGER.md`

- [ ] **Step 1: Document the shipped increment**

Add entries stating the proof covers WSL `.deb` extract/install-style command execution and non-GUI smoke only.

- [ ] **Step 2: Preserve non-claims**

Keep Linux desktop GUI launch, clean Linux install, seccomp/OS-sandbox enforcement, signing/SBOM, winget, Windows ARM64, production, and v1.0 as open.

### Task 5: Final Verification and PR

**Files:**
- Use: `target/pr-body-s117-linux-wsl-deb-install-proof.md`

- [ ] **Step 1: Run local gates**

Run:
- `python scripts\smoke_garnet_studio_linux_wsl_deb_install.py --gate --format json`
- `python scripts\garnet_mit_readiness_status.py --check-no-regression --format json`
- `cargo fmt --all -- --check`
- `cargo test --workspace --no-fail-fast`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `git diff --check`

- [ ] **Step 2: Commit, push, and open PR**

Commit with a focused message, push to `fork`, open a Navigata1 PR to `Island-Dev-Crew:main`, and include Dogfood Readiness headings.

- [ ] **Step 3: Merge only after 5/5 and green CI**

Run PR body validation, wait for CI, append REVIEW, then merge through authenticated Chrome once GitHub shows green checks and a clean merge state.
