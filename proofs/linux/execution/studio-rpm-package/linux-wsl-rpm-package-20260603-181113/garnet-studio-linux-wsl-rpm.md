# Garnet Studio Linux WSL RPM Package Proof

- schema: `garnet.studio.linux_wsl_rpm.v1`
- status: `passed`
- evidence tier: `wsl-linux-rpm-package-extract-command-smoke`
- package: `target/release/bundle/rpm/Garnet Studio-0.1.0-1.x86_64.rpm`
- package sha256: `2bcab818f9e038cf76a676dec28bd96c84ab68568e7bcf5440979198841d5ad6`
- package architecture: `x86_64`
- contains binary: `true`
- contains desktop file: `true`
- extracted binary: `target/linux-wsl-rpm-stage-20260603-181113/usr/bin/garnet-studio`
- extracted binary sha256: `117ff368ba44a2449fca0f732c975d2de5bc62a7c1a5460bf2948d2e450b6d3f`
- extracted studio smoke: `passed`
- RPM tooling installed by recorder: `false`

## RPM Tooling

| Tool | Path |
| --- | --- |
| `rpmbuild` | `/usr/bin/rpmbuild` |
| `rpm` | `/usr/bin/rpm` |
| `rpm2cpio` | `/usr/bin/rpm2cpio` |
| `cpio` | `/usr/bin/cpio` |

## Commands

| Command | Status |
| --- | --- |
| `wsl-uname` | `passed` |
| `rpm-tooling` | `passed` |
| `npm-install` | `passed` |
| `npm-build` | `passed` |
| `tauri-build-rpm` | `passed` |
| `rpm-info` | `passed` |
| `rpm-contents` | `passed` |
| `rpm-extract` | `passed` |
| `extracted-binary-ls` | `passed` |
| `extracted-studio-smoke` | `passed` |

## Honest Scope

- WSL is Linux RPM package extract and command-smoke evidence only
- not Linux desktop GUI launch proof
- not Linux seccomp or OS-sandbox enforcement
- not clean Linux install proof
- not privileged system package install proof
- not signed, production, or v1.0 readiness
