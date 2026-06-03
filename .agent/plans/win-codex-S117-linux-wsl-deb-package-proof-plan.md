# S117 Linux WSL DEB Package Proof Plan

Goal: add a reproducible S117 package-pipeline increment that records the Garnet Studio Tauri Linux `.deb` build and non-GUI `--studio-smoke` command proof from WSL, without claiming Linux desktop GUI install/launch, seccomp, OS-sandbox enforcement, signing, winget, Windows ARM64, production, or v1.0 readiness.

Architecture:
- Add a Windows-hosted Python recorder under `scripts/` that invokes WSL, refreshes the project-local Tauri optional dependency binding when needed, runs the frontend build, builds the Linux `.deb` bundle via Tauri, runs the Linux binary's non-GUI `--studio-smoke`, inspects the `.deb` with `dpkg-deb`, writes command logs, hashes the `.deb` and binary, and emits a manifest-backed proof bundle under `proofs/linux/execution/studio-package/`.
- Add unit tests around the verifier and claim guardrails using synthetic bundles first, then record the real WSL proof bundle.
- Wire the committed bundle into readiness/status docs as a package-pipeline increment only.

Files:
- Create `scripts/smoke_garnet_studio_linux_wsl_deb.py`
- Create `scripts/test_smoke_garnet_studio_linux_wsl_deb.py`
- Add committed proof bundle under `proofs/linux/execution/studio-package/`
- Update `scripts/garnet_mit_readiness_status.py` and `scripts/test_garnet_mit_readiness_status.py`
- Update `scripts/garnet_windows_linux_studio_status.py` and `scripts/test_garnet_windows_linux_studio_status.py`
- Update `CHANGELOG.md`, `CURRENT_STATE.md`, `F_Project_Management/GARNET_v0_8_1_PLAN.md`, and this coordination ledger section

TDD Steps:
1. Write verifier tests for a valid synthetic WSL `.deb` proof bundle and forbidden overclaims.
2. Run the focused tests and confirm they fail because the script does not exist.
3. Implement the recorder/verifier with `--record`, `--gate`, and `--format json|md`.
4. Run focused tests to green.
5. Record the real WSL proof bundle and run `--gate`.
6. Wire readiness/status reporters with focused tests.
7. Run local gates: focused tests, readiness no-regression, workspace test, clippy, fmt/diff check, PR body validator.
8. Open PR from `Navigata1`, wait for remote CI, merge via authenticated Chrome Work profile only after green.

Honesty Rules:
- WSL evidence is Linux package build and command-smoke evidence, not Linux desktop GUI runtime proof.
- The `.deb` is built and inspected, not installed into a clean Linux desktop session.
- No Linux seccomp or OS-sandbox enforcement claim comes from WSL.
- No signed MSI, winget, Windows ARM64, production, or v1.0 claim.
