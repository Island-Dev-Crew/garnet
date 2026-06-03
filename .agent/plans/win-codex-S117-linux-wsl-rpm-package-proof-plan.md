# S117 Linux WSL RPM Package Proof Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add and land a narrow S117 package-pipeline increment proving, from the Windows lane, that WSL can build a Tauri Linux RPM package, inspect/extract it, and run the extracted Garnet Studio binary's non-GUI `--studio-smoke`.

**Architecture:** Mirror the existing `.deb` recorders: a manifest-backed Python recorder/gate owns the proof schema, focused tests validate the bundle and reject overclaims, readiness/status reporters expose the new lane, and committed proof artifacts live under `proofs/linux/execution/studio-rpm-package/`. The proof remains WSL execution/portability evidence only.

**Tech Stack:** Python stdlib test/record scripts, WSL shell commands, Tauri CLI, RPM tooling (`rpmbuild`, `rpm`, `rpm2cpio`, `cpio`), existing Garnet readiness reporters.

---

### Task 1: Add RPM Proof Recorder With Tests

**Files:**
- Create: `scripts/smoke_garnet_studio_linux_wsl_rpm.py`
- Create: `scripts/test_smoke_garnet_studio_linux_wsl_rpm.py`

- [ ] **Step 1: Write failing verifier tests**

Create tests that build an in-temp fake RPM proof bundle and assert:
- a valid bundle verifies and `read_committed_evidence()` reports verified;
- missing extracted smoke output fails verification;
- forbidden overclaims fail verification;
- generated manifests use POSIX paths only.

- [ ] **Step 2: Run focused test and watch it fail**

Run: `python scripts\test_smoke_garnet_studio_linux_wsl_rpm.py`

Expected: failure because the script does not exist yet.

- [ ] **Step 3: Implement the recorder/gate**

Implement a script that:
- records WSL `uname`;
- ensures RPM tooling is present, installing `rpm`/`cpio` through WSL package manager only when absent and available;
- runs `npm install --include=optional`;
- runs `npm run build`;
- runs `npm exec -- tauri build --bundles rpm`;
- inspects the resulting RPM with `rpm -qip` and `rpm -qlp`;
- extracts the RPM payload with `rpm2cpio ... | cpio -idmv`;
- runs the extracted `usr/bin/garnet-studio --studio-smoke`;
- writes JSON, Markdown, command logs, extracted smoke JSON, and `MANIFEST.sha256`.

The schema must explicitly say WSL is not enforcement, not clean Linux install, not privileged system package install, not Linux desktop GUI launch, not signed/SBOM, not production, and not v1.0.

- [ ] **Step 4: Run focused tests and gate**

Run:
- `python scripts\test_smoke_garnet_studio_linux_wsl_rpm.py`
- `python scripts\smoke_garnet_studio_linux_wsl_rpm.py --gate --format json`

The gate will fail until committed evidence is recorded; the focused unit tests should pass.

### Task 2: Record Real WSL RPM Evidence

**Files:**
- Add generated bundle under `proofs/linux/execution/studio-rpm-package/`

- [ ] **Step 1: Run recorder**

Run: `python scripts\smoke_garnet_studio_linux_wsl_rpm.py --record`

Expected: a passing manifest-backed bundle with RPM metadata, contents, extraction logs, and extracted-binary smoke.

- [ ] **Step 2: Re-run gate on Windows and WSL**

Run:
- `python scripts\smoke_garnet_studio_linux_wsl_rpm.py --gate --format json`
- `wsl.exe -e sh -lc "cd '<wsl repo path>' && python3 scripts/smoke_garnet_studio_linux_wsl_rpm.py --gate --format json"`
- `wsl.exe -e sh -lc "cd '<wsl repo path>' && python3 scripts/test_smoke_garnet_studio_linux_wsl_rpm.py"`

Expected: all pass.

### Task 3: Wire Status, Readiness, And Docs

**Files:**
- Modify: `scripts/garnet_windows_linux_studio_status.py`
- Modify: `scripts/garnet_mit_readiness_status.py`
- Modify: `scripts/test_garnet_windows_linux_studio_status.py`
- Modify: `scripts/test_garnet_mit_readiness_status.py`
- Modify: `CURRENT_STATE.md`
- Modify: `CHANGELOG.md`
- Modify: `F_Project_Management/GARNET_v0_8_1_PLAN.md`

- [ ] **Step 1: Add failing status/readiness tests**

Add tests that expect the RPM lane to verify without changing the honest deferred boundaries.

- [ ] **Step 2: Run focused tests and watch them fail**

Run:
- `python scripts\test_garnet_windows_linux_studio_status.py`
- `python scripts\test_garnet_mit_readiness_status.py`

- [ ] **Step 3: Wire the RPM lane**

Import the RPM proof module, expose it as a dedicated readiness lane, update Windows/Linux distribution copy, and keep full S117 pending.

- [ ] **Step 4: Update docs**

Add concise S117 RPM increment notes to `CHANGELOG.md`, `CURRENT_STATE.md`, and `GARNET_v0_8_1_PLAN.md`.

### Task 4: Validate, PR, And Merge

**Files:**
- Modify: `F_Project_Management/AGENT_COORDINATION_LEDGER.md`

- [ ] **Step 1: Run local validation**

Run focused tests, readiness no-regression, `cargo fmt --all -- --check`, `cargo test --workspace --no-fail-fast`, `cargo clippy --workspace --all-targets -- -D warnings`, diff checks, overclaim grep, and PR body validator.

- [ ] **Step 2: Commit and open PR**

Push to `Navigata1`, open a fork PR to `Island-Dev-Crew:main`, and include dogfood headings.

- [ ] **Step 3: Wait for CI and merge with Chrome**

Wait for remote CI including dogfood evidence. Merge through the authenticated Chrome Work profile only after the Grep Loop reaches 5/5.
