# AGENTS.md - macOS SwiftUI Studio Shell Contract

## Scope

This directory owns the macOS SwiftUI Studio app. It is a thin native shell
around existing Garnet surfaces (the `garnet` CLI, repo Python reporters, the
agentic dogfood matrix, and the `docs/truth.json` truth surface) — not a fork
of the converter, checker, parser, or the Windows/Linux Tauri shell. It ports
the PR #391 shell *standard*, not the Tauri code
(`F_Project_Management/GARNET_STUDIO_SUITE_HANDOFF_2026_06_12.md` §1).

## Stable Contracts

- **Version truth (row 1):** `StudioVersion.release` in
  `Sources/GarnetStudio/StudioShell.swift` is the single version stamp and
  must equal `[workspace.package].version` in the root `Cargo.toml`. The
  Swift package cannot inherit the workspace version, so
  `scripts/test_garnet_macos_studio_shell.py` IS the sync gate. A workspace
  version bump must bump the stamp in the same PR. Never add a second stamp.
- **Process discipline (row 5):** every spawned command goes through
  `StudioProcessRunner` (thread-drained pipes, per-category timeout from
  settings, best-effort process-tree SIGKILL, `timedOut`/duration surfaced,
  UI payload caps with honest markers). New call sites must use
  `StudioProcessRunner.runBridged`/`run`, never raw
  `Process()` + `waitUntilExit()`. The agentic matrix uses the larger
  `.matrix` timeout category.
- **Settings (row 4):** `StudioSettings` persists
  `{mode, theme, commandTimeoutSecs, matrixTimeoutSecs}` as JSON under the
  per-user Application Support directory. Every write is validated/clamped in
  `normalized()`; a corrupt or missing file never blocks startup — defaults
  win. Mode/theme additionally mirror to `@AppStorage` for the native
  Settings scene.
- **Truth surface (row 6):** release statistics come from `docs/truth.json`
  via `StudioTruthSummary` and degrade to an explicit "unavailable" state.
  The UI must never reintroduce hand-written release numbers.
- **Evidence readers (row 8):** `StudioEvidenceReader` is read-only and
  constrained by `resolveWithinEvidenceRoots` (canonicalize both sides,
  reject anything outside the Studio evidence roots, skip symlinks,
  size/entry caps). Do not widen it into a general filesystem read primitive.
- **Modes (row 3):** simple/power interface modes hide the power-only
  sections (Agentic Tests, Release) without removing them from the compiled
  source — the contract test asserts their copy stays present.
- **Boundaries:** no provider API call path, no network handoff, no
  credential storage, and no network-client usage from this shell. Enabling
  any network-touching surface (including a local-model Co-typist panel)
  requires a Jon-approved amendment to this file plus contract tests and
  security review in the same change.
- Honest claim boundaries stay in the UI copy: research-grade prototype, not
  production/1.0; provider-backed conversion is not active; deferred lanes
  named. Evidence for Mac Studio actions belongs under `~/Desktop/dogfood`.

## Required Checks

Run after changing this app or its contract:

```sh
swift build --package-path apps/garnet-studio-macos
swift test --package-path apps/garnet-studio-macos
python3 scripts/test_garnet_macos_studio_shell.py
python3 scripts/check-agent-contracts.py
```
