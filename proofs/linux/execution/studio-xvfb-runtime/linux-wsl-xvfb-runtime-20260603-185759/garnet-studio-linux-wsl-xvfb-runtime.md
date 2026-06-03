# Garnet Studio Linux WSL Xvfb Runtime Proof

- schema: `garnet.studio.linux_wsl_xvfb_runtime.v1`
- status: `passed`
- evidence tier: `wsl-linux-xvfb-runtime-start-smoke`
- source package proof: `deb` `C:/Users/IslandDevCrew/.config/superpowers/worktrees/garnet/agent-win-codex-s106-windows-cross-os-proof-phase1/proofs/linux/execution/studio-package-install/linux-wsl-deb-install-20260603-1200/garnet-studio-linux-wsl-deb-install.json`
- extracted binary: `target/linux-wsl-deb-install-stage-20260603-170305/usr/bin/garnet-studio`
- extracted binary sha256: `2de52135e8c9d19594d89fe1b225e8e4b26196bb25e49e05c74ec540d69591c6`
- timeout_seconds: `8`
- runtime_seconds: `8.094`
- runtime exit code: `124`
- expected timeout exit code: `124`

## Xvfb Tooling

| Tool / variable | Value |
| --- | --- |
| `xvfb-run` | `/usr/bin/xvfb-run` |
| `timeout` | `/usr/bin/timeout` |
| `DISPLAY` | `:0` |
| `WAYLAND_DISPLAY` | `wayland-0` |
| `XDG_RUNTIME_DIR` | `/run/user/0/` |

## Commands

| Command | Exit | Status |
| --- | ---: | --- |
| `wsl-uname` | `0` | `passed` |
| `xvfb-tooling` | `0` | `passed` |
| `xvfb-runtime-start` | `124` | `passed` |

## Honest Scope

- WSL Xvfb runtime-start evidence only
- not Linux desktop GUI launch proof
- not Linux seccomp or OS-sandbox enforcement
- not clean Linux install proof
- not privileged system package install proof
- not signed, production, or v1.0 readiness
