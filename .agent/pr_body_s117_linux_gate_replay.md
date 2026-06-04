## Summary

Adds the S117 Linux/Tauri gate replay consolidation. The new repo-owned replay script runs the current committed Linux/Tauri WSL/WSLg gates from one manifest-backed bundle, verifies each child gate's JSON proof, and wires the consolidated evidence into the Windows/Linux status and MIT readiness reporters.

## Dogfood Readiness

### Current truth

- [x] This PR records a narrow S117 Windows-lane consolidation: replay of the already-committed Linux/Tauri WSL and WSLg execution/portability gates.
- [x] The replay covers 8 child gates: WSL `.deb` package proof, WSL `.deb` extract/install proof, WSL `.rpm` proof, WSL Xvfb runtime-start proof, WSL Xvfb window-capture proof, WSLg system install/launch proof, Studio domain-shell proof, and Studio Release / Readiness shell proof.
- [x] The MIT readiness reporter now includes a scored `Linux/Tauri gate replay proof (S117 consolidation)` lane and reports 92.3% overall committed readiness.
- [x] The Windows/Linux local distribution lane now reports 83.0% when the consolidated replay and all child evidence verify together.
- [x] WSL/WSLg rows remain labeled execution/portability only, not Linux seccomp, OS-sandbox enforcement, clean/non-WSL Linux desktop, signed release, production, or v1.0 proof.

### Local verification

- [x] `python scripts\test_smoke_garnet_studio_linux_gate_replay.py` -> 6 passed.
- [x] `python scripts\smoke_garnet_studio_linux_gate_replay.py --record --format json` -> recorded replay bundle at `proofs/linux/execution/studio-gate-replay/linux-gate-replay-20260603-235754/`.
- [x] `python scripts\smoke_garnet_studio_linux_gate_replay.py --gate --format json` -> verified the committed replay bundle and all 8 child gates.
- [x] `python scripts\test_garnet_windows_linux_studio_status.py` -> 17 passed.
- [x] `python scripts\test_garnet_mit_readiness_status.py` -> 36 passed.
- [x] `python scripts\garnet_mit_readiness_status.py --check-no-regression` -> passed, overall 92.3%, Windows/Linux distribution 83.0%.
- [x] `cargo fmt --all -- --check` -> passed.
- [x] `cargo test --workspace --no-fail-fast` -> passed with 0 failed.
- [x] `cargo clippy --workspace --all-targets -- -D warnings` -> passed.
- [x] `git diff --check` -> passed; Git emitted Windows line-ending normalization warnings only.

### Remote verification

- [x] Pending GitHub Actions on this PR; merge is held until CI is green, including the dogfood PR-body check.

### Merge confidence

- [x] Local self-audit confidence: 5/5 after focused replay tests, replay gate, readiness no-regression, full workspace tests, clippy, fmt, diff-check, and explicit overclaim boundaries.
- [x] Fused/merge confidence remains held on remote CI until the PR dogfood check and required matrix rows are green.

### Goal progress

- [x] This moves the committed MIT/productization headline from 92.1% to 92.3% and the local Windows/Linux distribution lane from 82.0% to 83.0%.
- [x] Remaining Windows/Linux items stay visible: clean/non-WSL Linux desktop GUI proof, Windows ARM64, live Release / Readiness GUI screenshot, signed MSI/AuthentiCode, winget, website/status copy sync, and signing credentials.

### Evidence bundle

- [x] Consolidated Linux/Tauri replay bundle: `proofs/linux/execution/studio-gate-replay/linux-gate-replay-20260603-235754/`.
- [x] Bundle contains `garnet-studio-linux-gate-replay.json`, Markdown summary, `MANIFEST.sha256`, and per-child-gate stdout/stderr captures under `commands/`.
- [x] Evidence fields preserve the boundary: `linux_gate_replay_proven=true`, `child_gate_count=8`, `all_child_gates_verified=true`, `platform_tier="WSL/WSLg execution-portability only, not enforcement"`, `linux_enforcement_proven=false`, `non_wsl_linux_desktop_proven=false`, `signed_release_claimed=false`, and `v1_0_claimed=false`.

### Deferred / out of scope

- [x] Clean/non-WSL Linux desktop GUI install/launch remains open.
- [x] Linux seccomp or OS-sandbox enforcement remains real-kernel work, not WSL/WSLg proof.
- [x] Live Release / Readiness GUI screenshot remains separate proof.
- [x] Signed MSI/AuthentiCode, SBOM/signing release proof, winget, Windows ARM64, production readiness, and v1.0 readiness remain unclaimed.
