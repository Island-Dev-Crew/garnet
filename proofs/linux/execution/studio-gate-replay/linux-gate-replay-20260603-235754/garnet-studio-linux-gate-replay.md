# Garnet Studio Linux/Tauri Gate Replay Proof

- status: `passed`
- platform tier: `WSL/WSLg execution-portability only, not enforcement`
- all current Linux gates replayed: `true`

## Gates

| Gate | Status | Script |
|---|---|---|
| `linux-wsl-deb-package` | `passed` | `smoke_garnet_studio_linux_wsl_deb.py` |
| `linux-wsl-deb-install` | `passed` | `smoke_garnet_studio_linux_wsl_deb_install.py` |
| `linux-wsl-rpm-package` | `passed` | `smoke_garnet_studio_linux_wsl_rpm.py` |
| `linux-wsl-xvfb-runtime` | `passed` | `smoke_garnet_studio_linux_wsl_xvfb.py` |
| `linux-wsl-xvfb-window` | `passed` | `smoke_garnet_studio_linux_wsl_xvfb_window.py` |
| `linux-wslg-system-install` | `passed` | `smoke_garnet_studio_linux_wslg_install_launch.py` |
| `windows-wsl-domain-shell` | `passed` | `smoke_garnet_studio_domain_shell.py` |
| `windows-wsl-release-readiness-shell` | `passed` | `smoke_garnet_studio_release_readiness_shell.py` |

## Honest Scope

- WSL/WSLg execution/portability only
- not clean/non-WSL Linux desktop proof
- not Linux seccomp or OS-sandbox enforcement
- not signed, production, or v1.0 readiness
