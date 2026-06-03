# win-codex S117 Release / Readiness Shell Proof Plan

## Scope

Record a narrow Windows/WSL Studio shell proof for the Release / Readiness panel command wrappers already exposed by the Tauri app. This follows the S117 package-pipeline pattern in `F_Project_Management/GARNET_v0_8_1_PLAN.md` and closes only the reporter-output portion of the status gap named by `scripts/garnet_windows_linux_studio_status.py`. If no live GUI screenshot is captured, the visual screenshot proof remains open as a separate next slice.

## Non-Claims

- Not clean/non-WSL Linux desktop GUI install/launch proof.
- Not Linux seccomp or OS-sandbox enforcement.
- Not signed MSI/AuthentiCode, signed SBOM release artifact, winget, Windows ARM64, production, or v1.0 readiness.
- WSL rows are execution/portability only.

## Steps

1. Write failing tests for a new `--studio-release-readiness-smoke` shell contract, proof recorder, status reporter lane, and readiness lane.
2. Add the Tauri CLI smoke path that invokes existing release/readiness command wrappers and records a manifest-backed payload.
3. Add a recorder/gate script for Windows and WSL proof bundles under `proofs/windows/studio-release-readiness-shell/` and `proofs/linux/execution/studio-release-readiness-shell/`.
4. Wire committed evidence into Windows/Linux Studio status, MIT readiness, docs, changelog, and the v0.8.1 plan without overclaiming.
5. Run focused tests, proof record/gate, readiness no-regression, workspace cargo test, clippy, fmt, diff checks, PR-body validator, then open/merge through the established Navigata1 PR + Chrome Work-profile path.
