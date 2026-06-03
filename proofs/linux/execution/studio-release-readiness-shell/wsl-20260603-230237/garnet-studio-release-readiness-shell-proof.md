# Garnet Studio Release / Readiness Shell Proof

- Status: `passed`
- Target platform: `wsl`
- Platform tier: `execution/portability, not enforcement`
- Release/readiness shell proven: `true`
- Source included: `false`
- Provider API called: `false`

## Commands

| Command | Exit | Status |
| --- | ---: | --- |
| `wsl.exe -e bash -lc uname -a` | 0 | `passed` |
| `wsl.exe -e bash -lc cd <repo> && cargo build -p garnet-cli --release` | 0 | `passed` |
| `wsl.exe -e bash -lc cd <repo> && cargo build --manifest-path apps/garnet-studio/src-tauri/Cargo.toml --release` | 0 | `passed` |
| `wsl.exe -e bash -lc cd <repo> && HOME=<repo>/target/wsl-release-readiness-shell-home-* ./target/release/garnet-studio --studio-release-readiness-smoke` | 0 | `passed` |

## Studio Payload

- Payload: `studio-payload/release-readiness-shell-smoke.json`
- Mode: `studio-release-readiness-smoke`
- Verified reporter commands: `converter-status, objective-pulse, windows-linux-studio-status, windows-vm-installer-status`

## Honest Scope

- Studio release/readiness shell proof exercises the Tauri command wrappers behind the Release / Readiness panel.
- WSL rows are execution/portability evidence only, not Linux seccomp or OS-sandbox enforcement.
- This is not clean/non-WSL Linux desktop GUI install/launch proof.
- No signed MSI, winget, Windows ARM64, production, or v1.0 claim is made.
- Source is not included in the evidence bundle and no provider API is called.
