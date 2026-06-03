# S117 Linux WSLg System Install/Launch Proof Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:test-driven-development for proof-code changes and superpowers:dogfood-readiness before merge. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a narrow S117 increment proving this Windows box can install the WSL-built Garnet Studio `.deb` as a privileged WSL system package and launch the installed Linux Studio binary through WSLg, without claiming clean Linux, native desktop distro, seccomp, OS-sandbox, signing, winget, Windows ARM64, production, or v1.0 readiness.

**Architecture:** Reuse the existing WSL `.deb` package proof as the input source, then add a new manifest-backed recorder/gate under `scripts/` and `proofs/linux/execution/`. The recorder verifies WSLg display variables, records pre-install package state, installs the `.deb` with `dpkg -i` as root/sudo, verifies `dpkg -s`, runs `/usr/bin/garnet-studio --studio-smoke`, starts `/usr/bin/garnet-studio` in the WSLg session long enough to observe a process/window signal, and removes the package after recording so the machine is not left mutated.

**Tech Stack:** Python standard library, WSL, dpkg, Tauri Linux `.deb`, WSLg display environment, existing Garnet readiness/status reporters.

---

## File Structure

- Create `scripts/smoke_garnet_studio_linux_wslg_install_launch.py`: recorder/gate for privileged WSL package install plus WSLg launch evidence.
- Create `scripts/test_smoke_garnet_studio_linux_wslg_install_launch.py`: focused tests for manifest verification, claim boundaries, command requirements, and markdown.
- Modify `scripts/garnet_windows_linux_studio_status.py`: import the new recorder and expose the new package gate only when verified.
- Modify `scripts/test_garnet_windows_linux_studio_status.py`: focused status regression for the new lane.
- Modify `scripts/garnet_mit_readiness_status.py`: add a committed lane and carefully move Windows/Linux distribution from 79.0 to the next active-partial increment only when the proof verifies.
- Modify `scripts/test_garnet_mit_readiness_status.py`: readiness regression for the new lane.
- Modify `CHANGELOG.md`, `CURRENT_STATE.md`, `README.md`, `docs/index.html`, `docs/status.html`, and `F_Project_Management/GARNET_v0_8_1_PLAN.md`: section-scoped calibrated-honesty updates.
- Create committed proof bundle under `proofs/linux/execution/studio-wslg-system-install/`.

## Tasks

- [ ] Write focused failing tests for the new recorder/gate.
- [ ] Implement the minimal recorder/gate and prove the focused tests pass.
- [ ] Add status/readiness integrations with failing tests first, then implementation.
- [ ] Run the recorder on this Windows/WSL box and commit the manifest-backed proof bundle.
- [ ] Update docs with exact evidence and open boundaries.
- [ ] Run local verification: focused tests, recorder `--gate`, WSL `--gate`, readiness no-regression, workspace tests, clippy, fmt, diff-check, PR-body dogfood validation.
- [ ] Open a fork PR from Navigata1, wait for remote CI, run dogfood-readiness self-audit to 5/5, merge through Chrome Work profile.

## Completion Boundary

This slice can claim WSL/WSLg privileged package install plus installed binary launch evidence only if the proof bundle verifies from a clean clone. It must not claim clean Linux install, non-WSL Linux desktop install, Linux seccomp, OS-sandbox enforcement, signed/SBOM release artifacts, winget, Windows ARM64, production readiness, or v1.0 readiness.
