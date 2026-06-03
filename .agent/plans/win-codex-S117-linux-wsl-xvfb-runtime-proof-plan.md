# win-codex S117 Linux WSL Xvfb Runtime Proof Plan

Goal: add a narrow S117 package-pipeline increment proving that the Linux Tauri
Studio binary can start under WSL with a virtual X display and remain alive until
the harness timeout, while preserving that this is not Linux desktop GUI
install/launch, clean Linux install, privileged package install, Linux seccomp,
OS-sandbox enforcement, signing/SBOM, winget, Windows ARM64, production, or v1.0
readiness.

## Scope

- Add a manifest-backed recorder/gate for WSL `xvfb-run` runtime-start smoke.
- Prefer the already committed/extracted RPM or DEB binary when available, and
  rebuild/extract only when the expected binary is missing.
- Record WSL display/tooling, command stdout/stderr, exit code, timeout duration,
  binary path, and SHA-256.
- Treat timeout exit `124` as the expected pass signal: the GUI process stayed
  alive until the harness stopped it.
- Treat immediate exit/crash as a failure and record logs without overclaiming.

## TDD

1. Add focused unit tests for the evidence classifier and committed-bundle reader.
2. Watch them fail before implementation.
3. Implement the recorder/gate.
4. Run focused tests and the live proof gate on Windows and WSL.

## Docs/Readiness

- Add a separate `linux_wsl_studio_xvfb_runtime` readiness lane.
- Update Windows/Linux Studio status to show `wsl-xvfb-runtime-start-verified`
  only when the proof verifies.
- Update `CURRENT_STATE.md`, `CHANGELOG.md`, and the v0.8.1 plan with calibrated
  scope.
- Keep S117 full package-pipeline readiness pending until real Linux desktop
  install/launch and signed/SBOM/public-release gates are proven.
