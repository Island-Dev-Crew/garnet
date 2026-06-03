# Garnet Studio Linux WSL Xvfb Window Capture Proof

- schema: `garnet.studio.linux_wsl_xvfb_window_capture.v1`
- status: `passed`
- evidence tier: `wsl-linux-xvfb-virtual-display-window-capture`
- source package proof: `deb` `C:/Users/IslandDevCrew/.config/superpowers/worktrees/garnet/agent-win-codex-s106-windows-cross-os-proof-phase1/proofs/linux/execution/studio-package-install/linux-wsl-deb-install-20260603-1200/garnet-studio-linux-wsl-deb-install.json`
- extracted binary: `target/linux-wsl-deb-install-stage-20260603-170305/usr/bin/garnet-studio`
- screenshot: `capture/screenshot.png`
- screenshot sha256: `5ad37eef50a2e0c82b99aef016b9a5336953ebd86563624f04d1f4e53068af6c`
- screenshot bytes: `340`
- window tree: `capture/xwininfo.txt`
- display info: `capture/xdpyinfo.txt`

## X11 Tooling

| Tool | Path |
| --- | --- |
| `xvfb-run` | `/usr/bin/xvfb-run` |
| `xwininfo` | `/usr/bin/xwininfo` |
| `xdpyinfo` | `/usr/bin/xdpyinfo` |
| `xwd` | `/usr/bin/xwd` |
| `convert` | `/usr/bin/convert` |
| `identify` | `/usr/bin/identify` |

## Commands

| Command | Exit | Status |
| --- | ---: | --- |
| `wsl-uname` | `0` | `passed` |
| `x11-capture-tooling` | `0` | `passed` |
| `xvfb-window-capture` | `0` | `passed` |

## Honest Scope

- WSL Xvfb virtual-display window-capture evidence only
- not Linux desktop GUI launch proof
- not Linux seccomp or OS-sandbox enforcement
- not clean Linux install proof
- not privileged system package install proof
- not signed, production, or v1.0 readiness
