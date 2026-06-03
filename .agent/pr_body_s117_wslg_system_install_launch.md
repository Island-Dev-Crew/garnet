## Dogfood Readiness

### Current truth

- [x] This PR records a narrow S117 Windows-lane increment: WSLg system package install/launch portability proof for Garnet Studio.
- [x] The proof verifies a Linux Tauri `.deb` build, `dpkg -i` install inside WSL, `/usr/bin/garnet-studio --studio-smoke`, WSLg/X11 window observation, and `dpkg -r` cleanup.
- [x] The proof is explicitly not clean/non-WSL Linux desktop evidence, not Linux seccomp/OS-sandbox enforcement, not signed/SBOM release evidence, not winget, not Windows ARM64, not production, and not v1.0.

### Local verification

- [x] `python scripts\test_smoke_garnet_studio_linux_wslg_install_launch.py` -> 6 passed.
- [x] `python scripts\smoke_garnet_studio_linux_wslg_install_launch.py --gate --format json` -> verified bundle `proofs/linux/execution/studio-wslg-system-install/linux-wslg-system-install-20260603-205713/garnet-studio-linux-wslg-system-install-launch.json`.
- [x] `wsl.exe -e sh -lc "cd '/mnt/c/Users/IslandDevCrew/.config/superpowers/worktrees/garnet/agent-win-codex-s106-windows-cross-os-proof-phase1' && python3 scripts/smoke_garnet_studio_linux_wslg_install_launch.py --gate --format json"` -> verified the same bundle from WSL.
- [x] `python scripts\test_garnet_windows_linux_studio_status.py` -> 14 passed.
- [x] `python scripts\test_garnet_mit_readiness_status.py` -> 33 passed.
- [x] `python scripts\garnet_mit_readiness_status.py --check-no-regression --format json` -> 91.9% overall, Windows/Linux distribution lane 80.0%, `linux_wsl_studio_wslg_system_install_launch=verified`.
- [x] `python scripts\garnet_windows_linux_studio_status.py --format json` -> `wslg-system-install-launch-verified-linux-desktop-still-open`.
- [x] `cargo fmt --all -- --check` -> passed.
- [x] `cargo test --workspace --no-fail-fast` -> passed with 0 failed.
- [x] `cargo clippy --workspace --all-targets -- -D warnings` -> passed.
- [x] `git diff --check` -> passed.

### Remote verification

- [x] Pending GitHub Actions on this PR; merge is held until CI is green, including the dogfood PR-body check.

### Desktop dogfood bundle

- [x] Committed proof bundle: `proofs/linux/execution/studio-wslg-system-install/linux-wslg-system-install-20260603-205713/`.
- [x] Bundle records WSL `uname`, WSLg display variables, npm/Tauri `.deb` build logs, `dpkg` before/install/remove status, installed binary metadata, smoke output, WSLg launch/window evidence, cleanup proof, and `MANIFEST.sha256`.
- [x] Manifest fields preserve the boundary: `wsl_is_enforcement=false`, `clean_linux_install_proven=false`, `linux_enforcement_proven=false`, `desktop_gui_launch_proven=false`, with only WSLg/system-install portability evidence verified.

### Deferred / out of scope

- [x] Real clean/non-WSL Linux desktop GUI install/launch remains open.
- [x] Linux seccomp or OS-sandbox enforcement remains Mac/Lane A real-kernel work, not WSL proof.
- [x] Signed MSI/AuthentiCode, winget, Windows ARM64, SBOM/signing release proof, production readiness, and v1.0 readiness remain unclaimed.
