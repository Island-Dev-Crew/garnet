## Summary

Adds the S117 Windows/WSL Studio Release / Readiness shell proof increment. The Studio binary now exposes `--studio-release-readiness-smoke`, which drives the same Tauri command wrappers behind the Release / Readiness status reporters and records manifest-backed Windows + WSL evidence.

## Dogfood Readiness

### Current truth

- [x] This PR records a narrow S117 Windows-lane increment: Release / Readiness shell reporter output from the Tauri Studio binary on Windows plus WSL execution/portability.
- [x] The proof verifies four repo-native status reporter command wrappers: Windows/Linux Studio status, MIT objective pulse, converter status, and Windows clean-VM installer status.
- [x] The proof narrows the Release / Readiness status gap to the live GUI screenshot only; it does not claim a clean/non-WSL Linux desktop GUI run.
- [x] WSL is labeled execution/portability only, not Linux seccomp or OS-sandbox enforcement.
- [x] The standalone `python -m dogfood_readiness` module is not installed in this Windows environment (`No module named dogfood_readiness`), so the acceptance record uses the repo-local dogfood PR-body checker plus the explicit self-audit below rather than claiming that module ran.

### Local verification

- [x] `python scripts\test_smoke_garnet_studio_release_readiness_shell.py` -> 4 passed.
- [x] `python scripts\test_garnet_windows_linux_studio_shell.py` -> 5 passed.
- [x] `python scripts\test_garnet_windows_linux_studio_status.py` -> 16 passed.
- [x] `python scripts\test_garnet_mit_readiness_status.py` -> 35 passed.
- [x] `python scripts\smoke_garnet_studio_release_readiness_shell.py --record --format json` -> recorded Windows and WSL proof bundles.
- [x] `python scripts\smoke_garnet_studio_release_readiness_shell.py --gate --format json` -> verified Windows bundle `proofs/windows/studio-release-readiness-shell/win-20260603-230129/garnet-studio-release-readiness-shell-proof.json` and WSL bundle `proofs/linux/execution/studio-release-readiness-shell/wsl-20260603-230237/garnet-studio-release-readiness-shell-proof.json`.
- [x] `python scripts\garnet_mit_readiness_status.py --check-no-regression` -> passed, overall 92.1%, Windows/Linux distribution 82.0%.
- [x] `cargo fmt --all -- --check` -> passed.
- [x] `cargo test --workspace --no-fail-fast` -> passed with 0 failed.
- [x] `cargo clippy --workspace --all-targets -- -D warnings` -> passed.
- [x] `git diff --check` -> passed.
- [x] `python scripts\test_check_dogfood_pr_body.py` -> 10 passed.

### Remote verification

- [x] Pending GitHub Actions on this PR; merge is held until CI is green, including the dogfood PR-body check.

### Merge confidence

- [x] Local self-audit confidence: 5/5 after focused tests, proof gate, readiness no-regression, workspace tests, clippy, fmt, diff-check, and overclaim scan.
- [x] Fused/merge confidence remains held on remote CI until the PR dogfood check and required matrix rows are green.

### Goal progress

- [x] This moves the Windows/Linux distribution local lane from 81.0% to 82.0% and overall MIT/productization from 92.0% to 92.1%.
- [x] Remaining Windows/Linux items stay visible: clean/non-WSL Linux desktop GUI proof, Windows ARM64, live Release / Readiness GUI screenshot, signed MSI/AuthentiCode, winget, website/status copy sync, and signing credentials.

### Evidence bundle

- [x] Windows proof bundle: `proofs/windows/studio-release-readiness-shell/win-20260603-230129/`.
- [x] WSL proof bundle: `proofs/linux/execution/studio-release-readiness-shell/wsl-20260603-230237/`.
- [x] Each bundle contains `garnet-studio-release-readiness-shell-proof.json`, Markdown summary, command stdout/stderr, copied Studio payload, and `MANIFEST.sha256`.
- [x] Evidence fields preserve the boundary: `release_readiness_shell_proven=true`, `studio_command_path_proven=true`, `source_included=false`, `provider_api_called=false`, `linux_enforcement_proven=false`, `non_wsl_linux_desktop_proven=false`, `signed_msi_claimed=false`, `winget_claimed=false`, and `windows_arm64_claimed=false`.

### Deferred / out of scope

- [x] Live Release / Readiness GUI screenshot remains separate proof.
- [x] Clean/non-WSL Linux desktop GUI install/launch remains open.
- [x] Linux seccomp or OS-sandbox enforcement remains real-kernel work, not WSL proof.
- [x] Signed MSI/AuthentiCode, SBOM/signing release proof, winget, Windows ARM64, production readiness, and v1.0 readiness remain unclaimed.
