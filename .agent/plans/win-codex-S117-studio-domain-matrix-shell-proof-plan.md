# win-codex S117 Studio Domain Matrix Shell Proof Plan

Branch: `agent-win-codex/s117-studio-domain-matrix-shell-proof`

## Current Truth

- `origin/main` is at `c2eacca` after PR #350 recorded the PR #349 WSLg system-install merge state.
- Baseline gates before edits:
  - `python scripts/garnet_mit_readiness_status.py --check-no-regression --format json` -> `91.9%`, `windows_linux_distribution=80.0% active-partial`
  - `cargo test --workspace --no-fail-fast` -> pass
  - `cargo clippy --workspace --all-targets -- -D warnings` -> pass
- The Linux package lane is at `wslg-system-install-launch-verified`.
- Clean/non-WSL Linux desktop GUI install/launch remains open and requires a real Linux desktop or VM/container target. WSL/WSLg stays portability evidence only.

## Scope

Record the next locally actionable Windows/Linux Studio proof named by `scripts/garnet_windows_linux_studio_status.py`: Domain Proof Matrix screenshots/output from the Windows shell and WSL/Linux shell.

This slice should:

1. Add a manifest-backed smoke recorder for Studio-domain proof presentation/output evidence.
2. Capture Windows domain-matrix output evidence using the existing repo-owned matrix.
3. Capture WSL domain-matrix output evidence as Linux execution/portability only.
4. Capture a Studio shell command-output substitute tied to the domain-matrix surface when the Tauri command path can launch.
5. Update readiness/status docs only for the evidence actually recorded.

## Non-Claims

- No clean/non-WSL Linux desktop GUI install/launch proof.
- No Linux seccomp or OS-sandbox enforcement proof.
- No signed/SBOM release artifact, winget, Windows ARM64, production, or v1.0 claim.
- No provider-backed conversion.

## Verification

- Focused recorder tests.
- Recorder `--record` and `--gate`.
- Windows/Linux Studio status tests.
- MIT readiness reporter tests and no-regression.
- `cargo fmt --all -- --check`.
- `cargo test --workspace --no-fail-fast`.
- `cargo clippy --workspace --all-targets -- -D warnings`.
- Repo-local dogfood PR-body gate.
- Remote CI green before Chrome Work-profile merge.
