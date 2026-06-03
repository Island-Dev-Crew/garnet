# Garnet Studio Windows/WSL Smoke Proof

- Status: `passed`
- Target platform: `windows`
- Platform tier: `windows-local-tauri-studio-smoke`
- Source included: `false`
- Provider API called: `false`

## Commands

| Command | Exit | Status |
| --- | ---: | --- |
| `npm run build` | 0 | `passed` |
| `cargo build --manifest-path apps/garnet-studio/src-tauri/Cargo.toml --release` | 0 | `passed` |
| `target/release/garnet-studio.exe --studio-smoke` | 0 | `passed` |

## Honest Scope

- Windows `--studio-smoke` is Tauri backend smoke evidence, not signed/package-manager proof
- WSL rows are execution/portability evidence only, not Linux seccomp or OS-sandbox enforcement
- No Linux desktop GUI launch, AppImage/deb/rpm package, Wasmtime fuel, production, or v1.0 claim is made
- Source is not included in the evidence bundle and no provider API is called
