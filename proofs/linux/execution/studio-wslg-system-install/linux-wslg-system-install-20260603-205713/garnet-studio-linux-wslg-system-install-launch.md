# Garnet Studio Linux WSLg System Install Proof

- Schema: `garnet.studio.linux_wslg_system_install_launch.v1`
- Status: `passed`
- Evidence tier: `wslg-system-package-install-launch`
- Package: `target/release/bundle/deb/Garnet Studio_0.1.0_amd64.deb`
- Package name: `garnet-studio`
- Installed binary: `/usr/bin/garnet-studio`
- WSLg display: `:0` / `wayland-0`
- Process observed: `True`
- Window observed: `True`

## Honest Scope

- WSLg is WSL package install and GUI-launch evidence only
- not Linux desktop GUI proof outside WSLg
- not Linux seccomp or OS-sandbox enforcement
- not clean Linux install proof
- not signed, production, or v1.0 readiness

## Commands

- `wsl-uname`: passed (exit 0)
- `wslg-env`: passed (exit 0)
- `npm-install`: passed (exit 0)
- `npm-build`: passed (exit 0)
- `tauri-build-deb`: passed (exit 0)
- `dpkg-status-before`: passed (exit 0)
- `dpkg-install`: passed (exit 0)
- `dpkg-status-after-install`: passed (exit 0)
- `installed-binary-ls`: passed (exit 0)
- `installed-studio-smoke`: passed (exit 0)
- `wslg-launch`: passed (exit 0)
- `dpkg-remove`: passed (exit 0)
- `dpkg-status-after-remove`: passed (exit 0)
