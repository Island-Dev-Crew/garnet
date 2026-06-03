# S117 Linux WSL Xvfb Window Capture Plan

Slot: `win-codex`
Branch: `agent-win-codex/s117-linux-wsl-xvfb-window-capture`
Base: `origin/main` `c6e6e0a` (`docs: record S117 Xvfb merge (#346)`)

## Scope

Record one narrow Linux/Tauri proof increment: WSL starts the extracted Linux Garnet Studio binary under `xvfb-run`, observes the X11 window tree, and captures a virtual-display screenshot artifact.

This is still WSL execution/portability evidence. It is not Linux desktop GUI install/launch proof, not clean Linux install proof, not privileged package install proof, not Linux seccomp or OS-sandbox enforcement, not signing/SBOM, not winget, not Windows ARM64, not production, and not v1.0 readiness.

## Implementation

- Add a focused recorder/gate script for `proofs/linux/execution/studio-xvfb-window-capture/`.
- Add test-first coverage for valid bundles, missing/blank screenshots, missing X11 window metadata, and forbidden overclaims.
- Wire the status/readiness reporters to a new `linux_wsl_studio_xvfb_window_capture` lane without closing the real Linux desktop gate.
- Update section-scoped docs and the coordination ledger.

## Verification

- Focused recorder tests.
- Status/readiness reporter tests.
- Record the proof on this Windows WSL host and verify it from Windows and WSL.
- `cargo fmt --all -- --check`
- `cargo test --workspace --no-fail-fast`
- `cargo clippy --workspace --all-targets -- -D warnings`
- Repo-local dogfood PR-body gate before opening the PR.

## Merge Discipline

Open a fork PR from `Navigata1` and merge only after remote CI is green, including the PR dogfood evidence check. Merge through the authenticated Chrome Work profile. Do not push a tag.
