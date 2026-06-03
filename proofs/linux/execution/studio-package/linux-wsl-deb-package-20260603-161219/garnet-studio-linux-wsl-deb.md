# Garnet Studio Linux WSL DEB Package Proof

- schema: `garnet.studio.linux_wsl_deb.v1`
- status: `passed`
- evidence tier: `wsl-linux-package-build-command-smoke`
- package: `target/release/bundle/deb/Garnet Studio_0.1.0_amd64.deb`
- package sha256: `cfb91f972ce6f6531329751d52fc653a565e537c892869c523ee92036c62c727`
- package architecture: `amd64`
- contains binary: `true`
- contains desktop file: `true`
- binary: `target/release/garnet-studio`
- binary sha256: `4efdb4f01ebfd6321717b12a9410784ed57dc029d7ea21c4ad4d5f6eeb554864`
- studio smoke: `passed`

## Commands

| Command | Status |
| --- | --- |
| `wsl-uname` | `passed` |
| `npm-install` | `passed` |
| `npm-build` | `passed` |
| `tauri-build-deb` | `passed` |
| `studio-smoke` | `passed` |
| `dpkg-info` | `passed` |
| `dpkg-contents` | `passed` |

## Honest Scope

- WSL is Linux package build and command-smoke evidence only
- not Linux desktop GUI launch proof
- not Linux seccomp or OS-sandbox enforcement
- not clean Linux install proof
- not signed, production, or v1.0 readiness
