# Garnet Studio Linux WSL DEB Install Proof

- schema: `garnet.studio.linux_wsl_deb_install.v1`
- status: `passed`
- evidence tier: `wsl-linux-package-extract-command-smoke`
- package: `target/release/bundle/deb/Garnet Studio_0.1.0_amd64.deb`
- package sha256: `96fde7fdc3f9b4a8632d98d5e815ef54d4e86fb49cd6ad60e8661e84b3e1b33f`
- package architecture: `amd64`
- contains binary: `true`
- contains desktop file: `true`
- extracted binary: `target/linux-wsl-deb-install-stage-20260603-170305/usr/bin/garnet-studio`
- extracted binary sha256: `2de52135e8c9d19594d89fe1b225e8e4b26196bb25e49e05c74ec540d69591c6`
- extracted studio smoke: `passed`

## Commands

| Command | Status |
| --- | --- |
| `wsl-uname` | `passed` |
| `npm-install` | `passed` |
| `npm-build` | `passed` |
| `tauri-build-deb` | `passed` |
| `dpkg-info` | `passed` |
| `dpkg-contents` | `passed` |
| `dpkg-extract` | `passed` |
| `extracted-binary-ls` | `passed` |
| `extracted-studio-smoke` | `passed` |

## Honest Scope

- WSL is Linux package extract and command-smoke evidence only
- not Linux desktop GUI launch proof
- not Linux seccomp or OS-sandbox enforcement
- not clean Linux install proof
- not privileged system package install proof
- not signed, production, or v1.0 readiness
