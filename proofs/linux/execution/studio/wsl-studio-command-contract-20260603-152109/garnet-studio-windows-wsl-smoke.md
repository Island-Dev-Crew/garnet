# Garnet Studio Windows/WSL Smoke Proof

- Status: `passed`
- Target platform: `wsl`
- Platform tier: `execution/portability, not enforcement`
- Source included: `false`
- Provider API called: `false`

## Commands

| Command | Exit | Status |
| --- | ---: | --- |
| `wsl.exe -e bash -lc uname -a` | 0 | `passed` |
| `wsl.exe -e bash -lc cd <repo> && python3 scripts/garnet_windows_linux_studio_status.py --format json` | 0 | `passed` |
| `wsl.exe -e bash -lc cd <repo> && python3 scripts/test_garnet_windows_linux_studio_status.py` | 0 | `passed` |

## Honest Scope

- Windows `--studio-smoke` is Tauri backend smoke evidence, not signed/package-manager proof
- WSL rows are execution/portability evidence only, not Linux seccomp or OS-sandbox enforcement
- No Linux desktop GUI launch, AppImage/deb/rpm package, Wasmtime fuel, production, or v1.0 claim is made
- Source is not included in the evidence bundle and no provider API is called
